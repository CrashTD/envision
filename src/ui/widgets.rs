#[macro_use]
pub mod widgets {
    use iced::Length;
    use iced_aw::quad::{Quad, InnerBounds};
    pub fn vseparator() -> Quad {
        Quad {
            width: Length::Fixed(1.0),
            height: Length::Fill,
            inner_bounds: InnerBounds::Ratio(0.0001, 0.9999),
            ..Default::default()
        }
    }

    pub fn hseparator() -> Quad {
        Quad {
            width: Length::Fill,
            height: Length::Fixed(1.0),
            inner_bounds: InnerBounds::Ratio(0.9999, 0.0001),
            ..Default::default()
        }
    }
}
