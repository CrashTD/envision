use super::{
    alert::alert,
    factories::env_var_row_factory::{EnvVarModel, EnvVarModelInit, VarType},
};
use crate::{
    env_var_descriptions::env_var_descriptions_as_paragraph,
    profile::{LighthouseDriver, Profile, XRServiceType},
    ui::preference_rows::{combo_row, entry_row, path_row, switch_row},
};
use adw::prelude::*;
use gtk::glib::clone;
use relm4::{factory::AsyncFactoryVecDeque, prelude::*};
use std::{cell::RefCell, rc::Rc};

#[tracker::track]
pub struct ProfileEditor {
    profile: Rc<RefCell<Profile>>,
    #[tracker::do_not_track]
    win: Option<adw::Window>,
    #[tracker::do_not_track]
    env_rows: AsyncFactoryVecDeque<EnvVarModel>,
    #[tracker::do_not_track]
    xrservice_cmake_flags_rows: AsyncFactoryVecDeque<EnvVarModel>,
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

#[relm4::component(pub)]
impl SimpleComponent for ProfileEditor {
    type Init = ProfileEditorInit;
    type Input = ProfileEditorMsg;
    type Output = ProfileEditorOutMsg;

    view! {
        #[name(win)]
        adw::Window {
            set_modal: true,
            set_transient_for: Some(&init.root_win),
            #[track = "model.changed(Self::profile())"]
            set_title: Some(model.profile.borrow().name.as_str()),
            set_default_height: 500,
            set_default_width: 600,
            adw::ToolbarView {
                set_top_bar_style: adw::ToolbarStyle::Flat,
                set_hexpand: true,
                set_vexpand: true,
                add_top_bar: top_bar = &adw::HeaderBar {
                    set_vexpand: false,
                    pack_end: save_btn = &gtk::Button {
                        set_label: "Save",
                        add_css_class: "suggested-action",
                        connect_clicked[sender] => move |_| {
                            sender.input(Self::Input::SaveProfile);
                        },
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
                            clone!(@strong prof => move |row| {
                                prof.borrow_mut().name = row.text().to_string();
                            })
                        ),
                        add: &switch_row(
                            "Update on Build", None,
                            model.profile.borrow().pull_on_build,
                            clone!(@strong prof => move |_, state| {
                                prof.borrow_mut().pull_on_build = state;
                                gtk::glib::Propagation::Proceed
                            })
                        ),
                        add: &path_row(
                            "Install Prefix",
                            None,
                            Some(model.profile.borrow().prefix.clone()),
                            Some(init.root_win.clone()),
                            clone!(@strong prof => move |n_path| {
                                prof.borrow_mut().prefix = n_path.unwrap_or_default()
                            }),
                        ),
                    },
                    add: xrservicegrp = &adw::PreferencesGroup {
                        set_title: "XR Service",
                        set_description: Some(concat!(
                            "When specifying a repository, you can set a ",
                            "specific git ref (branch, tag, commit...) by ",
                            "appending a '#' followed by the ref.\n\n",
                            "For launch options, you can insert %command% as ",
                            "a placeholder for the actual XR Service command.",
                        )),
                        add: &combo_row(
                            "XR Service Type",
                            Some("Monado is for PCVR headsets, while WiVRn is for Andorid standalone headsets"),
                            model.profile.borrow().xrservice_type.to_string().as_str(),
                            XRServiceType::iter()
                                .map(XRServiceType::to_string)
                                .collect::<Vec<String>>(),
                            clone!(@strong prof => move |row| {
                                prof.borrow_mut().xrservice_type =
                                    XRServiceType::from_number(row.selected());
                            }),
                        ),
                        add: &entry_row(
                            "XR Service Launch Options",
                            model.profile.borrow().xrservice_launch_options.as_str(),
                            clone!(@strong prof => move |row| {
                                prof.borrow_mut().xrservice_launch_options = row.text().trim().to_string();
                            })
                        ),
                        add: &combo_row(
                            "Lighthouse Driver",
                            Some(concat!(
                                "Driver for lighhouse tracked XR devices (ie: Valve Index, HTC Vive...). Only applicable for Monado.\n\n",
                                "Vive: 3DOF tracking\n\n",
                                "Survive: 6DOF reverse engineered lighthouse tracking provided by Libsurvive\n\n",
                                "SteamVR: 6DOF lighthouse tracking using the proprietary SteamVR driver",
                            )),
                            model.profile.borrow().lighthouse_driver.to_string().as_str(),
                            LighthouseDriver::iter()
                                .map(LighthouseDriver::to_string)
                                .collect::<Vec<String>>(),
                            clone!(@strong prof => move |row| {
                                prof.borrow_mut().lighthouse_driver =
                                    LighthouseDriver::from_number(row.selected());
                            })
                        ),
                        add: &path_row(
                            "XR Service Path",
                            None,
                            Some(model.profile.borrow().xrservice_path.clone()),
                            Some(init.root_win.clone()),
                            clone!(@strong prof => move |n_path| {
                                prof.borrow_mut().xrservice_path = n_path.unwrap_or_default()
                            }),
                        ),
                        add: &entry_row(
                            "XR Service Repo",
                            model.profile.borrow().xrservice_repo.clone().unwrap_or_default().as_str(),
                            clone!(@strong prof => move |row| {
                                let n_val = row.text().to_string();
                                prof.borrow_mut().xrservice_repo = (!n_val.is_empty()).then_some(n_val);
                            })
                        ),
                    },
                    add: model.xrservice_cmake_flags_rows.widget(),
                    add: opencompgrp = &adw::PreferencesGroup {
                        set_title: "OpenComposite",
                        set_description: Some("OpenVR driver built on top of OpenXR\n\nWhen specifying a repository, you can set a specific git ref (branch, tag, commit...) by appending a '#' followed by the ref"),
                        add: &path_row(
                            "OpenComposite Path", None,
                            Some(model.profile.borrow().opencomposite_path.clone()),
                            Some(init.root_win.clone()),
                            clone!(@strong prof => move |n_path| {
                                prof.borrow_mut().opencomposite_path = n_path.unwrap_or_default();
                            })
                        ),
                        add: &entry_row(
                            "OpenComposite Repo",
                            model.profile.borrow().opencomposite_repo.clone().unwrap_or_default().as_str(),
                            clone!(@strong prof => move |row| {
                                let n_val = row.text().to_string();
                                prof.borrow_mut().opencomposite_repo = (!n_val.is_empty()).then_some(n_val);
                            })
                        ),
                    },
                    add: libsurvivegrp = &adw::PreferencesGroup {
                        set_title: "Libsurvive",
                        set_description: Some("Lighthouse tracking driver\n\nWhen specifying a repository, you can set a specific git ref (branch, tag, commit...) by appending a '#' followed by the ref"),
                        add: &switch_row(
                            "Enable Libsurvive", None,
                            model.profile.borrow().features.libsurvive.enabled,
                            clone!(@strong prof => move |_, state| {
                                prof.borrow_mut().features.libsurvive.enabled = state;
                                gtk::glib::Propagation::Proceed
                            })
                        ),
                        add: &path_row(
                            "Libsurvive Path", None,
                            model.profile.borrow().features.libsurvive.path.clone(),
                            Some(init.root_win.clone()),
                            clone!(@strong prof => move |n_path| {
                                prof.borrow_mut().features.libsurvive.path = n_path;
                            })
                        ),
                        add: &entry_row(
                            "Libsurvive Repo",
                            model.profile.borrow().features.libsurvive.repo.clone().unwrap_or_default().as_str(),
                            clone!(@strong prof => move |row| {
                                let n_val = row.text().to_string();
                                prof.borrow_mut().features.libsurvive.repo = (!n_val.is_empty()).then_some(n_val);
                            })
                        ),
                    },
                    add: openhmdgrp = &adw::PreferencesGroup {
                        set_title: "OpenHMD",
                        set_description: Some("Legacy driver for older Oculus HMDs\n\nWhen specifying a repository, you can set a specific git ref (branch, tag, commit...) by appending a '#' followed by the ref"),
                        add: &switch_row(
                            "Enable OpenHMD", None,
                            model.profile.borrow().features.openhmd.enabled,
                            clone!(@strong prof => move |_, state| {
                                prof.borrow_mut().features.openhmd.enabled = state;
                                gtk::glib::Propagation::Proceed
                            })
                        ),
                        add: &path_row(
                            "OpenHMD Path", None,
                            model.profile.borrow().features.openhmd.path.clone(),
                            Some(init.root_win.clone()),
                            clone!(@strong prof => move |n_path| {
                                prof.borrow_mut().features.openhmd.path = n_path;
                            })
                        ),
                        add: &entry_row(
                            "OpenHMD Repo",
                            model.profile.borrow().features.openhmd.repo.clone().unwrap_or_default().as_str(),
                            clone!(@strong prof => move |row| {
                                let n_val = row.text().to_string();
                                prof.borrow_mut().features.openhmd.repo = (!n_val.is_empty()).then_some(n_val);
                            })
                        ),
                    },
                    add: basaltgrp = &adw::PreferencesGroup {
                        set_title: "Basalt",
                        set_description: Some("Camera based SLAM tracking driver\n\nWhen specifying a repository, you can set a specific git ref (branch, tag, commit...) by appending a '#' followed by the ref"),
                        add: &switch_row(
                            "Enable Basalt", None,
                            model.profile.borrow().features.basalt.enabled,
                            clone!(@strong prof => move |_, state| {
                                prof.borrow_mut().features.basalt.enabled = state;
                                gtk::glib::Propagation::Proceed
                            })
                        ),
                        add: &path_row(
                            "Basalt Path", None,
                            model.profile.borrow().features.basalt.path.clone(),
                            Some(init.root_win.clone()),
                            clone!(@strong prof => move |n_path| {
                                prof.borrow_mut().features.basalt.path = n_path;
                            })
                        ),
                        add: &entry_row(
                            "Basalt Repo",
                            model.profile.borrow().features.basalt.repo.clone().unwrap_or_default().as_str(),
                            clone!(@strong prof => move |row| {
                                let n_val = row.text().to_string();
                                prof.borrow_mut().features.basalt.repo = n_val.is_empty().then_some(n_val);
                            })
                        ),
                    },
                    add: mercurygrp = &adw::PreferencesGroup {
                        set_title: "Mercury",
                        set_description: Some("Camera and OpenCV based hand tracking driver"),
                        add: &switch_row(
                            "Enable Mercury", None,
                            model.profile.borrow().features.mercury_enabled,
                            clone!(@strong prof => move |_, state| {
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
                self.win.as_ref().unwrap().present();
            }
            Self::Input::SaveProfile => {
                let prof = self.profile.borrow();
                if prof.validate() {
                    sender
                        .output(ProfileEditorOutMsg::SaveProfile(prof.clone()))
                        .expect("Sender output failed");
                    self.win.as_ref().unwrap().close();
                } else {
                    alert(
                        "Profile validation failed",
                        Some("Check the values you set and try again"),
                        Some(&self.win.as_ref().unwrap().clone().upcast::<gtk::Window>()),
                    );
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
            Self::Input::AddEnvVar(name) => {
                let mut prof = self.profile.borrow_mut();
                if !prof.environment.contains_key(&name) {
                    prof.environment.insert(name.clone(), "".to_string());
                    self.env_rows.guard().push_back(EnvVarModelInit {
                        name,
                        value: "".to_string(),
                        var_type: VarType::EnvVar,
                    });
                }
            }
            Self::Input::AddXrServiceCmakeFlag(name) => {
                let mut prof = self.profile.borrow_mut();
                if !prof.xrservice_cmake_flags.contains_key(&name) {
                    prof.xrservice_cmake_flags
                        .insert(name.clone(), "".to_string());
                    self.xrservice_cmake_flags_rows
                        .guard()
                        .push_back(EnvVarModelInit {
                            name,
                            value: "".to_string(),
                            var_type: VarType::XrServiceCmakeFlags,
                        });
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
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

                    add_btn.connect_clicked(
                        clone!(@strong sender, @strong name_entry, @strong popover => move |_| {
                            let key_gstr = name_entry.text();
                            let key = key_gstr.trim();
                            if !key.is_empty() {
                                popover.popdown();
                                name_entry.set_text("");
                                sender.input($event(key.to_string()));
                            }
                        })
                    );
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

        let mut model = Self {
            profile,
            win: None,
            env_rows: AsyncFactoryVecDeque::new(
                adw::PreferencesGroup::builder()
                    .title("Environment Variables")
                    .description(env_var_descriptions_as_paragraph())
                    .header_suffix(&add_env_var_btn)
                    .build(),
                sender.input_sender(),
            ),
            xrservice_cmake_flags_rows: AsyncFactoryVecDeque::new(
                adw::PreferencesGroup::builder()
                    .title("XR Service CMake Flags")
                    .header_suffix(&add_cmake_flag_btn)
                    .build(),
                sender.input_sender(),
            ),
            tracker: 0,
        };
        {
            let mut guard = model.env_rows.guard();
            guard.clear();
            for (k, v) in prof.borrow().environment.iter() {
                guard.push_back(EnvVarModelInit {
                    name: k.clone(),
                    value: v.clone(),
                    var_type: VarType::EnvVar,
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
                    var_type: VarType::XrServiceCmakeFlags,
                });
            }
        }

        let widgets = view_output!();
        model.win = Some(widgets.win.clone());

        ComponentParts { model, widgets }
    }
}
