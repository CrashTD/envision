use gtk::{gdk, gio, glib::clone, prelude::*};

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

pub fn warning_heading() -> gtk::Box {
    let b = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .hexpand(true)
        .build();

    b.append(
        &gtk::Image::builder()
            .css_classes(["warning"])
            .icon_name("dialog-warning-symbolic")
            .build(),
    );
    b.append(
        &gtk::Label::builder()
            .css_classes(["warning", "heading"])
            .label("Warning")
            .build(),
    );

    b
}

pub fn open_with_default_handler(uri: &str) {
    if let Err(e) = gio::AppInfo::launch_default_for_uri(uri, gio::AppLaunchContext::NONE) {
        eprintln!("Error opening uri {}: {}", uri, e)
    };
}

pub fn copy_text(txt: &str) {
    match gdk::Display::default() {
        None => {
            eprintln!("Warning: could not get default gdk display")
        }
        Some(d) => {
            d.clipboard().set_text(txt);
        }
    }
}

pub fn bits_to_mbits(bits: u32) -> Option<u32> {
    bits.checked_div(1000000)
}

pub fn bits_from_mbits(mbits: u32) -> Option<u32> {
    mbits.checked_mul(1000000)
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
