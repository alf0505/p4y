use iced::{application, color, Color, Background, Border};
use iced::widget::{container, button, scrollable, text, text_input};

pub const BACKGROUND_MAIN: Color = color!(0x1e1e1e);
pub const BACKGROUND_SIDEBAR: Color = color!(0x252526);
pub const BORDER_COLOR: Color = color!(0x333333);
pub const ACCENT: Color = color!(0x007acc);
pub const TEXT_NORMAL: Color = color!(0xcccccc);
pub const TEXT_BRIGHT: Color = color!(0xffffff);
pub const HEADER_BG: Color = color!(0x2d2d2d);

#[derive(Debug, Clone, Copy, Default)]
pub struct PremiumDark;

impl application::StyleSheet for PremiumDark {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> application::Appearance {
        application::Appearance {
            background_color: BACKGROUND_MAIN,
            text_color: TEXT_NORMAL,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Container {
    #[default]
    Main,
    Sidebar,
    Header,
    Box,
    Overlay,
}

impl container::StyleSheet for PremiumDark {
    type Style = Container;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        match style {
            Container::Main => container::Appearance {
                background: Some(Background::Color(BACKGROUND_MAIN)),
                text_color: Some(TEXT_NORMAL),
                ..Default::default()
            },
            Container::Sidebar => container::Appearance {
                background: Some(Background::Color(BACKGROUND_SIDEBAR)),
                text_color: Some(TEXT_NORMAL),
                border: Border {
                    color: BORDER_COLOR,
                    width: 1.0,
                    ..Default::default()
                },
                ..Default::default()
            },
            Container::Header => container::Appearance {
                background: Some(Background::Color(HEADER_BG)),
                text_color: Some(TEXT_BRIGHT),
                border: Border {
                    color: BORDER_COLOR,
                    width: 1.0,
                    ..Default::default()
                },
                ..Default::default()
            },
            Container::Box => container::Appearance {
                background: Some(Background::Color(color!(0x2a2d2e))),
                text_color: Some(TEXT_NORMAL),
                border: Border {
                    color: BORDER_COLOR,
                    width: 1.0,
                    radius: 2.0.into(),
                },
                ..Default::default()
            },
            Container::Overlay => container::Appearance {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.7))),
                ..Default::default()
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Button {
    #[default]
    Primary,
    Secondary,
    Ghost,
    ListItem {
        selected: bool,
    },
}

impl button::StyleSheet for PremiumDark {
    type Style = Button;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        match style {
            Button::Primary => button::Appearance {
                background: Some(Background::Color(ACCENT)),
                text_color: TEXT_BRIGHT,
                border: Border {
                    radius: 2.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            Button::Secondary => button::Appearance {
                background: Some(Background::Color(color!(0x3a3d41))),
                text_color: TEXT_NORMAL,
                border: Border {
                    radius: 2.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            Button::Ghost => button::Appearance {
                text_color: TEXT_NORMAL,
                ..Default::default()
            },
            Button::ListItem { selected } => button::Appearance {
                background: if *selected {
                    Some(Background::Color(color!(0x37373d)))
                } else {
                    None
                },
                text_color: if *selected { TEXT_BRIGHT } else { TEXT_NORMAL },
                border: Border {
                    color: if *selected { ACCENT } else { Color::TRANSPARENT },
                    width: if *selected { 1.0 } else { 0.0 },
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);
        match style {
            Button::Primary => button::Appearance {
                background: Some(Background::Color(color!(0x1c97ea))),
                ..active
            },
            Button::ListItem { .. } => button::Appearance {
                background: Some(Background::Color(color!(0x2a2d2e))),
                text_color: TEXT_BRIGHT,
                ..active
            },
            Button::Ghost => button::Appearance {
                background: Some(Background::Color(color!(0x3a3d41))),
                text_color: TEXT_BRIGHT,
                ..active
            },
            _ => button::Appearance {
                background: Some(Background::Color(color!(0x45494e))),
                ..active
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Text {
    #[default]
    Normal,
    Bright,
    Accent,
    Color(Color),
}

impl From<Color> for Text {
    fn from(color: Color) -> Self {
        Text::Color(color)
    }
}

impl text::StyleSheet for PremiumDark {
    type Style = Text;

    fn appearance(&self, style: Self::Style) -> text::Appearance {
        match style {
            Text::Normal => text::Appearance {
                color: Some(TEXT_NORMAL),
            },
            Text::Bright => text::Appearance {
                color: Some(TEXT_BRIGHT),
            },
            Text::Accent => text::Appearance {
                color: Some(ACCENT),
            },
            Text::Color(c) => text::Appearance {
                color: Some(c),
            },
        }
    }
}

impl scrollable::StyleSheet for PremiumDark {
    type Style = ();

    fn active(&self, _style: &Self::Style) -> scrollable::Appearance {
        scrollable::Appearance {
            scrollbar: scrollable::Scrollbar {
                background: Some(BACKGROUND_MAIN.into()),
                border: Border {
                    radius: 0.0.into(),
                    ..Default::default()
                },
                scroller: scrollable::Scroller {
                    color: color!(0x4f4f4f),
                    border: Border {
                        radius: 0.0.into(),
                        ..Default::default()
                    },
                },
            },
            gap: None,
            container: container::Appearance::default(),
        }
    }

    fn hovered(&self, _style: &Self::Style, _is_mouse_over_scrollbar: bool) -> scrollable::Appearance {
        let active = self.active(&());
        scrollable::Appearance {
            scrollbar: scrollable::Scrollbar {
                scroller: scrollable::Scroller {
                    color: color!(0x6f6f6f),
                    ..active.scrollbar.scroller
                },
                ..active.scrollbar
            },
            ..active
        }
    }
}

impl text_input::StyleSheet for PremiumDark {
    type Style = ();

    fn active(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(color!(0x3c3c3c)),
            border: Border {
                color: BORDER_COLOR,
                width: 1.0,
                radius: 2.0.into(),
            },
            icon_color: TEXT_NORMAL,
        }
    }

    fn focused(&self, _style: &Self::Style) -> text_input::Appearance {
        let active = self.active(&());
        text_input::Appearance {
            border: Border {
                color: ACCENT,
                ..active.border
            },
            ..active
        }
    }

    fn disabled(&self, _style: &Self::Style) -> text_input::Appearance {
        let active = self.active(&());
        text_input::Appearance {
            background: Background::Color(color!(0x2d2d2d)),
            border: Border {
                color: color!(0x444444),
                ..active.border
            },
            icon_color: color!(0x555555),
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        color!(0x888888)
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        TEXT_BRIGHT
    }

    fn disabled_color(&self, _style: &Self::Style) -> Color {
        color!(0x666666)
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        color!(0x264f78)
    }
}
