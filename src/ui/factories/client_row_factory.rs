#![allow(unused_assignments)] // vars named in view! macro are not correctly recognized as used

use crate::xr_clients::XRClient;
use adw::prelude::*;
use relm4::{AsyncFactorySender, factory::AsyncFactoryComponent, prelude::*};

#[derive(Debug)]
pub struct ClientRowModel {
    pub client: XRClient,
}

#[derive(Debug, Clone)]
pub struct ClientRowModelInit {
    pub client: XRClient,
}

#[relm4::factory(async pub)]
impl AsyncFactoryComponent for ClientRowModel {
    type Init = ClientRowModelInit;
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        root = adw::ActionRow {
            set_title: &self.client.name,
        }
    }

    async fn update(&mut self, _message: Self::Input, _sender: AsyncFactorySender<Self>) -> () {}

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        _sender: AsyncFactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();

        widgets
    }

    async fn init_model(
        init: Self::Init,
        _index: &DynamicIndex,
        _sender: AsyncFactorySender<Self>,
    ) -> Self {
        Self {
            client: init.client,
        }
    }
}
