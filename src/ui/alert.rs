use gtk::prelude::{GtkApplicationExt, GtkWindowExt};
use relm4::{adw::prelude::MessageDialogExt, prelude::*};

fn alert_base(title: &str, msg: Option<&str>, parent: Option<&gtk::Window>) -> adw::MessageDialog {
    let d = adw::MessageDialog::builder()
        .modal(true)
        .heading(title)
        .build();
    if let Some(m) = msg {
        d.set_body(m);
    }
    if parent.is_some() {
        d.set_transient_for(parent);
    } else {
        d.set_transient_for(gtk::Application::default().active_window().as_ref());
    }
    d.add_response("ok", "_Ok");
    d
}

pub fn alert(title: &str, msg: Option<&str>, parent: Option<&gtk::Window>) {
    let d = alert_base(title, msg, parent);
    d.present();
}

pub fn alert_w_widget(
    title: &str,
    msg: Option<&str>,
    widget: Option<&gtk::Widget>,
    parent: Option<&gtk::Window>,
) {
    let d = alert_base(title, msg, parent);
    if let Some(w) = widget {
        d.set_extra_child(Some(w));
    }
    d.present();
}
