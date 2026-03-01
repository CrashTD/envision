use gtk::{gdk, gio, glib::clone, prelude::*};
use tracing::{error, warn};

pub fn limit_dropdown_width(dd: &gtk::DropDown) {
    let mut dd_child = dd
        .first_child()
        .unwrap()
        .first_child()
        .unwrap()
        .first_child()
        .unwrap()
        .last_child();
    loop {
        if dd_child.is_none() {
            break;
        }
        if let Ok(label) = dd_child.clone().unwrap().downcast::<gtk::Label>() {
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        }
        let nc = dd_child.unwrap().first_child().clone();
        dd_child = nc;
    }
}

pub fn open_with_default_handler(uri: &str) {
    if let Err(e) = gio::AppInfo::launch_default_for_uri(uri, gio::AppLaunchContext::NONE) {
        error!("opening uri {uri}: {e}")
    };
}

pub fn copy_text(txt: &str) {
    match gdk::Display::default() {
        None => {
            warn!("could not get default gdk display")
        }
        Some(d) => {
            d.clipboard().set_text(txt);
        }
    }
}

pub fn copiable_code_snippet(code: &str) -> gtk::Widget {
    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(6)
        .build();
    let btn = gtk::Button::builder()
        .css_classes(["flat", "circular"])
        .tooltip_text("Copy")
        .icon_name("edit-copy-symbolic")
        .vexpand(false)
        .hexpand(false)
        .valign(gtk::Align::Center)
        .halign(gtk::Align::Center)
        .build();
    btn.connect_clicked(clone!(
        #[to_owned]
        code,
        move |_| copy_text(&code)
    ));
    container.append(
        &gtk::ScrolledWindow::builder()
            .vscrollbar_policy(gtk::PolicyType::Never)
            .hscrollbar_policy(gtk::PolicyType::Automatic)
            .css_classes(["card"])
            .overflow(gtk::Overflow::Hidden)
            .child(
                &gtk::TextView::builder()
                    .hexpand(true)
                    .vexpand(false)
                    .monospace(true)
                    .editable(false)
                    .left_margin(6)
                    .right_margin(6)
                    .top_margin(6)
                    .bottom_margin(18)
                    .buffer(
                        &gtk::TextBuffer::builder()
                            .text(code)
                            .enable_undo(false)
                            .build(),
                    )
                    .build(),
            )
            .build(),
    );
    container.append(&btn);
    container.upcast()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InfoCardLevel {
    #[default]
    Info,
    Warn,
    // Err,
}

impl InfoCardLevel {
    fn css_class(&self) -> Option<&'static str> {
        match self {
            Self::Info => None,
            Self::Warn => Some("warning"),
            // Self::Err => Some("error"),
        }
    }

    fn image(&self) -> gtk::Image {
        let i = gtk::Image::builder()
            .icon_name(match self {
                Self::Info => "dialog-information-symbolic",
                Self::Warn => "dialog-warning-symbolic",
                // Self::Err => "dialog-error-symbolic",
            })
            .build();
        if let Some(cls) = self.css_class() {
            i.add_css_class(cls);
        }
        i
    }

    fn default_label(&self) -> &'static str {
        match self {
            InfoCardLevel::Info => "Info",
            InfoCardLevel::Warn => "Warning",
            // InfoCardLevel::Err => "Error",
        }
    }

    pub fn heading(&self, title: Option<&str>) -> gtk::Box {
        let b = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .hexpand(true)
            .build();

        b.append(&self.image());
        let title = title.unwrap_or_else(|| self.default_label());
        let label = gtk::Label::builder()
            .label(title)
            .css_classes(["heading"])
            .build();
        if let Some(cls) = self.css_class() {
            label.add_css_class(cls);
        }
        b.append(&label);

        b
    }
}

pub fn info_card(level: InfoCardLevel, title: Option<&str>, text: &str) -> gtk::Box {
    let b = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .hexpand(true)
        .vexpand(false)
        .spacing(12)
        .css_classes(["padded", "card"])
        .build();

    b.append(&level.heading(title));

    let text_label = gtk::Label::builder()
        .label(text)
        .use_markup(true)
        .xalign(0.0)
        .wrap(true)
        .wrap_mode(gtk::pango::WrapMode::Word)
        .build();

    if let Some(cls) = level.css_class() {
        text_label.add_css_class(cls);
    }

    b.append(&text_label);

    b
}

pub fn warn_card(title: Option<&str>, text: &str) -> gtk::Box {
    info_card(InfoCardLevel::Warn, title, text)
}
