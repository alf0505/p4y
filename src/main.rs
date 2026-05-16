mod p4;
mod tree;
mod style;

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
    selected_path: Option<String>,
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
    type Theme = style::PremiumDark;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let root_path = "//".to_string();
        let path_for_fetch = root_path.clone();
        (
            P4y {
                root_node: Some(TreeNode::new_directory("root".to_string(), root_path.clone())),
                history: Vec::new(),
                selected_cl: None,
                selected_path: None,
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

    fn theme(&self) -> Self::Theme {
        style::PremiumDark
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
                self.selected_path = Some(path.clone());
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

    fn view(&self) -> Element<'_, Message, style::PremiumDark> {
        let toolbar = container(
            row![
                text("p4y").size(24).width(iced::Length::Fill).style(style::TEXT_BRIGHT),
                button(text("Refresh").size(14))
                    .on_press(Message::Refresh)
                    .style(style::Button::Secondary)
                    .padding([6, 12]),
                button(text("Settings").size(14))
                    .on_press(Message::ToggleSettings)
                    .style(style::Button::Secondary)
                    .padding([6, 12]),
            ]
            .spacing(12)
            .padding(12)
            .align_items(iced::Alignment::Center)
        )
        .style(style::Container::Sidebar);

        // Tree Pane
        let tree_header = container(
            text("DEPOT TREE").size(12).style(style::TEXT_BRIGHT)
        )
        .width(iced::Length::Fill)
        .padding([8, 12])
        .style(style::Container::Header);

        let tree_content = if self.loading_tree {
            container(text("Loading tree...").style(style::TEXT_NORMAL))
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .center_x()
                .center_y()
                .into()
        } else if let Some(ref root) = self.root_node {
            scrollable(view_tree(root, 0, self.selected_path.as_deref())).into()
        } else {
            container(text("No tree data").style(style::TEXT_NORMAL))
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .center_x()
                .center_y()
                .into()
        };

        let tree_pane = container(
            column![tree_header, tree_content]
        )
        .width(300)
        .height(iced::Length::Fill)
        .style(style::Container::Sidebar);

        // History Pane
        let history_header = container(
            text("FILE HISTORY").size(12).style(style::TEXT_BRIGHT)
        )
        .width(iced::Length::Fill)
        .padding([8, 12])
        .style(style::Container::Header);

        let history_content = if self.loading_history {
            container(text("Loading history...").style(style::TEXT_NORMAL))
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .center_x()
                .center_y()
                .into()
        } else {
            if self.history.is_empty() {
                container(text("Select a file to see its history").size(14).style(style::TEXT_NORMAL))
                    .width(iced::Length::Fill)
                    .height(iced::Length::Fill)
                    .center_x()
                    .center_y()
                    .into()
            } else {
                let mut items = column![].spacing(1);
                for cl in &self.history {
                    let is_selected = self.selected_cl.as_ref().map(|d| d.id == cl.id).unwrap_or(false);
                    items = items.push(
                        button(
                            column![
                                row![
                                    text(format!("CL {}", cl.id)).size(14).style(style::TEXT_BRIGHT),
                                    iced::widget::horizontal_space().width(iced::Length::Fill),
                                    text(&cl.date).size(12).style(style::TEXT_NORMAL),
                                ],
                                text(&cl.author).size(12).style(style::ACCENT),
                                text(&cl.description).size(12).style(style::TEXT_NORMAL),
                            ].spacing(4)
                        )
                        .width(iced::Length::Fill)
                        .padding(10)
                        .on_press(Message::CLSelected(cl.id))
                        .style(style::Button::ListItem { selected: is_selected })
                    );
                }
                scrollable(items).into()
            }
        };

        let history_pane = container(
            column![history_header, history_content]
        )
        .width(350)
        .height(iced::Length::Fill)
        .style(style::Container::Sidebar);

        // Detail Pane
        let detail_header = container(
            text("CHANGELIST DETAILS").size(12).style(style::TEXT_BRIGHT)
        )
        .width(iced::Length::Fill)
        .padding([8, 12])
        .style(style::Container::Header);

        let detail_content = if self.loading_detail {
            container(text("Loading CL details...").style(style::TEXT_NORMAL))
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .center_x()
                .center_y()
                .into()
        } else if let Some(ref detail) = self.selected_cl {
            let info = column![
                text(format!("Changelist {}", detail.id)).size(24).style(style::TEXT_BRIGHT),
                row![
                    text(format!("Author: {}", detail.author)).size(14).style(style::ACCENT),
                    text(format!("Date: {}", detail.date)).size(14).style(style::TEXT_NORMAL),
                ].spacing(20),
                container(
                    scrollable(text(&detail.description).size(14).style(style::TEXT_NORMAL))
                )
                .padding(12)
                .width(iced::Length::Fill)
                .height(iced::Length::Fixed(120.0))
                .style(style::Container::Box),
                text("AFFECTED FILES").size(12).style(style::TEXT_BRIGHT),
            ].spacing(16);

            let mut files_col = column![].spacing(1);
            for file in &detail.affected_files {
                let file_path = file.clone();
                let cl_id = detail.id;
                files_col = files_col.push(
                    container(
                        row![
                            text(file).size(13).style(style::TEXT_NORMAL).width(iced::Length::Fill),
                            button(text("View").size(12))
                                .on_press(Message::ViewContent(format!("{}@{}", file_path, cl_id)))
                                .style(style::Button::Primary)
                                .padding([4, 10])
                        ].spacing(10).align_items(iced::Alignment::Center)
                    )
                    .padding([4, 8])
                    .style(style::Container::Main)
                );
            }
            column![info, scrollable(files_col)].padding(20).spacing(16).into()
        } else {
            container(text("Select a changelist to see details").style(style::TEXT_NORMAL))
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .center_x()
                .center_y()
                .into()
        };

        let detail_pane = container(
            column![detail_header, detail_content]
        )
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .style(style::Container::Main);

        let main_content = row![tree_pane, history_pane, detail_pane];

        let mut content = column![toolbar, main_content];

        if let Some(ref err) = self.error {
            content = content.push(
                container(
                    row![
                        text(format!("Error: {}", err)).style(iced::Color::from_rgb(0.9, 0.3, 0.3)),
                        iced::widget::horizontal_space().width(iced::Length::Fill),
                        button(text("Clear").size(12))
                            .on_press(Message::ClearError)
                            .style(style::Button::Secondary)
                    ].spacing(10).align_items(iced::Alignment::Center)
                )
                .width(iced::Length::Fill)
                .padding(10)
                .style(style::Container::Header)
            );
        }

        let base_view: Element<_, style::PremiumDark> = content.into();

        let mut final_view = base_view;

        if self.show_settings {
            let settings_content = container(
                column![
                    row![
                        text("Settings").size(24).style(style::TEXT_BRIGHT).width(iced::Length::Fill),
                        button(text("✕").size(16))
                            .on_press(Message::ToggleSettings)
                            .style(style::Button::Ghost)
                    ],
                    column![
                        text("P4PORT").size(12).style(style::TEXT_NORMAL),
                        text_input("e.g. perforce:1666", &self.settings.port).on_input(Message::P4PortChanged).padding(10),
                        text("P4USER").size(12).style(style::TEXT_NORMAL),
                        text_input("username", &self.settings.user).on_input(Message::P4UserChanged).padding(10),
                        text("P4CLIENT").size(12).style(style::TEXT_NORMAL),
                        text_input("workspace", &self.settings.client).on_input(Message::P4ClientChanged).padding(10),
                    ].spacing(12)
                ].spacing(24)
            )
            .width(450)
            .padding(24)
            .style(style::Container::Box);

            final_view = container(settings_content)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .center_x()
                .center_y()
                .style(|_theme: &style::PremiumDark| {
                    iced::widget::container::Appearance {
                        background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.7))),
                        ..Default::default()
                    }
                })
                .into();
        }

        if let Some(ref content) = self.content_modal {
            let modal_content = container(
                column![
                    row![
                        text("File Content").size(20).style(style::TEXT_BRIGHT).width(iced::Length::Fill),
                        button(text("Close").size(14))
                            .on_press(Message::CloseModal)
                            .style(style::Button::Primary)
                            .padding([6, 12])
                    ].align_items(iced::Alignment::Center),
                    container(scrollable(text(content).size(13).style(style::TEXT_NORMAL)))
                        .width(iced::Length::Fill)
                        .height(iced::Length::Fill)
                        .padding(12)
                        .style(style::Container::Main)
                ].spacing(16)
            )
            .width(iced::Length::FillPortion(9))
            .height(iced::Length::FillPortion(9))
            .padding(24)
            .style(style::Container::Box);

            final_view = container(modal_content)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .center_x()
                .center_y()
                .style(|_theme: &style::PremiumDark| {
                    iced::widget::container::Appearance {
                        background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.7))),
                        ..Default::default()
                    }
                })
                .into();
        }

        final_view
    }
}

fn view_tree<'a>(node: &'a TreeNode, indent: u16, selected_path: Option<&'a str>) -> Element<'a, Message, style::PremiumDark> {
    let mut content = column![].spacing(0);

    let prefix = if node.is_directory() {
        if node.is_expanded { "▼ " } else { "▶ " }
    } else {
        "  "
    };

    let on_press = if node.is_directory() {
        Message::ToggleExpanded(node.path.clone())
    } else {
        Message::FileSelected(node.path.clone())
    };

    let is_selected = selected_path == Some(&node.path);

    let label = button(
        row![
            iced::widget::horizontal_space().width(iced::Length::Fixed(indent as f32 * 12.0)),
            text(format!("{}{}", prefix, node.name)).size(13)
        ].align_items(iced::Alignment::Center)
    )
    .width(iced::Length::Fill)
    .padding([3, 8])
    .style(style::Button::ListItem { selected: is_selected })
    .on_press(on_press);

    content = content.push(label);

    if node.is_expanded {
        if let Some(ref children) = node.children {
            for child in children {
                content = content.push(view_tree(child, indent + 1, selected_path));
            }
        }
    }

    content.into()
}
