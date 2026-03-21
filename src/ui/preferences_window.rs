#![allow(unused_assignments)] // vars named in view! macro are not correctly recognized as used

use crate::{
    config::Config,
    plugins::XRPlugin,
    profile::OvrCompatibilityModuleType,
    ui::{
        SENDER_IO_ERR_MSG,
        factories::env_var_row_factory::{EnvVarModel, EnvVarModelInit, EnvVarModelOutMsg},
        preference_rows::{combo_row, spin_row, switch_row},
    },
    util::env_var::parse_var,
};
use adw::prelude::*;
use gtk::glib::{self, clone};
use relm4::prelude::*;
use std::{cell::RefCell, path::PathBuf, rc::Rc};
use tracing::warn;

pub struct PreferencesWindow {
    config: Rc<RefCell<Config>>,
    win: Option<adw::PreferencesDialog>,
    parent: gtk::Window,
    plugins_grp: Option<adw::PreferencesGroup>,
    env_rows: AsyncFactoryVecDeque<EnvVarModel>,
}

#[derive(Debug)]
pub enum PreferencesWindowMsg {
    Present,
    OnClosed,
    PopulatePlugins,
    EnvVarChanged(String, String),
    EnvVarDelete(String),
    AddEnvVar(String),
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
                    // add: &combo_row(
                    //     "XR service",
                    //     None,
                    //     model.config.borrow().global_preferences.xrservice_type.to_string().as_str(),
                    //     XRServiceType::iter().map(XRServiceType::to_string).collect(),
                    //     clone!(#[strong(rename_to=config)] model.config, move |row| {
                    //         config.borrow_mut().global_preferences.xrservice_type = XRServiceType::from(row.selected());
                    //     })
                    // ),
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
            add: advanced_page = &adw::PreferencesPage {
                set_title: "Advanced",
                set_icon_name: Some("preferences-other-symbolic"),
                add: model.env_rows.widget(),
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
            Self::Input::EnvVarChanged(name, value) => {
                self.config
                    .borrow_mut()
                    .global_preferences
                    .env_vars
                    .insert(name, value);
            }
            Self::Input::EnvVarDelete(name) => {
                self.config
                    .borrow_mut()
                    .global_preferences
                    .env_vars
                    .remove(&name);
                let pos = self
                    .env_rows
                    .guard()
                    .iter()
                    .position(|evr| evr.unwrap().name == name);
                if let Some(p) = pos {
                    self.env_rows.guard().remove(p);
                }
            }
            Self::Input::AddEnvVar(var) => {
                let mut conf = self.config.borrow_mut();
                let (name, value) = parse_var(&var);
                if !conf.global_preferences.env_vars.contains_key(&name) {
                    conf.global_preferences
                        .env_vars
                        .insert(name.clone(), value.clone());
                    self.env_rows
                        .guard()
                        .push_back(EnvVarModelInit { name, value });
                }
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let add_env_var_btn = {
            let popover = gtk::Popover::builder().build();
            let popover_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .css_classes(["linked"])
                .build();
            let name_entry = gtk::Entry::builder()
                .placeholder_text("Variable Name...")
                .build();
            let add_btn = gtk::Button::builder()
                .css_classes(["suggested-action"])
                .icon_name("list-add-symbolic")
                .tooltip_text("Add Env Var")
                .build();
            popover_box.append(&name_entry);
            popover_box.append(&add_btn);
            popover.set_child(Some(&popover_box));

            let btn = gtk::MenuButton::builder()
                .icon_name("list-add-symbolic")
                .tooltip_text("Add Environment Variable")
                .css_classes(["flat"])
                .popover(&popover)
                .valign(gtk::Align::Start)
                .halign(gtk::Align::End)
                .build();

            let on_add = clone!(
                #[strong]
                sender,
                #[weak]
                name_entry,
                #[weak]
                popover,
                move || {
                    let key_gstr = name_entry.text();
                    let key = key_gstr.trim();
                    if !key.is_empty() {
                        popover.popdown();
                        name_entry.set_text("");
                        sender.input(Self::Input::AddEnvVar(key.to_string()));
                    }
                }
            );
            name_entry.connect_activate(clone!(
                #[strong]
                on_add,
                move |_| on_add()
            ));
            add_btn.connect_clicked(move |_| on_add());
            btn
        };
        let env_var_prefs_group = {
            let pg = adw::PreferencesGroup::builder()
                .title("Environment Variables")
                .header_suffix(&add_env_var_btn)
                .build();
            if let Some(desc) = pg
                .first_child()
                .and_then(|c| c.first_child())
                .and_then(|c| c.first_child())
                .and_then(|c| c.last_child())
                .and_downcast::<gtk::Label>()
            {
                desc.set_selectable(true);
            } else {
                warn!(
                    "failed to make env var preference group description selectable, please open a bug report"
                );
            }
            pg
        };

        let mut model = Self {
            win: None,
            config: Rc::new(RefCell::new(init.config)),
            parent: init.root_win,
            plugins_grp: None,
            env_rows: AsyncFactoryVecDeque::builder()
                .launch(env_var_prefs_group)
                .forward(sender.input_sender(), |msg| match msg {
                    EnvVarModelOutMsg::Changed(name, value) => {
                        Self::Input::EnvVarChanged(name, value)
                    }
                    EnvVarModelOutMsg::Delete(name) => Self::Input::EnvVarDelete(name),
                }),
        };

        {
            let mut guard = model.env_rows.guard();
            guard.clear();
            for (k, v) in model.config.borrow().global_preferences.env_vars.iter() {
                guard.push_back(EnvVarModelInit {
                    name: k.clone(),
                    value: v.clone(),
                });
            }
        }

        let widgets = view_output!();

        model.win = Some(widgets.win.clone());
        model.plugins_grp = Some(widgets.plugins_grp.clone());

        AsyncComponentParts { model, widgets }
    }
}
