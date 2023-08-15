use adw::prelude::*;
use gtk::{
    gio,
    glib::{self, GString},
};
use relm4::prelude::*;

use crate::withclones;

pub fn switch_row<F: Fn(&gtk::Switch, bool) -> glib::signal::Inhibit + 'static>(
    title: &str,
    description: Option<&str>,
    value: bool,
    cb: F,
) -> adw::ActionRow {
    let row = adw::ActionRow::builder()
        .title(title)
        .subtitle_lines(0)
        .build();

    if description.is_some() {
        row.set_subtitle(description.unwrap());
    }

    let switch = gtk::Switch::builder()
        .active(value)
        .valign(gtk::Align::Center)
        .build();
    row.add_suffix(&switch);
    row.set_activatable_widget(Some(&switch));

    switch.connect_state_set(cb);

    row
}

pub fn entry_row<F: Fn(&adw::EntryRow) + 'static>(
    title: &str,
    value: &str,
    cb: F,
) -> adw::EntryRow {
    let row = adw::EntryRow::builder().title(title).text(value).build();

    let clear_btn = gtk::Button::builder()
        .icon_name("edit-clear-symbolic")
        .tooltip_text(GString::from("Clear"))
        .css_classes(["circular", "flat"])
        .valign(gtk::Align::Center)
        .build();

    row.add_suffix(&clear_btn);

    {
        let row_c = row.clone();
        clear_btn.connect_clicked(move |_| {
            row_c.set_text("");
        });
    }

    row.connect_changed(cb);

    row
}

pub fn path_row<F: Fn(Option<String>) + 'static + Clone>(
    title: &str,
    description: Option<&str>,
    value: Option<String>,
    root_win: Option<gtk::Window>,
    cb: F,
) -> adw::ActionRow {
    let row = adw::ActionRow::builder()
        .title(title)
        .subtitle_lines(0)
        .activatable(true)
        .icon_name(GString::from("folder-open-symbolic"))
        .build();

    if description.is_some() {
        row.set_subtitle(description.unwrap());
    }

    let path_label = &gtk::Label::builder()
        .label(match value.as_ref() {
            None => "(None)",
            Some(p) => p.as_str(),
        })
        .build();
    row.add_suffix(path_label);

    let clear_btn = gtk::Button::builder()
        .icon_name("edit-clear-symbolic")
        .tooltip_text(GString::from("Clear Path"))
        .css_classes(["circular", "flat"])
        .valign(gtk::Align::Center)
        .build();
    row.add_suffix(&clear_btn);
    {
        withclones![path_label, cb];
        clear_btn.connect_clicked(move |_| {
            path_label.set_label("(None)");
            cb(None)
        });
    }
    let filedialog = gtk::FileDialog::builder()
        .modal(true)
        .title(format!("Select Path for {}", title))
        .build();

    {
        withclones![path_label];
        row.connect_activated(move |_| {
            withclones![path_label, cb];
            filedialog.select_folder(
                root_win.as_ref(),
                gio::Cancellable::NONE,
                move |res| match res {
                    Ok(file) => {
                        let path = file.path();
                        if path.is_some() {
                            let path_s = path.unwrap().to_str().unwrap().to_string();
                            path_label.set_text(path_s.as_str());
                            cb(Some(path_s))
                        }
                    }
                    _ => {}
                },
            )
        });
    }

    row
}

pub fn combo_row<F: Fn(&adw::ComboRow) + 'static>(
    title: &str,
    description: Option<&str>,
    value: &str,
    values: Vec<String>,
    cb: F,
) -> adw::ComboRow {
    let row = adw::ComboRow::builder()
        .title(title)
        .subtitle_lines(0)
        .model(&gtk::StringList::new(
            values
                .iter()
                .map(String::as_str)
                .collect::<Vec<&str>>()
                .as_slice(),
        ))
        .build();

    if description.is_some() {
        row.set_subtitle(description.unwrap());
    }

    let selected = values.iter().position(|v| *v == value.to_string());
    row.set_selected(match selected {
        Some(i) => i,
        None => 0,
    } as u32);

    row.connect_selected_item_notify(cb);

    row
}
