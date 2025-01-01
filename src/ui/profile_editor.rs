use super::{
    alert::alert_w_widget,
    factories::env_var_row_factory::{EnvVarModel, EnvVarModelInit, EnvVarModelOutMsg},
    preference_rows::{combo_row, entry_row, path_row, switch_row},
    ADW_DIALOG_WIDTH,
};
use crate::{
    env_var_descriptions::ENV_VAR_DESCRIPTIONS_AS_PARAGRAPH,
    profile::{LighthouseDriver, OvrCompatibilityModuleType, Profile, XRServiceType},
};
use adw::prelude::*;
use gtk::glib::{self, clone};
use relm4::{factory::AsyncFactoryVecDeque, prelude::*};
use std::{cell::RefCell, path::PathBuf, rc::Rc};
use tracing::warn;

#[tracker::track]
pub struct ProfileEditor {
    profile: Rc<RefCell<Profile>>,
    #[tracker::do_not_track]
    win: Option<adw::Dialog>,
    #[tracker::do_not_track]
    env_rows: AsyncFactoryVecDeque<EnvVarModel>,
    #[tracker::do_not_track]
    xrservice_cmake_flags_rows: AsyncFactoryVecDeque<EnvVarModel>,
    #[tracker::do_not_track]
    parent: gtk::Window,
}

#[derive(Debug)]
pub enum ProfileEditorMsg {
    Present,
    EnvVarChanged(String, String),
    EnvVarDelete(String),
    XrServiceCmakeFlagsChanged(String, String),
    XrServiceCmakeFlagsDelete(String),
    AddEnvVar(String),
    AddXrServiceCmakeFlag(String),
    SaveProfile,
}

#[derive(Debug)]
pub enum ProfileEditorOutMsg {
    SaveProfile(Profile),
}

pub struct ProfileEditorInit {
    pub root_win: gtk::Window,
    pub profile: Profile,
}

/// This parses a var (either env var or cmake var) and if it contains
/// a '=' char, it assumes it's key and value, otherwise it's just going to
/// be the key and the value is gonna be an empty string
fn parse_var(input: &str) -> (String, String) {
    if input.contains('=') {
        let mut sp = input.split('=');
        (
            sp.next().unwrap().to_string(),
            sp.next().unwrap().to_string(),
        )
    } else {
        (input.to_string(), "".to_string())
    }
}

#[relm4::component(pub)]
impl SimpleComponent for ProfileEditor {
    type Init = ProfileEditorInit;
    type Input = ProfileEditorMsg;
    type Output = ProfileEditorOutMsg;

    view! {
        #[name(win)]
        adw::Dialog {
            #[track = "model.changed(Self::profile())"]
            set_title: model.profile.borrow().name.as_str(),
            set_content_width: ADW_DIALOG_WIDTH,
            #[wrap(Some)]
            set_child: tbview = &adw::ToolbarView {
                set_top_bar_style: adw::ToolbarStyle::Flat,
                set_hexpand: true,
                set_vexpand: true,
                add_top_bar: top_bar = &adw::HeaderBar {
                    set_vexpand: false,
                    set_show_end_title_buttons: false,
                    set_show_start_title_buttons: false,
                    pack_end: save_btn = &gtk::Button {
                        set_label: "Save",
                        add_css_class: "suggested-action",
                        connect_clicked[sender] => move |_| {
                            sender.input(Self::Input::SaveProfile);
                        },
                    },
                    pack_start: cancel_btn = &gtk::Button {
                        set_label: "Cancel",
                        add_css_class: "destructive-action",
                        connect_clicked[win] => move |_| {
                            win.close();
                        }
                    },
                },
                #[wrap(Some)]
                set_content: pref_page = &adw::PreferencesPage {
                    set_hexpand: true,
                    set_vexpand: true,
                    add: maingrp = &adw::PreferencesGroup {
                        set_title: "General",
                        add: &entry_row(
                            "Profile Name",
                            model.profile.borrow().name.as_str(),
                            clone!(#[strong] prof, move |row| {
                                prof.borrow_mut().name = row.text().to_string();
                            })
                        ),
                        add: &switch_row(
                            "Update on Build", None,
                            model.profile.borrow().pull_on_build,
                            clone!(#[strong] prof, move |_, state| {
                                prof.borrow_mut().pull_on_build = state;
                                gtk::glib::Propagation::Proceed
                            })
                        ),
                        add: &path_row(
                            "Install Prefix",
                            None,
                            Some(model.profile.borrow().prefix.to_string_lossy().to_string()),
                            Some(init.root_win.clone()),
                            clone!(#[strong] prof, move |n_path| {
                                prof.borrow_mut().prefix = n_path.unwrap_or_default().into();
                            }),
                        ),
                        add: &switch_row("Dependency Check",
                            Some("Warning: disabling dependency checks may result in build failures"),
                            !model.profile.borrow().skip_dependency_check,
                            clone!(#[strong] prof, move |_, state| {
                                prof.borrow_mut().skip_dependency_check = !state;
                                gtk::glib::Propagation::Proceed
                            })
                        ),
                    },
                    add: xrservicegrp = &adw::PreferencesGroup {
                        set_title: "XR Service",
                        set_description: Some(concat!(
                            "For launch options, you can insert %command% as ",
                            "a placeholder for the actual XR Service command, ",
                            "similarly to how Steam launch options work.",
                        )),
                        add: &combo_row(
                            "XR Service Type",
                            Some("Monado is for PCVR headsets, while WiVRn is for Android standalone headsets"),
                            model.profile.borrow().xrservice_type.to_string().as_str(),
                            XRServiceType::iter()
                                .map(XRServiceType::to_string)
                                .collect::<Vec<String>>(),
                            clone!(#[strong] prof, move |row| {
                                prof.borrow_mut().xrservice_type =
                                    XRServiceType::from(row.selected());
                            }),
                        ),
                        add: &entry_row(
                            "XR Service Launch Options",
                            model.profile.borrow().xrservice_launch_options.as_str(),
                            clone!(#[strong] prof, move |row| {
                                prof.borrow_mut().xrservice_launch_options = row.text().trim().to_string();
                            })
                        ),
                        add: &combo_row(
                            "Lighthouse Driver",
                            Some(concat!(
                                "Driver for lighhouse tracked XR devices (ie: Valve Index, HTC Vive...). Only applicable for Monado.\n\n",
                                " \u{2022} Vive: 3DOF tracking; use this if your device doesn't use lighthouses\n",
                                " \u{2022} Survive: 6DOF reverse engineered lighthouse tracking provided by Libsurvive\n",
                                " \u{2022} SteamVR: 6DOF lighthouse tracking using the proprietary SteamVR driver",
                            )),
                            model.profile.borrow().lighthouse_driver.to_string().as_str(),
                            LighthouseDriver::iter()
                                .map(LighthouseDriver::to_string)
                                .collect::<Vec<String>>(),
                            clone!(#[strong] prof, move |row| {
                                prof.borrow_mut().lighthouse_driver =
                                    LighthouseDriver::from(row.selected());
                            })
                        ),
                        add: &path_row(
                            "XR Service Path",
                            None,
                            Some(model.profile.borrow().xrservice_path.clone().to_string_lossy().to_string()),
                            Some(init.root_win.clone()),
                            clone!(#[strong] prof, move |n_path| {
                                prof.borrow_mut().xrservice_path = n_path.unwrap_or_default().into();
                            }),
                        ),
                        add: &entry_row(
                            "XR Service Repo",
                            model.profile.borrow().xrservice_repo.clone().unwrap_or_default().as_str(),
                            clone!(#[strong] prof, move |row| {
                                let n_val = row.text().to_string();
                                prof.borrow_mut().xrservice_repo = (!n_val.is_empty()).then_some(n_val);
                            })
                        ),
                        add: &entry_row(
                            "XR Service Branch",
                            model.profile.borrow().xrservice_branch.clone().unwrap_or_default().as_str(),
                            clone!(#[strong] prof, move |row| {
                                let n_val = row.text().to_string();
                                prof.borrow_mut().xrservice_branch = (!n_val.is_empty()).then_some(n_val);
                            })
                        ),
                    },
                    add: model.xrservice_cmake_flags_rows.widget(),
                    add: ovr_comp_grp = &adw::PreferencesGroup {
                        set_title: "OpenVR Compatibility",
                        set_description: Some("OpenVR compatibility module, translates between OpenXR and OpenVR to run legacy OpenVR apps"),
                        add: &combo_row(
                            "OpenVR Module Type",
                            None,
                            model.profile.borrow().ovr_comp.mod_type.to_string().as_str(),
                            OvrCompatibilityModuleType::iter()
                                .map(OvrCompatibilityModuleType::to_string)
                                .collect::<Vec<String>>(),
                            clone!(#[strong] prof, move |row| {
                                prof.borrow_mut().ovr_comp.mod_type =
                                    OvrCompatibilityModuleType::from(row.selected());
                            }),
                        ),
                        add: &path_row(
                            "OpenVR Module Path", None,
                            Some(model.profile.borrow().ovr_comp.path.clone().to_string_lossy().to_string()),
                            Some(init.root_win.clone()),
                            clone!(#[strong] prof, move |n_path| {
                                prof.borrow_mut().ovr_comp.path = n_path.unwrap_or_default().into();
                            })
                        ),
                        add: &entry_row(
                            "OpenVR Compatibility Repo",
                            model.profile.borrow().ovr_comp.repo.clone().unwrap_or_default().as_str(),
                            clone!(#[strong] prof, move |row| {
                                let n_val = row.text().to_string();
                                prof.borrow_mut().ovr_comp.repo = (!n_val.is_empty()).then_some(n_val);
                            })
                        ),
                        add: &entry_row(
                            "OpenVR Compatibility Branch",
                            model.profile.borrow().ovr_comp.branch.clone().unwrap_or_default().as_str(),
                            clone!(#[strong] prof, move |row| {
                                let n_val = row.text().to_string();
                                prof.borrow_mut().ovr_comp.branch = (!n_val.is_empty()).then_some(n_val);
                            })
                        ),
                    },
                    add: libsurvivegrp = &adw::PreferencesGroup {
                        set_title: "Libsurvive",
                        set_description: Some("Lighthouse tracking driver"),
                        add: &switch_row(
                            "Enable Libsurvive", None,
                            model.profile.borrow().features.libsurvive.enabled,
                            clone!(#[strong] prof, move |_, state| {
                                prof.borrow_mut().features.libsurvive.enabled = state;
                                gtk::glib::Propagation::Proceed
                            })
                        ),
                        add: &path_row(
                            "Libsurvive Path", None,
                            model.profile.borrow().features.libsurvive.path.clone().map(|p| p.to_string_lossy().to_string()),
                            Some(init.root_win.clone()),
                            clone!(#[strong] prof, move |n_path| {
                                prof.borrow_mut().features.libsurvive.path = n_path.map(PathBuf::from);
                            })
                        ),
                        add: &entry_row(
                            "Libsurvive Repo",
                            model.profile.borrow().features.libsurvive.repo.clone().unwrap_or_default().as_str(),
                            clone!(#[strong] prof, move |row| {
                                let n_val = row.text().to_string();
                                prof.borrow_mut().features.libsurvive.repo = (!n_val.is_empty()).then_some(n_val);
                            })
                        ),
                        add: &entry_row(
                            "Libsurvive Branch",
                            model.profile.borrow().features.libsurvive.branch.clone().unwrap_or_default().as_str(),
                            clone!(#[strong] prof, move |row| {
                                let n_val = row.text().to_string();
                                prof.borrow_mut().features.libsurvive.branch = (!n_val.is_empty()).then_some(n_val);
                            })
                        ),
                    },
                    add: openhmdgrp = &adw::PreferencesGroup {
                        set_title: "OpenHMD",
                        set_description: Some("Legacy driver for older Oculus HMDs"),
                        add: &switch_row(
                            "Enable OpenHMD", None,
                            model.profile.borrow().features.openhmd.enabled,
                            clone!(#[strong] prof, move |_, state| {
                                prof.borrow_mut().features.openhmd.enabled = state;
                                gtk::glib::Propagation::Proceed
                            })
                        ),
                        add: &path_row(
                            "OpenHMD Path", None,
                            model.profile.borrow().features.openhmd.path.clone().map(|p| p.to_string_lossy().to_string()),
                            Some(init.root_win.clone()),
                            clone!(#[strong] prof, move |n_path| {
                                prof.borrow_mut().features.openhmd.path = n_path.map(PathBuf::from);
                            })
                        ),
                        add: &entry_row(
                            "OpenHMD Repo",
                            model.profile.borrow().features.openhmd.repo.clone().unwrap_or_default().as_str(),
                            clone!(#[strong] prof, move |row| {
                                let n_val = row.text().to_string();
                                prof.borrow_mut().features.openhmd.repo = (!n_val.is_empty()).then_some(n_val);
                            })
                        ),
                        add: &entry_row(
                            "OpenHMD Branch",
                            model.profile.borrow().features.openhmd.branch.clone().unwrap_or_default().as_str(),
                            clone!(#[strong] prof, move |row| {
                                let n_val = row.text().to_string();
                                prof.borrow_mut().features.openhmd.branch = (!n_val.is_empty()).then_some(n_val);
                            })
                        ),
                    },
                    add: basaltgrp = &adw::PreferencesGroup {
                        set_title: "Basalt",
                        set_description: Some("Camera based SLAM tracking driver"),
                        add: &switch_row(
                            "Enable Basalt", None,
                            model.profile.borrow().features.basalt.enabled,
                            clone!(#[strong] prof, move |_, state| {
                                prof.borrow_mut().features.basalt.enabled = state;
                                gtk::glib::Propagation::Proceed
                            })
                        ),
                        add: &path_row(
                            "Basalt Path", None,
                            model.profile.borrow().features.basalt.path.clone().map(|p| p.to_string_lossy().to_string()),
                            Some(init.root_win.clone()),
                            clone!(#[strong] prof, move |n_path| {
                                prof.borrow_mut().features.basalt.path = n_path.map(PathBuf::from);
                            })
                        ),
                        add: &entry_row(
                            "Basalt Repo",
                            model.profile.borrow().features.basalt.repo.clone().unwrap_or_default().as_str(),
                            clone!(#[strong] prof, move |row| {
                                let n_val = row.text().to_string();
                                prof.borrow_mut().features.basalt.repo = (!n_val.is_empty()).then_some(n_val);
                            })
                        ),
                        add: &entry_row(
                            "Basalt Branch",
                            model.profile.borrow().features.basalt.branch.clone().unwrap_or_default().as_str(),
                            clone!(#[strong] prof, move |row| {
                                let n_val = row.text().to_string();
                                prof.borrow_mut().features.basalt.branch = (!n_val.is_empty()).then_some(n_val);
                            })
                        ),
                    },
                    add: mercurygrp = &adw::PreferencesGroup {
                        set_title: "Mercury",
                        set_description: Some("Camera and OpenCV based hand tracking driver"),
                        add: &switch_row(
                            "Enable Mercury", None,
                            model.profile.borrow().features.mercury_enabled,
                            clone!(#[strong] prof, move |_, state| {
                                prof.borrow_mut().features.mercury_enabled = state;
                                gtk::glib::Propagation::Proceed
                            })
                        ),
                    },
                    add: model.env_rows.widget(),
                }
            }
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        self.reset();

        match message {
            Self::Input::Present => {
                self.win.as_ref().unwrap().present(Some(&self.parent));
            }
            Self::Input::SaveProfile => {
                let prof = self.profile.borrow();
                if let Err(errors) = prof.validate() {
                    alert_w_widget(
                        "Profile validation failed",
                        None,
                        Some(
                            &gtk::Label::builder()
                                .label(
                                    errors
                                        .iter()
                                        .map(|e| format!(" \u{2022} {e}"))
                                        .collect::<Vec<String>>()
                                        .join("\n"),
                                )
                                .wrap(true)
                                .xalign(0.0)
                                .max_width_chars(40)
                                .build()
                                .upcast(),
                        ),
                        Some(&self.parent),
                    );
                } else {
                    sender
                        .output(ProfileEditorOutMsg::SaveProfile(prof.clone()))
                        .expect("Sender output failed");
                    self.win.as_ref().unwrap().close();
                }
            }
            Self::Input::EnvVarChanged(name, value) => {
                self.profile.borrow_mut().environment.insert(name, value);
            }
            Self::Input::EnvVarDelete(name) => {
                self.profile.borrow_mut().environment.remove(&name);
                let pos = self
                    .env_rows
                    .guard()
                    .iter()
                    .position(|evr| evr.unwrap().name == name);
                if let Some(p) = pos {
                    self.env_rows.guard().remove(p);
                }
            }
            Self::Input::XrServiceCmakeFlagsChanged(name, value) => {
                self.profile
                    .borrow_mut()
                    .xrservice_cmake_flags
                    .insert(name, value);
            }
            Self::Input::XrServiceCmakeFlagsDelete(name) => {
                self.profile
                    .borrow_mut()
                    .xrservice_cmake_flags
                    .remove(&name);
                let pos = self
                    .xrservice_cmake_flags_rows
                    .guard()
                    .iter()
                    .position(|evr| evr.unwrap().name == name);
                if let Some(p) = pos {
                    self.xrservice_cmake_flags_rows.guard().remove(p);
                }
            }
            Self::Input::AddEnvVar(var) => {
                let mut prof = self.profile.borrow_mut();
                let (name, value) = parse_var(&var);
                if !prof.environment.contains_key(&name) {
                    prof.environment.insert(name.clone(), value.clone());
                    self.env_rows
                        .guard()
                        .push_back(EnvVarModelInit { name, value });
                }
            }
            Self::Input::AddXrServiceCmakeFlag(var) => {
                let mut prof = self.profile.borrow_mut();
                let (name, value) = parse_var(&var);
                if !prof.xrservice_cmake_flags.contains_key(&name) {
                    prof.xrservice_cmake_flags
                        .insert(name.clone(), value.clone());
                    self.xrservice_cmake_flags_rows
                        .guard()
                        .push_back(EnvVarModelInit { name, value });
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let (add_env_var_btn, add_cmake_flag_btn) = {
            macro_rules! add_var_btn {
                ($name:expr, $event:expr) => {{
                    let popover = gtk::Popover::builder().build();
                    let popover_box = gtk::Box::builder()
                        .orientation(gtk::Orientation::Horizontal)
                        .css_classes(["linked"])
                        .build();
                    let name_entry = gtk::Entry::builder()
                        .placeholder_text(format!("{} Name...", $name))
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
                                sender.input($event(key.to_string()));
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
                }};
            }
            (
                add_var_btn!("Env Var", Self::Input::AddEnvVar),
                add_var_btn!("XR Service CMake Flag", Self::Input::AddXrServiceCmakeFlag),
            )
        };

        let profile = Rc::new(RefCell::new(init.profile));
        let prof = profile.clone();

        let env_var_prefs_group = {
            let pg = adw::PreferencesGroup::builder()
                .title("Environment Variables")
                .description(ENV_VAR_DESCRIPTIONS_AS_PARAGRAPH.as_str())
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
                warn!("failed to make env var preference group description selectable, please open a bug report");
            }
            pg
        };
        let mut model = Self {
            profile,
            win: None,
            env_rows: AsyncFactoryVecDeque::builder()
                .launch(env_var_prefs_group)
                .forward(sender.input_sender(), |msg| match msg {
                    EnvVarModelOutMsg::Changed(name, value) => {
                        ProfileEditorMsg::EnvVarChanged(name, value)
                    }
                    EnvVarModelOutMsg::Delete(name) => ProfileEditorMsg::EnvVarDelete(name),
                }),
            xrservice_cmake_flags_rows: AsyncFactoryVecDeque::builder()
                .launch(
                    adw::PreferencesGroup::builder()
                        .title("XR Service CMake Flags")
                        .header_suffix(&add_cmake_flag_btn)
                        .build(),
                )
                .forward(sender.input_sender(), |msg| match msg {
                    EnvVarModelOutMsg::Changed(name, value) => {
                        ProfileEditorMsg::XrServiceCmakeFlagsChanged(name, value)
                    }
                    EnvVarModelOutMsg::Delete(name) => {
                        ProfileEditorMsg::XrServiceCmakeFlagsDelete(name)
                    }
                }),
            parent: init.root_win.clone(),
            tracker: 0,
        };
        {
            let mut guard = model.env_rows.guard();
            guard.clear();
            for (k, v) in prof.borrow().environment.iter() {
                guard.push_back(EnvVarModelInit {
                    name: k.clone(),
                    value: v.clone(),
                });
            }
        }
        {
            let mut guard = model.xrservice_cmake_flags_rows.guard();
            guard.clear();
            for (k, v) in prof.borrow().xrservice_cmake_flags.iter() {
                guard.push_back(EnvVarModelInit {
                    name: k.clone(),
                    value: v.clone(),
                });
            }
        }

        let widgets = view_output!();
        model.win = Some(widgets.win.clone());

        ComponentParts { model, widgets }
    }
}
