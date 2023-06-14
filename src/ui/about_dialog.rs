use relm4::gtk::traits::GtkWindowExt;
use relm4::prelude::*;
use relm4::{ComponentParts, SimpleComponent};

use crate::constants::{APP_NAME, REPO_URL, SINGLE_DEVELOPER, VERSION, get_developers};

pub struct AboutDialog {}

impl SimpleComponent for AboutDialog {
    type Init = ();
    type Input = ();
    type Output = ();
    type Root = adw::AboutWindow;
    type Widgets = adw::AboutWindow;

    fn init_root() -> Self::Root {
        adw::AboutWindow::builder()
            .application_name(APP_NAME)
            .license_type(gtk::License::Agpl30)
            .version(VERSION)
            .website(REPO_URL)
            .developer_name(SINGLE_DEVELOPER)
            .developers(get_developers())
            .modal(true)
            .hide_on_close(true)
            .build()
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = AboutDialog {};
        let widgets = root.clone();
        ComponentParts { model, widgets }
    }

    fn update_view(&self, dialog: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        dialog.present();
    }
}
