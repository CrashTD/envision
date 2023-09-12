use gtk4::prelude::*;

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
