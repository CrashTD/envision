use gtk4::{gdk, gio, glib::clone, prelude::*};

pub fn limit_dropdown_width(dd: &gtk4::DropDown, chars: i32) {
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
        if let Ok(label) = dd_child.clone().unwrap().downcast::<gtk4::Label>() {
            label.set_max_width_chars(chars);
            label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        }
        let nc = dd_child.unwrap().first_child().clone();
        dd_child = nc;
    }
}

pub fn warning_heading() -> gtk4::Box {
    let b = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(12)
        .hexpand(true)
        .build();

    b.append(
        &gtk4::Image::builder()
            .css_classes(["warning"])
            .icon_name("dialog-warning-symbolic")
            .build(),
    );
    b.append(
        &gtk4::Label::builder()
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

pub fn copiable_code_snippet(code: &str) -> gtk4::Widget {
    let container = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(6)
        .build();
    let btn = gtk4::Button::builder()
        .css_classes(["flat", "circular"])
        .tooltip_text("Copy")
        .icon_name("edit-copy-symbolic")
        .vexpand(false)
        .hexpand(false)
        .valign(gtk4::Align::Center)
        .halign(gtk4::Align::Center)
        .build();
    btn.connect_clicked(clone!(
        #[to_owned]
        code,
        move |_| copy_text(&code)
    ));
    container.append(
        &gtk4::ScrolledWindow::builder()
            .vscrollbar_policy(gtk4::PolicyType::Never)
            .hscrollbar_policy(gtk4::PolicyType::Automatic)
            .css_classes(["card"])
            .overflow(gtk4::Overflow::Hidden)
            .child(
                &gtk4::TextView::builder()
                    .hexpand(true)
                    .vexpand(false)
                    .monospace(true)
                    .editable(false)
                    .left_margin(6)
                    .right_margin(6)
                    .top_margin(6)
                    .bottom_margin(18)
                    .buffer(
                        &gtk4::TextBuffer::builder()
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
