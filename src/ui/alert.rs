use gtk::prelude::GtkApplicationExt;
use relm4::{adw::prelude::*, prelude::*};

fn alert_base(title: &str, msg: Option<&str>) -> adw::AlertDialog {
    let d = adw::AlertDialog::builder().heading(title).build();
    if let Some(m) = msg {
        d.set_body(m);
    }
    d.add_response("ok", "_Ok");
    d
}

fn present_alert(d: adw::AlertDialog, parent: Option<&gtk::Window>) {
    if parent.is_some() {
        d.present(parent);
    } else {
        let active_win = gtk::Application::default().active_window();
        d.present(active_win.as_ref());
    };
}

pub fn alert(title: &str, msg: Option<&str>, parent: Option<&gtk::Window>) {
    let d = alert_base(title, msg);
    present_alert(d, parent);
}

pub fn alert_w_widget(
    title: &str,
    msg: Option<&str>,
    widget: Option<&gtk::Widget>,
    parent: Option<&gtk::Window>,
) {
    let d = alert_base(title, msg);
    if let Some(w) = widget {
        d.set_extra_child(Some(w));
    }
    present_alert(d, parent);
}
