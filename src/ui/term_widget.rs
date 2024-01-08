use gtk4::gdk;
use relm4::adw;
use zoha_vte4::{Terminal, TerminalExt};

const MAX_SCROLLBACK: u32 = 2000;

#[derive(Debug, Clone)]
pub struct TermWidget {
    pub container: gtk4::ScrolledWindow,
    pub term: Terminal,
}

impl TermWidget {
    pub fn new() -> Self {
        let term = Terminal::builder()
            .scroll_on_output(true)
            .scrollback_lines(MAX_SCROLLBACK)
            .scroll_unit_is_pixels(true)
            .vexpand(true)
            .hexpand(true)
            .build();
        term.set_clear_background(false);
        term.search_set_wrap_around(true);
        let container = gtk4::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .child(&term)
            .build();
        let this = Self { container, term };
        this
    }

    pub fn set_color_scheme(&self) {
        // TODO: use theme colors
        if adw::StyleManager::default().is_dark() {
            self.term
                .set_color_foreground(&gdk::RGBA::new(1.0, 1.0, 1.0, 1.0));
        } else {
            self.term
                .set_color_foreground(&gdk::RGBA::new(0.0, 0.0, 0.0, 1.0));
        }
    }

    pub fn feed(&self, txt: &str) {
        self.term.feed(txt.replace('\n', "\r\n").as_bytes())
    }

    pub fn clear(&self) {
        self.term.feed("\x1bc".as_bytes());
    }

    pub fn set_search_term(&self, term: Option<&str>) {
        self.term.search_set_regex(
            term.map(|txt| zoha_vte4::Regex::for_search(txt, 0).ok())
                .flatten()
                .as_ref(),
            0,
        );
    }

    pub fn search_next(&self) {
        self.term.search_find_next();
    }

    pub fn search_prev(&self) {
        self.term.search_find_previous();
    }
}
