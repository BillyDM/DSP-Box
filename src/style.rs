use iced::{button, container};

use iced_audio::knob;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    _Light,
    Dark,
}

impl Theme {
    pub fn page_container(&self) -> Box<dyn container::StyleSheet> {
        match self {
            Theme::_Light => Default::default(),
            Theme::Dark => dark::Container.into(),
        }
    }

    pub fn top_bar_container(&self) -> Box<dyn container::StyleSheet> {
        match self {
            Theme::_Light => Default::default(),
            Theme::Dark => dark::TopBarContainer.into(),
        }
    }

    pub fn button(&self) -> Box<dyn button::StyleSheet> {
        match self {
            Theme::_Light => Default::default(),
            Theme::Dark => dark::Button.into(),
        }
    }

    pub fn disabled_button(&self) -> Box<dyn button::StyleSheet> {
        match self {
            Theme::_Light => Default::default(),
            Theme::Dark => dark::DisabledButton.into(),
        }
    }

    pub fn bypassed_button(&self) -> Box<dyn button::StyleSheet> {
        match self {
            Theme::_Light => Default::default(),
            Theme::Dark => dark::BypassedButton.into(),
        }
    }

    pub fn knob(&self) -> Box<dyn knob::StyleSheet> {
        match self {
            Theme::_Light => Default::default(),
            Theme::Dark => dark::Knob.into(),
        }
    }
}

mod dark {
    use iced::{button, container, Background, Color};
    use iced_audio::knob;

    const BACKGROUND: Color = Color::from_rgb(
        0x20 as f32 / 255.0,
        0x24 as f32 / 255.0,
        0x2B as f32 / 255.0,
    );

    const PANEL_BACKGROUND: Color = Color::from_rgb(
        0x40 as f32 / 255.0,
        0x44 as f32 / 255.0,
        0x4B as f32 / 255.0,
    );

    const EMPTY: Color = Color::from_rgb(
        0x70 as f32 / 255.0,
        0x74 as f32 / 255.0,
        0x7B as f32 / 255.0,
    );

    const ACCENT: Color = Color::from_rgb(
        0x6F as f32 / 255.0,
        0xFF as f32 / 255.0,
        0xE9 as f32 / 255.0,
    );

    const ACTIVE: Color = Color::from_rgb(
        0x72 as f32 / 255.0,
        0x89 as f32 / 255.0,
        0xDA as f32 / 255.0,
    );

    const HOVERED: Color = Color::from_rgb(
        0x67 as f32 / 255.0,
        0x7B as f32 / 255.0,
        0xC4 as f32 / 255.0,
    );

    const DISABLED: Color = Color::from_rgb(
        0x6B as f32 / 255.0,
        0x6E as f32 / 255.0,
        0x7C as f32 / 255.0,
    );

    const BYPASS_ACTIVE: Color = Color::from_rgb(
        0xDA as f32 / 255.0,
        0x73 as f32 / 255.0,
        0x72 as f32 / 255.0,
    );

    const BYPASS_HOVERED: Color = Color::from_rgb(
        0xC4 as f32 / 255.0,
        0x67 as f32 / 255.0,
        0x67 as f32 / 255.0,
    );

    pub struct Container;
    impl container::StyleSheet for Container {
        fn style(&self) -> container::Style {
            container::Style {
                background: Some(Background::Color(BACKGROUND)),
                text_color: Some(Color::WHITE),
                ..container::Style::default()
            }
        }
    }

    pub struct TopBarContainer;
    impl container::StyleSheet for TopBarContainer {
        fn style(&self) -> container::Style {
            container::Style {
                background: Some(Background::Color(PANEL_BACKGROUND)),
                text_color: Some(Color::WHITE),
                ..container::Style::default()
            }
        }
    }

    pub struct Knob;
    impl knob::StyleSheet for Knob {
        fn active(&self) -> knob::Style {
            knob::Style::Arc(knob::ArcStyle {
                width: 2.5,
                empty_color: EMPTY,
                filled_color: ACCENT,
                notch: Some(knob::ArcNotch {
                    width: 2.5,
                    length_scale: 0.55.into(),
                    color: ACCENT,
                }),
            })
        }

        fn hovered(&self) -> knob::Style {
            self.active()
        }

        fn dragging(&self) -> knob::Style {
            self.active()
        }

        fn angle_range(&self) -> iced_audio::KnobAngleRange {
            iced_audio::KnobAngleRange::from_deg(40.0, 320.0)
        }
    }

    pub struct Button;
    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(ACTIVE)),
                border_radius: 3,
                text_color: Color::WHITE,
                ..button::Style::default()
            }
        }

        fn hovered(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(HOVERED)),
                text_color: Color::WHITE,
                ..self.active()
            }
        }

        fn pressed(&self) -> button::Style {
            button::Style {
                border_width: 1,
                border_color: Color::WHITE,
                ..self.hovered()
            }
        }
    }

    pub struct DisabledButton;
    impl button::StyleSheet for DisabledButton {
        fn active(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(DISABLED)),
                border_radius: 3,
                text_color: Color::WHITE,
                ..button::Style::default()
            }
        }

        fn hovered(&self) -> button::Style {
            self.active()
        }

        fn pressed(&self) -> button::Style {
            self.active()
        }
    }

    pub struct BypassedButton;
    impl button::StyleSheet for BypassedButton {
        fn active(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(BYPASS_ACTIVE)),
                border_radius: 3,
                text_color: Color::WHITE,
                ..button::Style::default()
            }
        }

        fn hovered(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(BYPASS_HOVERED)),
                text_color: Color::WHITE,
                ..self.active()
            }
        }

        fn pressed(&self) -> button::Style {
            button::Style {
                border_width: 1,
                border_color: Color::WHITE,
                ..self.hovered()
            }
        }
    }
}
