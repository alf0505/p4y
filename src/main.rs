mod p4;
mod tree;

use iced::widget::{button, column, container, row, scrollable, text};
use iced::{executor, Application, Command, Element, Settings, Theme};
use tree::{TreeNode, fetch_children, find_node_mut};
use p4::{Changelist, ChangelistDetail, fetch_history, fetch_cl_detail, fetch_file_content};

pub fn main() -> iced::Result {
    P4y::run(Settings::default())
}

struct P4y {
    root_node: Option<TreeNode>,
    history: Vec<Changelist>,
    selected_cl: Option<ChangelistDetail>,
    loading_history: bool,
    loading_detail: bool,
    content_modal: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Clone)]
enum Message {
    P4Error(String),
    ToggleExpanded(String),
    ChildrenLoaded(String, Vec<TreeNode>),
    FileSelected(String),
    HistoryLoaded(Vec<Changelist>),
    CLSelected(u32),
    CLDetailLoaded(ChangelistDetail),
    ViewContent(String),
    ContentLoaded(String),
    CloseModal,
    ClearError,
}

impl Application for P4y {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let root_path = "//".to_string();
        let path_for_fetch = root_path.clone();
        (
            P4y {
                root_node: Some(TreeNode::new_directory("root".to_string(), root_path.clone())),
                history: Vec::new(),
                selected_cl: None,
                loading_history: false,
                loading_detail: false,
                content_modal: None,
                error: None,
            },
            Command::perform(
                async move { fetch_children(&path_for_fetch).await },
                move |res| match res {
                    Ok(children) => Message::ChildrenLoaded(root_path.clone(), children),
                    Err(e) => Message::P4Error(e.to_string()),
                }
            ),
        )
    }

    fn title(&self) -> String {
        String::from("p4y - Perforce Inspector")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::P4Error(err) => {
                self.error = Some(err);
                Command::none()
            }
            Message::ClearError => {
                self.error = None;
                Command::none()
            }
            Message::ToggleExpanded(path) => {
                let mut needs_fetching = false;
                if let Some(ref mut root) = self.root_node {
                    if let Some(node) = find_node_mut(root, &path) {
                        node.toggle_expanded();
                        if node.is_expanded && node.children.is_none() {
                            needs_fetching = true;
                        }
                    }
                }
                if needs_fetching {
                    let path_clone = path.clone();
                    let path_for_fetch = path.clone();
                    Command::perform(
                        async move { fetch_children(&path_for_fetch).await },
                        move |res| match res {
                            Ok(children) => Message::ChildrenLoaded(path_clone.clone(), children),
                            Err(e) => Message::P4Error(e.to_string()),
                        }
                    )
                } else {
                    Command::none()
                }
            }
            Message::ChildrenLoaded(path, children) => {
                if let Some(ref mut root) = self.root_node {
                    root.update_node(&path, children);
                }
                Command::none()
            }
            Message::FileSelected(path) => {
                self.loading_history = true;
                let path_for_fetch = path.clone();
                Command::perform(
                    async move { fetch_history(&path_for_fetch).await },
                    |res| match res {
                        Ok(history) => Message::HistoryLoaded(history),
                        Err(e) => Message::P4Error(e.to_string()),
                    }
                )
            }
            Message::HistoryLoaded(history) => {
                self.history = history;
                self.loading_history = false;
                Command::none()
            }
            Message::CLSelected(cl_id) => {
                self.loading_detail = true;
                Command::perform(fetch_cl_detail(cl_id), |res| match res {
                    Ok(detail) => Message::CLDetailLoaded(detail),
                    Err(e) => Message::P4Error(e.to_string()),
                })
            }
            Message::CLDetailLoaded(detail) => {
                self.selected_cl = Some(detail);
                self.loading_detail = false;
                Command::none()
            }
            Message::ViewContent(path_with_rev) => {
                Command::perform(fetch_file_content(path_with_rev), |res| match res {
                    Ok(content) => Message::ContentLoaded(content),
                    Err(e) => Message::P4Error(e.to_string()),
                })
            }
            Message::ContentLoaded(content) => {
                self.content_modal = Some(content);
                Command::none()
            }
            Message::CloseModal => {
                self.content_modal = None;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let tree_content = if let Some(ref root) = self.root_node {
            scrollable(view_tree(root, 0))
        } else {
            scrollable(text("Loading tree..."))
        };

        let tree_pane = container(tree_content)
            .width(300)
            .height(iced::Length::Fill)
            .padding(10)
            .style(iced::theme::Container::Box);

        let history_list = if self.loading_history {
            column![text("Loading history...")]
        } else {
            let mut col = column![].spacing(5);
            for cl in &self.history {
                col = col.push(
                    button(
                        column![
                            text(format!("CL {} - {} ({})", cl.id, cl.author, cl.date)).size(14),
                            text(&cl.description).size(12),
                        ]
                    )
                    .width(iced::Length::Fill)
                    .on_press(Message::CLSelected(cl.id))
                    .style(iced::theme::Button::Secondary)
                );
            }
            col
        };

        let history_pane = container(scrollable(history_list))
            .width(350)
            .height(iced::Length::Fill)
            .padding(10)
            .style(iced::theme::Container::Box);

        let detail_content = if self.loading_detail {
            column![text("Loading CL details...")]
        } else if let Some(ref detail) = self.selected_cl {
            let mut col = column![
                text(format!("Changelist {}", detail.id)).size(20),
                text(format!("Author: {}", detail.author)),
                text(format!("Date: {}", detail.date)),
                text("Description:").size(16),
                text(&detail.description),
                text("Affected Files:").size(16),
            ].spacing(10);

            for file in &detail.affected_files {
                let file_path = file.clone();
                let cl_id = detail.id;
                col = col.push(
                    row![
                        text(file).size(12),
                        button("View").on_press(Message::ViewContent(format!("{}@{}", file_path, cl_id))).padding(2)
                    ].spacing(10)
                );
            }
            col
        } else {
            column![text("Select a changelist to see details")]
        };

        let detail_pane = container(scrollable(detail_content))
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .padding(10)
            .style(iced::theme::Container::Box);

        let main_content = row![tree_pane, history_pane, detail_pane];

        let base_view: Element<_> = if let Some(ref err) = self.error {
            column![
                main_content,
                container(
                    row![
                        text(format!("Error: {}", err)).style(iced::theme::Text::Color(iced::Color::from_rgb(0.8, 0.0, 0.0))),
                        button("Clear").on_press(Message::ClearError)
                    ].spacing(10)
                )
                .width(iced::Length::Fill)
                .padding(10)
                .style(iced::theme::Container::Box)
            ].into()
        } else {
            main_content.into()
        };

        if let Some(ref content) = self.content_modal {
            let modal_content = container(
                column![
                    row![
                        iced::widget::horizontal_space(),
                        button("Close").on_press(Message::CloseModal)
                    ],
                    scrollable(text(content).size(12))
                ].spacing(10)
            )
            .width(iced::Length::FillPortion(8))
            .height(iced::Length::FillPortion(8))
            .padding(20)
            .style(iced::theme::Container::Box);

            container(modal_content)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .center_x()
                .center_y()
                .style(|_theme: &Theme| {
                    iced::widget::container::Appearance {
                        background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
                        ..Default::default()
                    }
                })
                .into()
        } else {
            base_view
        }
    }
}

fn view_tree(node: &TreeNode, indent: u16) -> Element<'_, Message> {
    let mut content = column![].spacing(2);

    let prefix = if node.is_directory() {
        if node.is_expanded { "[-] " } else { "[+] " }
    } else {
        "  "
    };

    let on_press = if node.is_directory() {
        Message::ToggleExpanded(node.path.clone())
    } else {
        Message::FileSelected(node.path.clone())
    };

    let label = button(text(format!("{}{}", prefix, node.name)).size(14))
        .padding(2)
        .style(iced::theme::Button::Text)
        .on_press(on_press);

    content = content.push(
        row![iced::widget::horizontal_space().width(iced::Length::Fixed(indent as f32 * 15.0)), label]
    );

    if node.is_expanded {
        if let Some(ref children) = node.children {
            for child in children {
                content = content.push(view_tree(child, indent + 1));
            }
        }
    }

    content.into()
}
