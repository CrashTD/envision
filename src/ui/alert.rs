use gtk::traits::GtkWindowExt;
use relm4::{adw::traits::MessageDialogExt, prelude::*};

pub fn alert(title: &str, msg: Option<&str>, parent: Option<&gtk::Window>) {
    let d = adw::MessageDialog::builder()
        .modal(true)
        .heading(title)
        .build();
    if msg.is_some() {
        d.set_body(msg.unwrap());
    }
    if parent.is_some() {
        d.set_transient_for(parent);
    }
    d.add_response("ok", "_Ok");
    d.present();
}
