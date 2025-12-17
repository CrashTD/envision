#![allow(unused_assignments)] // vars named in view! macro are not correctly recognized as used

use crate::{
    config::Config,
    plugins::XRPlugin,
    profile::{OvrCompatibilityModuleType, XRServiceType},
    ui::{
        SENDER_IO_ERR_MSG,
        preference_rows::{combo_row, spin_row, switch_row},
    },
};
use adw::prelude::*;
use gtk::glib::clone;
use relm4::prelude::*;
use std::{cell::RefCell, path::PathBuf, rc::Rc};

pub struct PreferencesWindow {
    config: Rc<RefCell<Config>>,
    win: Option<adw::PreferencesDialog>,
    parent: gtk::Window,
    plugins_grp: Option<adw::PreferencesGroup>,
}

#[derive(Debug)]
pub enum PreferencesWindowMsg {
    Present,
    OnClosed,
    PopulatePlugins,
}

#[derive(Debug)]
pub enum PreferencesWindowOutMsg {
    OnClosed(Config),
}

pub struct PreferencesWindowInit {
    pub root_win: gtk::Window,
    pub config: Config,
}

#[relm4::component(pub async)]
impl AsyncComponent for PreferencesWindow {
    type Init = PreferencesWindowInit;
    type Input = PreferencesWindowMsg;
    type Output = PreferencesWindowOutMsg;
    type CommandOutput = ();

    view! {
        #[name(win)]
        adw::PreferencesDialog {
            set_search_enabled: true,
            connect_closed[sender] => move |_| {
                sender.input(Self::Input::OnClosed);
            },
            add: general_page = &adw::PreferencesPage {
                set_title: "General",
                set_icon_name: Some("preferences-system-symbolic"),
                add: general_grp = &adw::PreferencesGroup {
                    add: &combo_row(
                        "XR service",
                        Some("Choose Monado for PCVR and WiVRn for standalone"),
                        model.config.borrow().global_preferences.xrservice_type.to_string().as_str(),
                        XRServiceType::iter().map(XRServiceType::to_string).collect(),
                        clone!(#[strong(rename_to=config)] model.config, move |row| {
                            config.borrow_mut().global_preferences.xrservice_type = XRServiceType::from(row.selected());
                        })
                    ),
                    add: &combo_row(
                        "OpenVR compatibility",
                        None,
                        model.config.borrow().global_preferences.ovr_comp.to_string().as_str(),
                        OvrCompatibilityModuleType::iter().map(OvrCompatibilityModuleType::to_string).collect(),
                        clone!(#[strong(rename_to=config)] model.config, move |row| {
                            config.borrow_mut().global_preferences.ovr_comp = OvrCompatibilityModuleType::from(row.selected());
                        })
                    ),
                    add: &spin_row(
                        "Render scale percentage",
                        None,
                        model.config.borrow().global_preferences.scale_percentage.into(),
                        20.0,
                        500.0,
                        1.0,
                        clone!(#[strong(rename_to=config)] model.config, move |adj| {
                            let raw = adj.value().trunc();
                            // boring f64 re-clamping for sake of correctness
                            let val = if raw >= 500.0 {
                                500
                            } else if raw <= 20.0 {
                                20
                            } else {
                                raw as u16
                            };
                            config.borrow_mut().global_preferences.scale_percentage = val;
                        })
                    ),
                    add: &switch_row(
                        "Compositor GPU acceleration",
                        None,
                        model.config.borrow().global_preferences.use_compositor_gpu_compute,
                        clone!(#[strong(rename_to=config)] model.config, move |_, v| {
                            config.borrow_mut().global_preferences.use_compositor_gpu_compute = v;
                        })
                    ),
                    add: &switch_row(
                        "Use minimum frame period",
                        Some("Disables the refresh rate limit; this can increase performance"),
                        model.config.borrow().global_preferences.use_min_frame_period,
                        clone!(#[strong(rename_to=config)] model.config, move |_, v| {
                            config.borrow_mut().global_preferences.use_min_frame_period = v;
                        })
                    ),
                    add: &switch_row(
                        "Monado UI",
                        None,
                        model.config.borrow().global_preferences.monado_ui,
                        clone!(#[strong(rename_to=config)] model.config, move |_, v| {
                            config.borrow_mut().global_preferences.monado_ui = v;
                        })
                    ),
                },
            },
            add: plugins_page = &adw::PreferencesPage {
                set_title: "Plugins",
                set_icon_name: Some("applications-utilities-symbolic"),
                add: plugins_grp = &adw::PreferencesGroup {
                },
            },
        }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            Self::Input::Present => {
                if let Some(win) = self.win.as_ref() {
                    win.present(Some(&self.parent));
                    sender.input(Self::Input::PopulatePlugins);
                }
            }
            Self::Input::PopulatePlugins => {
                if let Some(grp) = self.plugins_grp.as_ref() {
                    for plugin in XRPlugin::find_plugins() {
                        let row = switch_row(
                            &plugin.name,
                            plugin.description.as_deref(),
                            self.config.borrow().xr_plugins.contains(&plugin.path),
                            clone!(
                                #[strong(rename_to=config)]
                                self.config,
                                move |_, v| {
                                    if v {
                                        config.borrow_mut().xr_plugins.insert(plugin.path.clone());
                                    } else {
                                        config
                                            .borrow_mut()
                                            .xr_plugins
                                            .retain(|p| p != &plugin.path);
                                    }
                                }
                            ),
                        );
                        let img = if let Some(icon) = plugin.icon.as_deref() {
                            if let Ok(p) = PathBuf::from(icon).canonicalize()
                                && p.is_file()
                            {
                                gtk::Image::from_file(p.to_string_lossy().to_string())
                            } else {
                                gtk::Image::from_icon_name(icon)
                            }
                        } else {
                            gtk::Image::from_icon_name("application-x-executable-symbolic")
                        };
                        img.set_icon_size(gtk::IconSize::Large);
                        row.add_prefix(&img);
                        grp.add(&row);
                    }
                }
            }
            Self::Input::OnClosed => {
                sender
                    .output(Self::Output::OnClosed(self.config.borrow().clone()))
                    .expect(SENDER_IO_ERR_MSG);
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let mut model = Self {
            win: None,
            config: Rc::new(RefCell::new(init.config)),
            parent: init.root_win,
            plugins_grp: None,
        };

        let widgets = view_output!();

        model.win = Some(widgets.win.clone());
        model.plugins_grp = Some(widgets.plugins_grp.clone());

        AsyncComponentParts { model, widgets }
    }
}
