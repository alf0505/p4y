mod p4;
mod tree;

use iced::{executor, Application, Command, Element, Settings, Theme};
use tree::TreeNode;

pub fn main() -> iced::Result {
    P4y::run(Settings::default())
}

struct P4y {
    root_node: Option<TreeNode>,
}

#[derive(Debug, Clone)]
enum Message {
    P4Error(String),
}

impl Application for P4y {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            P4y {
                root_node: Some(TreeNode::new_directory("root".to_string(), "//".to_string())),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("p4y - Perforce Inspector")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::P4Error(err) => {
                eprintln!("P4 Error: {}", err);
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        "Hello, p4y!".into()
    }
}
