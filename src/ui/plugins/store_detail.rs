#![allow(unused_assignments)] // vars named in view! macro are not correctly recognized as used

use super::Plugin;
use crate::{downloader::cache_file, ui::SENDER_IO_ERR_MSG};
use adw::prelude::*;
use relm4::prelude::*;
use tracing::{error, warn};

#[tracker::track]
pub struct StoreDetail {
    plugin: Option<Plugin>,
    enabled: bool,
    #[tracker::do_not_track]
    carousel: Option<adw::Carousel>,
    #[tracker::do_not_track]
    icon: Option<gtk::Image>,
    needs_update: bool,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum StoreDetailMsg {
    SetPlugin(Plugin, bool, bool),
    SetIcon,
    SetScreenshots,
    Refresh(String, bool, bool),
    Install,
    Remove,
    SetEnabled(bool),
    OpenHomepage,
}

#[derive(Debug)]
pub enum StoreDetailOutMsg {
    Install(Plugin),
    Remove(Plugin),
    GoBack,
    SetEnabled(Plugin, bool),
}

#[relm4::component(pub async)]
impl AsyncComponent for StoreDetail {
    type Init = ();
    type Input = StoreDetailMsg;
    type Output = StoreDetailOutMsg;
    type CommandOutput = ();

    view! {
        adw::ToolbarView {
            set_top_bar_style: adw::ToolbarStyle::Flat,
            add_top_bar: headerbar = &adw::HeaderBar {
                #[wrap(Some)]
                set_title_widget: title = &adw::WindowTitle {
                    #[track = "model.changed(Self::plugin())"]
                    set_title: model
                        .plugin
                        .as_ref()
                        .map(|p| p.name.as_str())
                        .unwrap_or_default(),
                },
                pack_start: backbtn = &gtk::Button {
                    set_icon_name: "go-previous-symbolic",
                    set_tooltip_text: Some("Back"),
                    connect_clicked[sender] => move |_| {
                        sender.output(Self::Output::GoBack).expect(SENDER_IO_ERR_MSG);
                    }
                },
            },
            #[wrap(Some)]
            set_content: inner = &gtk::ScrolledWindow {
                set_hscrollbar_policy: gtk::PolicyType::Never,
                set_hexpand: true,
                set_vexpand: true,
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_hexpand: true,
                    set_vexpand: true,
                    set_margin_top: 12,
                    set_margin_bottom: 48,
                    set_margin_start: 12,
                    set_margin_end: 12,
                    set_spacing: 24,
                    adw::Clamp { // icon, name, buttons
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_hexpand: true,
                            set_vexpand: false,
                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_hexpand: true,
                                #[name(icon)]
                                gtk::Image {
                                    set_icon_name: Some("application-x-addon-symbolic"),
                                    set_margin_end: 12,
                                    set_pixel_size: 96,
                                },
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_hexpand: true,
                                    set_valign: gtk::Align::Center,
                                    set_spacing: 6,
                                    gtk::Label {
                                        add_css_class: "title-2",
                                        set_xalign: 0.0,
                                        #[track = "model.changed(Self::plugin())"]
                                        set_text: model
                                            .plugin
                                            .as_ref()
                                            .map(|p| p.name.as_str())
                                            .unwrap_or_default(),
                                            set_ellipsize: gtk::pango::EllipsizeMode::None,
                                            set_wrap: true,
                                    },
                                    gtk::Label {
                                        add_css_class: "dim-label",
                                        set_xalign: 0.0,
                                        #[track = "model.changed(Self::plugin())"]
                                        set_visible: model
                                            .plugin
                                            .as_ref()
                                            .is_some_and(|p| p.author.is_some()),
                                        #[track = "model.changed(Self::plugin())"]
                                        set_text: model
                                            .plugin
                                            .as_ref()
                                            .and_then(
                                                |p| p.author.as_deref()
                                            )
                                            .unwrap_or_default(),
                                        set_ellipsize: gtk::pango::EllipsizeMode::None,
                                        set_wrap: true,
                                    },
                                },
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_halign: gtk::Align::Center,
                                    set_valign: gtk::Align::Center,
                                    set_spacing: 6,
                                    gtk::Button {
                                        #[track = "model.changed(Self::plugin())"]
                                        set_visible: !model
                                            .plugin
                                            .as_ref()
                                            .is_some_and(|p| p.is_installed()),
                                        set_label: "Install",
                                        add_css_class: "suggested-action",
                                        connect_clicked[sender] => move |_| {
                                            sender.input(Self::Input::Install);
                                        }
                                    },
                                    gtk::Button {
                                        #[track = "model.changed(Self::plugin())"]
                                        set_visible: model
                                            .plugin
                                            .as_ref()
                                            .is_some_and(|p| p.is_installed()),
                                        set_label: "Remove",
                                        add_css_class: "destructive-action",
                                        connect_clicked[sender] => move |_| {
                                            sender.input(Self::Input::Remove);
                                        }
                                    },
                                    gtk::Button {
                                        #[track = "model.changed(Self::plugin()) || model.changed(Self::needs_update())"]
                                        set_visible: model
                                            .plugin
                                            .as_ref()
                                            .is_some_and(|p| p.is_installed()) && model.needs_update,
                                        add_css_class: "suggested-action",
                                        set_label: "Update",
                                        set_valign: gtk::Align::Center,
                                        set_halign: gtk::Align::Center,
                                        connect_clicked[sender] => move |_| {
                                            sender.input(Self::Input::Install);
                                        }
                                    },
                                    gtk::Switch {
                                        #[track = "model.changed(Self::plugin())"]
                                        set_visible: model.plugin.as_ref()
                                            .is_some_and(|p| p.is_installed()),
                                        #[track = "model.changed(Self::enabled())"]
                                        set_active: model.enabled,
                                        set_tooltip_text: Some("Plugin enabled"),
                                        set_valign: gtk::Align::Center,
                                        set_halign: gtk::Align::Center,
                                        connect_state_set[sender] => move |_, state| {
                                            sender.input(Self::Input::SetEnabled(state));
                                            gtk::glib::Propagation::Proceed
                                        }
                                    },
                                    gtk::Button {
                                        #[track = "model.changed(Self::plugin())"]
                                        set_visible: model.plugin.as_ref()
                                            .is_some_and(|p| p.homepage_url.is_some()),
                                        set_label: "Homepage",
                                        connect_clicked[sender] => move |_| {
                                            sender.input(Self::Input::OpenHomepage);
                                        }
                                    }
                                }
                            }
                        },
                    },
                    gtk::Box { // screenshots
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 12,
                        #[name(carousel)]
                        adw::Carousel {
                            set_allow_mouse_drag: true,
                            set_allow_scroll_wheel: false,
                            set_spacing: 24,
                        },
                        adw::CarouselIndicatorDots {
                            set_carousel: Some(&carousel),
                        },
                    },
                    adw::Clamp { // description
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            gtk::Label {
                                set_xalign: 0.0,
                                #[track = "model.changed(Self::plugin())"]
                                set_text: model
                                    .plugin
                                    .as_ref()
                                    .and_then(|p| p
                                        .description
                                        .as_deref()
                                    ).unwrap_or(""),
                                set_ellipsize: gtk::pango::EllipsizeMode::None,
                                set_wrap: true,
                                set_justify: gtk::Justification::Fill,
                            },
                        },
                    },
                }
            }
        }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        self.reset();

        match message {
            Self::Input::OpenHomepage => {
                if let Some(plugin) = self.plugin.as_ref()
                    && let Some(homepage) = plugin.homepage_url.as_ref()
                    && let Err(e) = gtk::gio::AppInfo::launch_default_for_uri(
                        homepage,
                        gtk::gio::AppLaunchContext::NONE,
                    )
                {
                    error!("opening uri {homepage}: {e}");
                }
            }
            Self::Input::SetPlugin(p, enabled, needs_update) => {
                self.set_plugin(Some(p));
                self.set_enabled(enabled);
                self.set_needs_update(needs_update);
                sender.input(Self::Input::SetIcon);
                sender.input(Self::Input::SetScreenshots);
            }
            Self::Input::SetIcon => {
                if let Some(plugin) = self.plugin.as_ref() {
                    if let Some(url) = plugin.icon_url.as_ref() {
                        match cache_file(url, None).await {
                            Ok(dest) => {
                                self.icon.as_ref().unwrap().set_from_file(Some(dest));
                            }
                            Err(e) => {
                                warn!("Failed downloading icon '{url}': {e}");
                            }
                        };
                    } else {
                        self.icon
                            .as_ref()
                            .unwrap()
                            .set_icon_name(Some("application-x-addon-symbolic"));
                    }
                }
            }
            Self::Input::SetScreenshots => {
                let carousel = self.carousel.as_ref().unwrap().clone();
                while let Some(child) = carousel.first_child() {
                    carousel.remove(&child);
                }
                if let Some(plugin) = self.plugin.as_ref() {
                    carousel
                        .parent()
                        .unwrap()
                        .set_visible(!plugin.screenshots.is_empty());
                    for url in plugin.screenshots.iter() {
                        match cache_file(url, None).await {
                            Ok(dest) => {
                                let pic = gtk::Picture::builder()
                                    .height_request(300)
                                    .css_classes(["card"])
                                    .overflow(gtk::Overflow::Hidden)
                                    .valign(gtk::Align::Center)
                                    .build();
                                pic.set_filename(Some(dest));
                                let clamp = adw::Clamp::builder().child(&pic).build();
                                carousel.append(&clamp);
                            }
                            Err(e) => {
                                warn!("failed downloading screenshot '{url}': {e}");
                            }
                        };
                    }
                }
            }
            Self::Input::Refresh(appid, enabled, needs_update) => {
                if self.plugin.as_ref().is_some_and(|p| p.appid == appid) {
                    self.mark_all_changed();
                    self.set_enabled(enabled);
                    self.set_needs_update(needs_update);
                }
            }
            Self::Input::Install => {
                if let Some(plugin) = self.plugin.as_ref() {
                    sender
                        .output(Self::Output::Install(plugin.clone()))
                        .expect(SENDER_IO_ERR_MSG);
                }
            }
            Self::Input::Remove => {
                if let Some(plugin) = self.plugin.as_ref() {
                    sender
                        .output(Self::Output::Remove(plugin.clone()))
                        .expect(SENDER_IO_ERR_MSG);
                    if plugin.exec_url.is_none() {
                        sender
                            .output(Self::Output::GoBack)
                            .expect(SENDER_IO_ERR_MSG);
                    }
                }
            }
            Self::Input::SetEnabled(enabled) => {
                self.set_enabled(enabled);
                if let Some(plugin) = self.plugin.as_ref() {
                    sender
                        .output(Self::Output::SetEnabled(plugin.clone(), enabled))
                        .expect(SENDER_IO_ERR_MSG);
                }
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let mut model = Self {
            tracker: 0,
            plugin: None,
            enabled: false,
            carousel: None,
            icon: None,
            needs_update: false,
        };
        let widgets = view_output!();

        model.carousel = Some(widgets.carousel.clone());
        model.icon = Some(widgets.icon.clone());

        AsyncComponentParts { model, widgets }
    }
}
