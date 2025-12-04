const ANSI_ESCAPE_BEGIN: &str = "\u{001B}[";
const ANSI_ESCAPE_END: &str = "m";

const ANSI_RESET: u8 = 0;

const ANSI_BLACK: u8 = 30;
const ANSI_RED: u8 = 31;
const ANSI_GREEN: u8 = 32;
const ANSI_YELLOW: u8 = 33;
const ANSI_BLUE: u8 = 34;
const ANSI_PURPLE: u8 = 35;
const ANSI_CYAN: u8 = 36;
const ANSI_WHITE: u8 = 37;

// const ANSI_BG_ADDENDUM: u8 = 10;
const ANSI_BRIGHT_ADDENDUM: u8 = 60;

#[derive(Debug, Clone, Copy)]
pub enum TermColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Purple,
    Cyan,
    White,
    Gray,
}

impl TermColor {
    fn code(&self) -> u8 {
        match self {
            Self::Black => ANSI_BLACK,
            Self::Red => ANSI_RED,
            Self::Green => ANSI_GREEN,
            Self::Yellow => ANSI_YELLOW,
            Self::Blue => ANSI_BLUE,
            Self::Purple => ANSI_PURPLE,
            Self::Cyan => ANSI_CYAN,
            Self::White => ANSI_WHITE,
            Self::Gray => ANSI_BLACK + ANSI_BRIGHT_ADDENDUM,
        }
    }

    pub fn colorize(&self, s: &str) -> String {
        format!(
            "{ANSI_ESCAPE_BEGIN}{code}{ANSI_ESCAPE_END}{s}{ANSI_ESCAPE_BEGIN}{ANSI_RESET}{ANSI_ESCAPE_END}",
            code = self.code()
        )
    }
}
