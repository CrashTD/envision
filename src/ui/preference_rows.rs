use adw::prelude::*;
use gtk::{
    gio,
    glib::{self, GString, clone},
};
use relm4::prelude::*;

pub fn switch_row<F: Fn(&adw::SwitchRow, bool) + 'static>(
    title: &str,
    description: Option<&str>,
    value: bool,
    cb: F,
) -> adw::SwitchRow {
    let row = adw::SwitchRow::builder()
        .title(title)
        .subtitle_lines(0)
        .active(value)
        .build();

    if let Some(d) = description {
        row.set_subtitle(d);
    }

    row.connect_active_notify(move |row| {
        cb(row, row.is_active());
    });

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

pub fn spin_row<F: Fn(&gtk::Adjustment) + 'static>(
    title: &str,
    subtitle: Option<&str>,
    val: f64,
    min: f64,
    max: f64,
    min_increment: f64,
    cb: F,
) -> adw::SpinRow {
    let adj = gtk::Adjustment::builder()
        .lower(min)
        .upper(max)
        .step_increment(min_increment)
        .page_increment(min_increment)
        .value(val)
        .build();
    let row = adw::SpinRow::builder()
        .title(title)
        .adjustment(&adj)
        .digits(if min_increment == 1.0 {
            0
        } else {
            min_increment.to_string().len() - 2
        } as u32)
        .build();
    if let Some(sub) = subtitle {
        row.set_subtitle(sub);
    }

    adj.connect_value_changed(cb);

    row
}

fn filedialog_row_base<F: Fn(Option<String>) + 'static + Clone>(
    title: &str,
    description: Option<&str>,
    value: Option<String>,
    cb: F,
) -> (adw::ActionRow, gtk::Label) {
    let row = adw::ActionRow::builder()
        .title(title)
        .subtitle_lines(0)
        .activatable(true)
        .build();
    row.add_prefix(&gtk::Image::from_icon_name("folder-open-symbolic"));

    if let Some(d) = description {
        row.set_subtitle(d);
    }

    let path_label = gtk::Label::builder()
        .label(match value.as_ref() {
            None => "(None)",
            Some(p) => p.as_str(),
        })
        .wrap(true)
        .build();
    row.add_suffix(&path_label);

    let clear_btn = gtk::Button::builder()
        .icon_name("edit-clear-symbolic")
        .tooltip_text(GString::from("Clear Path"))
        .css_classes(["circular", "flat"])
        .valign(gtk::Align::Center)
        .build();
    row.add_suffix(&clear_btn);
    clear_btn.connect_clicked(clone!(
        #[weak]
        path_label,
        #[strong]
        cb,
        move |_| {
            path_label.set_label("(None)");
            cb(None)
        }
    ));
    (row, path_label)
}

// pub fn file_row<F: Fn(Option<String>) + 'static + Clone>(
//     title: &str,
//     description: Option<&str>,
//     value: Option<String>,
//     root_win: Option<gtk::Window>,
//     cb: F,
// ) -> adw::ActionRow {
//     let (row, path_label) = filedialog_row_base(title, description, value, cb.clone());
//
//     let filedialog = gtk::FileDialog::builder()
//         .modal(true)
//         .title(format!("Select {title}"))
//         .build();
//
//     row.connect_activated(clone!(
//         #[weak]
//         path_label,
//         move |_| {
//             filedialog.open(
//                 root_win.as_ref(),
//                 gio::Cancellable::NONE,
//                 clone!(
//                     #[weak]
//                     path_label,
//                     #[strong]
//                     cb,
//                     move |res| {
//                         if let Ok(file) = res
//                             && let Some(path) = file.path()
//                         {
//                             let path_s = path.to_string_lossy().to_string();
//                             path_label.set_text(&path_s);
//                             cb(Some(path_s))
//                         }
//                     }
//                 ),
//             )
//         }
//     ));
//
//     row
// }

pub fn path_row<F: Fn(Option<String>) + 'static + Clone>(
    title: &str,
    description: Option<&str>,
    value: Option<String>,
    root_win: Option<gtk::Window>,
    cb: F,
) -> adw::ActionRow {
    let (row, path_label) = filedialog_row_base(title, description, value, cb.clone());
    let filedialog = gtk::FileDialog::builder()
        .modal(true)
        .title(format!("Select Path for {title}"))
        .build();

    row.connect_activated(clone!(
        #[weak]
        path_label,
        move |_| {
            filedialog.select_folder(
                root_win.as_ref(),
                gio::Cancellable::NONE,
                clone!(
                    #[weak]
                    path_label,
                    #[strong]
                    cb,
                    move |res| {
                        if let Ok(file) = res
                            && let Some(path) = file.path()
                        {
                            let path_s = path.to_string_lossy().to_string();
                            path_label.set_text(&path_s);
                            cb(Some(path_s))
                        }
                    }
                ),
            )
        }
    ));

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

    if let Some(desc) = description {
        row.set_subtitle(desc);
    }

    let selected = values.iter().position(|v| v == value);
    row.set_selected(selected.unwrap_or(0) as u32);

    row.connect_selected_item_notify(cb);

    row
}
