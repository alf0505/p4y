mod p4;
mod tree;

use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{executor, Application, Command, Element, Settings, Theme};
use tree::{TreeNode, fetch_children, find_node_mut};
use p4::{Changelist, ChangelistDetail, P4Settings, fetch_history, fetch_cl_detail, fetch_file_content};

pub fn main() -> iced::Result {
    P4y::run(Settings::default())
}

struct P4y {
    root_node: Option<TreeNode>,
    history: Vec<Changelist>,
    selected_cl: Option<ChangelistDetail>,
    loading_tree: bool,
    loading_history: bool,
    loading_detail: bool,
    content_modal: Option<String>,
    error: Option<String>,
    settings: P4Settings,
    show_settings: bool,
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
    ToggleSettings,
    P4PortChanged(String),
    P4UserChanged(String),
    P4ClientChanged(String),
    Refresh,
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
                loading_tree: true,
                loading_history: false,
                loading_detail: false,
                content_modal: None,
                error: None,
                settings: P4Settings::default(),
                show_settings: false,
            },
            Command::perform(
                async move { fetch_children(&path_for_fetch, None).await },
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
                self.loading_tree = false;
                self.loading_history = false;
                self.loading_detail = false;
                Command::none()
            }
            Message::ClearError => {
                self.error = None;
                Command::none()
            }
            Message::Refresh => {
                self.loading_tree = true;
                let root_path = "//".to_string();
                let path_for_fetch = root_path.clone();
                let settings = self.settings.clone();
                Command::perform(
                    async move { fetch_children(&path_for_fetch, Some(settings)).await },
                    move |res| match res {
                        Ok(children) => Message::ChildrenLoaded(root_path.clone(), children),
                        Err(e) => Message::P4Error(e.to_string()),
                    }
                )
            }
            Message::ToggleSettings => {
                self.show_settings = !self.show_settings;
                Command::none()
            }
            Message::P4PortChanged(port) => {
                self.settings.port = port;
                Command::none()
            }
            Message::P4UserChanged(user) => {
                self.settings.user = user;
                Command::none()
            }
            Message::P4ClientChanged(client) => {
                self.settings.client = client;
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
                    let settings = self.settings.clone();
                    Command::perform(
                        async move { fetch_children(&path_for_fetch, Some(settings)).await },
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
                self.loading_tree = false;
                Command::none()
            }
            Message::FileSelected(path) => {
                self.loading_history = true;
                let path_for_fetch = path.clone();
                let settings = self.settings.clone();
                Command::perform(
                    async move { fetch_history(&path_for_fetch, Some(settings)).await },
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
                let settings = self.settings.clone();
                Command::perform(fetch_cl_detail(cl_id, Some(settings)), |res| match res {
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
                let settings = self.settings.clone();
                Command::perform(async move { fetch_file_content(&path_with_rev, Some(settings)).await }, |res| match res {
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
        let toolbar = container(
            row![
                text("p4y").size(24).width(iced::Length::Fill),
                button("Refresh").on_press(Message::Refresh),
                button("Settings").on_press(Message::ToggleSettings),
            ]
            .spacing(10)
            .padding(10)
            .align_items(iced::Alignment::Center)
        )
        .style(iced::theme::Container::Box);

        let tree_content = if self.loading_tree {
            column![text("Loading tree...")]
        } else if let Some(ref root) = self.root_node {
            column![
                text("Depot Tree").size(18),
                scrollable(view_tree(root, 0))
            ].spacing(10)
        } else {
            column![text("No tree data")]
        };

        let tree_pane = container(tree_content)
            .width(300)
            .height(iced::Length::Fill)
            .padding(10)
            .style(iced::theme::Container::Box);

        let history_list = if self.loading_history {
            column![
                text("File History").size(18),
                text("Loading history...").style(iced::theme::Text::Color(iced::Color::from_rgb(0.5, 0.5, 0.5)))
            ].spacing(10)
        } else {
            let col = column![text("File History").size(18)].spacing(5);
            let history_scroll = if self.history.is_empty() {
                scrollable(text("Select a file to see its history"))
            } else {
                let mut items = column![].spacing(5);
                for cl in &self.history {
                    items = items.push(
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
                scrollable(items)
            };
            col.push(history_scroll)
        };

        let history_pane = container(history_list)
            .width(350)
            .height(iced::Length::Fill)
            .padding(10)
            .style(iced::theme::Container::Box);

        let detail_content = if self.loading_detail {
            column![
                text("CL Details").size(18),
                text("Loading CL details...").style(iced::theme::Text::Color(iced::Color::from_rgb(0.5, 0.5, 0.5)))
            ].spacing(10)
        } else if let Some(ref detail) = self.selected_cl {
            let col = column![
                text("CL Details").size(18),
                text(format!("Changelist {}", detail.id)).size(22),
                row![
                    text(format!("Author: {}", detail.author)).size(14),
                    text(format!("Date: {}", detail.date)).size(14),
                ].spacing(20),
                container(
                    scrollable(text(&detail.description).size(14))
                )
                .padding(10)
                .width(iced::Length::Fill)
                .height(iced::Length::Fixed(100.0))
                .style(iced::theme::Container::Box),
                text("Affected Files:").size(16),
            ].spacing(10);

            let mut files_col = column![].spacing(5);
            for file in &detail.affected_files {
                let file_path = file.clone();
                let cl_id = detail.id;
                files_col = files_col.push(
                    row![
                        text(file).size(12).width(iced::Length::Fill),
                        button("View").on_press(Message::ViewContent(format!("{}@{}", file_path, cl_id))).padding(2)
                    ].spacing(10)
                );
            }
            col.push(scrollable(files_col))
        } else {
            column![
                text("CL Details").size(18),
                text("Select a changelist to see details")
            ].spacing(10)
        };

        let detail_pane = container(detail_content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .padding(10)
            .style(iced::theme::Container::Box);

        let main_content = row![tree_pane, history_pane, detail_pane];

        let mut content = column![toolbar, main_content];

        if let Some(ref err) = self.error {
            content = content.push(
                container(
                    row![
                        text(format!("Error: {}", err)).style(iced::theme::Text::Color(iced::Color::from_rgb(0.8, 0.0, 0.0))),
                        button("Clear").on_press(Message::ClearError)
                    ].spacing(10)
                )
                .width(iced::Length::Fill)
                .padding(10)
                .style(iced::theme::Container::Box)
            );
        }

        let base_view: Element<_> = content.into();

        let mut final_view = base_view;

        if self.show_settings {
            let settings_content = container(
                column![
                    row![
                        text("Settings").size(24).width(iced::Length::Fill),
                        button("Close").on_press(Message::ToggleSettings)
                    ],
                    column![
                        text("P4PORT"),
                        text_input("e.g. perforce:1666", &self.settings.port).on_input(Message::P4PortChanged),
                        text("P4USER"),
                        text_input("username", &self.settings.user).on_input(Message::P4UserChanged),
                        text("P4CLIENT"),
                        text_input("workspace", &self.settings.client).on_input(Message::P4ClientChanged),
                    ].spacing(10)
                ].spacing(20)
            )
            .width(400)
            .padding(20)
            .style(iced::theme::Container::Box);

            final_view = container(settings_content)
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
                .into();
        }

        if let Some(ref content) = self.content_modal {
            let modal_content = container(
                column![
                    row![
                        text("File Content").size(20).width(iced::Length::Fill),
                        button("Close").on_press(Message::CloseModal)
                    ],
                    container(scrollable(text(content).size(12)))
                        .width(iced::Length::Fill)
                        .height(iced::Length::Fill)
                        .style(iced::theme::Container::Box)
                ].spacing(10)
            )
            .width(iced::Length::FillPortion(8))
            .height(iced::Length::FillPortion(8))
            .padding(20)
            .style(iced::theme::Container::Box);

            final_view = container(modal_content)
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
                .into();
        }

        final_view
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
