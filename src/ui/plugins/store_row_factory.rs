#![allow(unused_assignments)] // vars named in view! macro are not correctly recognized as used

use super::Plugin;
use crate::{downloader::cache_file, ui::SENDER_IO_ERR_MSG};
use gtk::prelude::*;
use relm4::{
    AsyncFactorySender, RelmWidgetExt, factory::AsyncFactoryComponent, prelude::DynamicIndex,
};
use tracing::error;

#[derive(Debug)]
#[tracker::track]
pub struct StoreRowModel {
    #[no_eq]
    pub plugin: Plugin,
    #[tracker::do_not_track]
    icon: Option<gtk::Image>,
    #[tracker::do_not_track]
    pub input_sender: relm4::Sender<StoreRowModelMsg>,
    pub enabled: bool,
    pub needs_update: bool,
}

#[derive(Debug)]
pub struct StoreRowModelInit {
    pub plugin: Plugin,
    pub enabled: bool,
    pub needs_update: bool,
}

#[derive(Debug)]
pub enum StoreRowModelMsg {
    LoadIcon,
    /// params: enabled, needs_update
    Refresh(bool, bool),
    SetEnabled(bool),
}

#[derive(Debug)]
pub enum StoreRowModelOutMsg {
    Install(Plugin),
    Remove(Plugin),
    SetEnabled(Plugin, bool),
}

#[relm4::factory(async pub)]
impl AsyncFactoryComponent for StoreRowModel {
    type Init = StoreRowModelInit;
    type Input = StoreRowModelMsg;
    type Output = StoreRowModelOutMsg;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        root = gtk::ListBoxRow {
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,
                set_vexpand: false,
                set_spacing: 12,
                set_margin_all: 12,
                #[name(icon)]
                gtk::Image {
                    set_icon_name: Some("application-x-addon-symbolic"),
                    set_icon_size: gtk::IconSize::Large,
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 6,
                    set_hexpand: true,
                    set_vexpand: true,
                    gtk::Label {
                        add_css_class: "title-3",
                        set_hexpand: true,
                        set_xalign: 0.0,
                        set_text: &self.plugin.name,
                        set_ellipsize: gtk::pango::EllipsizeMode::None,
                        set_wrap: true,
                    },
                    gtk::Label {
                        add_css_class: "dim-label",
                        set_hexpand: true,
                        set_xalign: 0.0,
                        set_text: self.plugin.short_description
                            .as_deref()
                            .unwrap_or(""),
                        set_ellipsize: gtk::pango::EllipsizeMode::None,
                        set_wrap: true,
                    },
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 6,
                    set_vexpand: true,
                    set_valign: gtk::Align::Center,
                    set_halign: gtk::Align::Center,
                    gtk::Button {
                        #[track = "self.changed(StoreRowModel::plugin())"]
                        set_visible: !self.plugin.is_installed(),
                        set_icon_name: "folder-download-symbolic",
                        add_css_class: "suggested-action",
                        set_tooltip_text: Some("Install"),
                        set_valign: gtk::Align::Center,
                        set_halign: gtk::Align::Center,
                        connect_clicked[sender, plugin] => move |_| {
                            sender
                                .output(Self::Output::Install(
                                    plugin.clone(),
                                ))
                                .expect(SENDER_IO_ERR_MSG);
                        }
                    },
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 6,
                        set_valign: gtk::Align::Center,
                        set_halign: gtk::Align::Center,
                        gtk::Button {
                            #[track = "self.changed(StoreRowModel::plugin())"]
                            set_visible: self.plugin.is_installed(),
                            set_icon_name: "user-trash-symbolic",
                            add_css_class: "destructive-action",
                            set_tooltip_text: Some("Remove"),
                            set_valign: gtk::Align::Center,
                            set_halign: gtk::Align::Center,
                            connect_clicked[sender, plugin] => move |_| {
                                sender
                                    .output(Self::Output::Remove(
                                        plugin.clone(),
                                    ))
                                    .expect(SENDER_IO_ERR_MSG);
                            }
                        },
                        gtk::Button {
                            #[track = "self.changed(StoreRowModel::plugin()) || self.changed(StoreRowModel::needs_update())"]
                            set_visible: self.plugin.is_installed() && self.needs_update,
                            set_icon_name: "view-refresh-symbolic",
                            add_css_class: "suggested-action",
                            set_tooltip_text: Some("Update"),
                            set_valign: gtk::Align::Center,
                            set_halign: gtk::Align::Center,
                            connect_clicked[sender, plugin] => move |_| {
                                sender
                                    .output(Self::Output::Install(
                                        plugin.clone(),
                                    ))
                                    .expect(SENDER_IO_ERR_MSG);
                            }
                        },
                    },
                    gtk::Switch {
                        #[track = "self.changed(StoreRowModel::plugin())"]
                        set_visible: self.plugin.is_installed(),
                        #[track = "self.changed(StoreRowModel::enabled())"]
                        set_active: self.enabled,
                        set_valign: gtk::Align::Center,
                        set_halign: gtk::Align::Center,
                        set_tooltip_text: Some("Plugin enabled"),
                        connect_state_set[sender] => move |_, state| {
                            sender.input(Self::Input::SetEnabled(state));
                            gtk::glib::Propagation::Proceed
                        }
                    },
                },
            }
        }
    }

    async fn update(&mut self, message: Self::Input, sender: AsyncFactorySender<Self>) {
        self.reset();

        match message {
            Self::Input::LoadIcon => {
                if let Some(url) = self.plugin.icon_url.as_ref() {
                    match cache_file(url, None).await {
                        Ok(dest) => {
                            self.icon.as_ref().unwrap().set_from_file(Some(dest));
                        }
                        Err(e) => {
                            error!("failed downloading icon '{url}': {e}");
                        }
                    };
                }
            }
            Self::Input::SetEnabled(state) => {
                self.set_enabled(state);
                sender
                    .output(Self::Output::SetEnabled(self.plugin.clone(), state))
                    .expect(SENDER_IO_ERR_MSG);
            }
            Self::Input::Refresh(enabled, needs_update) => {
                self.mark_all_changed();
                self.set_enabled(enabled);
                self.set_needs_update(needs_update);
            }
        }
    }

    async fn init_model(
        init: Self::Init,
        _index: &DynamicIndex,
        sender: AsyncFactorySender<Self>,
    ) -> Self {
        Self {
            tracker: 0,
            plugin: init.plugin,
            enabled: init.enabled,
            icon: None,
            input_sender: sender.input_sender().clone(),
            needs_update: init.needs_update,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: AsyncFactorySender<Self>,
    ) -> Self::Widgets {
        let plugin = self.plugin.clone(); // for use in a signal handler
        let widgets = view_output!();
        self.icon = Some(widgets.icon.clone());
        sender.input(Self::Input::LoadIcon);
        widgets
    }
}
