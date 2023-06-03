#[macro_use]
pub mod widgets {
    use iced::{
        widget::{column, row, Column},
        Length,
    };

    macro_rules! vspacer {
        () => {
            column![].height(Length::Fill)
        };
    }

    macro_rules! hspacer {
        () => {
            row![].width(Length::Fill)
        };
    }

    pub(crate) use vspacer;
    pub(crate) use hspacer;
}
