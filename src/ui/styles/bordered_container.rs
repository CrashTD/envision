use iced::{
    widget::container::{Appearance, StyleSheet},
    Color,
};

const BORDER_COLOR: Color = Color {
    r: 0.5,
    g: 0.5,
    b: 0.5,
    a: 1.0,
};

#[derive(Default)]
pub struct RoundedBorderedContainer;

impl StyleSheet for RoundedBorderedContainer {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            border_width: 1.0,
            border_color: BORDER_COLOR,
            border_radius: 3.0,
            ..Default::default()
        }
    }
}

pub struct BorderedContainer;

impl StyleSheet for BorderedContainer {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            border_width: 1.0,
            border_color: BORDER_COLOR,
            ..Default::default()
        }
    }
}
