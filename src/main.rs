mod p4;

use iced::{executor, Application, Command, Element, Settings, Theme};

pub fn main() -> iced::Result {
    P4y::run(Settings::default())
}

struct P4y {}

#[derive(Debug, Clone)]
enum Message {}

impl Application for P4y {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (P4y {}, Command::none())
    }

    fn title(&self) -> String {
        String::from("p4y - Perforce Inspector")
    }

    fn update(&mut self, _message: Message) -> Command<Message> {
        Command::none()
    }

    fn view(&self) -> Element<'_, Message> {
        "Hello, p4y!".into()
    }
}
