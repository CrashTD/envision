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
        match dd_child.clone().unwrap().downcast::<gtk4::Label>() {
            Ok(label) => {
                label.set_max_width_chars(chars);
                label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            }
            _ => {}
        }
        let nc = dd_child.unwrap().first_child().clone();
        dd_child = nc;
    }
}
