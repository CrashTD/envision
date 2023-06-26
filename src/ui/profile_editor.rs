use super::factories::{
    entry_row_factory::{EntryModel, EntryModelInit},
    env_var_row_factory::{EnvVarModel, EnvVarModelInit},
    path_row_factory::{PathModel, PathModelInit},
    switch_row_factory::{SwitchModel, SwitchModelInit},
};
use crate::{
    env_var_descriptions::env_var_descriptions_as_paragraph,
    profile::{Profile, XRServiceType},
};
use adw::prelude::*;
use gtk::prelude::*;
use relm4::{factory::FactoryVecDeque, prelude::*};

#[tracker::track]
pub struct ProfileEditor {
    profile: Option<Profile>,

    #[tracker::do_not_track]
    win: Option<adw::PreferencesWindow>,

    #[tracker::do_not_track]
    name_row: adw::EntryRow,
    #[tracker::do_not_track]
    type_row: adw::ComboRow,
    #[tracker::do_not_track]
    pull_on_build_switch: Option<gtk::Switch>,
    #[tracker::do_not_track]
    env_rows: FactoryVecDeque<EnvVarModel>,
    #[tracker::do_not_track]
    switch_rows: FactoryVecDeque<SwitchModel>,
    #[tracker::do_not_track]
    path_rows: FactoryVecDeque<PathModel>,
    #[tracker::do_not_track]
    repo_rows: FactoryVecDeque<EntryModel>,
}

#[derive(Debug)]
pub enum ProfileEditorMsg {
    Present(Profile),
    EnvVarChanged(String, String),
    EnvVarDelete(String),
    EntryChanged(String, String),
    PathChanged(String, Option<String>),
    SwitchChanged(String, bool),
    ComboChanged(String, String),
    AddEnvVar(String),
    SaveProfile,
}

#[derive(Debug)]
pub enum ProfileEditorOutMsg {
    SaveProfile(Profile),
}

pub struct ProfileEditorInit {
    pub root_win: gtk::Window,
}

#[relm4::component(pub)]
impl SimpleComponent for ProfileEditor {
    type Init = ProfileEditorInit;
    type Input = ProfileEditorMsg;
    type Output = ProfileEditorOutMsg;

    view! {
        #[name(win)]
        adw::PreferencesWindow {
            set_hide_on_close: true,
            set_modal: true,
            set_transient_for: Some(&init.root_win),
            #[track = "model.changed(Self::profile())"]
            set_title: match model.profile.as_ref() {
                Some(p) => Some(p.name.as_str()),
                None => None,
            },
            add: mainpage = &adw::PreferencesPage {
                add: maingrp = &adw::PreferencesGroup {
                    set_title: "General",
                    model.name_row.clone(),
                    model.type_row.clone(),
                    adw::ActionRow {
                        set_title: "Update on Build",
                        add_suffix: pull_on_build_switch = &gtk::Switch {
                            set_valign: gtk::Align::Center,
                            connect_state_set[sender] => move |_, state| {
                                sender.input(Self::Input::SwitchChanged("pull_on_build".into(), state));
                                gtk::Inhibit(false)
                            }
                        },
                        set_activatable_widget: Some(&pull_on_build_switch),
                    }
                },
                add: model.env_rows.widget(),
                add: model.switch_rows.widget(),
                add: model.path_rows.widget(),
                add: model.repo_rows.widget(),
                add: save_grp = &adw::PreferencesGroup {
                    add: save_box = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_hexpand: true,
                        gtk::Button {
                            set_halign: gtk::Align::Center,
                            set_label: "Save",
                            add_css_class: "pill",
                            add_css_class: "suggested-action",
                            connect_clicked[sender] => move |_| {
                                sender.input(Self::Input::SaveProfile);
                            },
                        },
                    }
                },
            }
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        self.reset();

        match message {
            Self::Input::Present(prof) => {
                let p = prof.clone();

                self.name_row.set_text(p.name.as_str());

                self.type_row.set_selected(prof.xrservice_type.as_number());

                self.pull_on_build_switch
                    .as_ref()
                    .unwrap()
                    .set_active(prof.pull_on_build);

                {
                    let mut guard = self.env_rows.guard();
                    guard.clear();
                    for (k, v) in p.environment.iter() {
                        guard.push_back(EnvVarModelInit {
                            name: k.clone(),
                            value: v.clone(),
                        });
                    }
                }

                {
                    let mut guard = self.switch_rows.guard();
                    guard.clear();
                    guard.push_back(SwitchModelInit {
                        name: "Libsurvive".into(),
                        description: Some("Lighthouse based spacial tracking".into()),
                        key: "libsurvive_enabled".into(),
                        value: p.libsurvive_enabled,
                    });
                    guard.push_back(SwitchModelInit {
                        name: "Basalt".into(),
                        description: Some("Camera based SLAM tracking".into()),
                        key: "basalt_enabled".into(),
                        value: p.basalt_enabled,
                    });
                    guard.push_back(SwitchModelInit {
                        name: "Mercury".into(),
                        description: Some("Camera based hand tracking".into()),
                        key: "mercury_enabled".into(),
                        value: p.mercury_enabled,
                    });
                }

                {
                    let mut guard = self.path_rows.guard();
                    guard.clear();
                    guard.push_back(PathModelInit {
                        name: "XR Service Path".into(),
                        key: "xrservice_path".into(),
                        value: Some(p.xrservice_path),
                    });
                    guard.push_back(PathModelInit {
                        name: "OpenComposite Path".into(),
                        key: "opencomposite_path".into(),
                        value: Some(p.opencomposite_path),
                    });
                    guard.push_back(PathModelInit {
                        name: "Libsurvive Path".into(),
                        key: "libsurvive_path".into(),
                        value: p.libsurvive_path,
                    });
                    guard.push_back(PathModelInit {
                        name: "Basalt Path".into(),
                        key: "basalt_path".into(),
                        value: p.basalt_path,
                    });
                    guard.push_back(PathModelInit {
                        name: "Mercury Path".into(),
                        key: "mercury_path".into(),
                        value: p.mercury_path,
                    });
                    guard.push_back(PathModelInit {
                        name: "Install Prefix".into(),
                        key: "prefix".into(),
                        value: Some(p.prefix),
                    });
                }

                {
                    let mut guard = self.repo_rows.guard();
                    guard.clear();
                    guard.push_back(EntryModelInit {
                        key: "xrservice_repo".into(),
                        name: "XR Service Repo".into(),
                        value: p.xrservice_repo.unwrap_or("".into()),
                    });
                    guard.push_back(EntryModelInit {
                        key: "opencomposite_repo".into(),
                        name: "OpenComposite Repo".into(),
                        value: p.opencomposite_repo.unwrap_or("".into()),
                    });
                    guard.push_back(EntryModelInit {
                        key: "libsurvive_repo".into(),
                        name: "Libsurvive Repo".into(),
                        value: p.libsurvive_repo.unwrap_or("".into()),
                    });
                    guard.push_back(EntryModelInit {
                        key: "basalt_repo".into(),
                        name: "Basalt Repo".into(),
                        value: p.basalt_repo.unwrap_or("".into()),
                    });
                    guard.push_back(EntryModelInit {
                        key: "mercury_repo".into(),
                        name: "Mercury Repo".into(),
                        value: p.mercury_repo.unwrap_or("".into()),
                    });
                }

                self.set_profile(Some(prof.clone()));

                self.win.as_ref().unwrap().present();
            }
            Self::Input::SaveProfile => {
                let prof = self.profile.as_ref().unwrap();
                if prof.validate() {
                    sender.output(ProfileEditorOutMsg::SaveProfile(prof.clone()));
                    self.win.as_ref().unwrap().close();
                } else {
                    self.win.as_ref().unwrap().add_toast(
                        adw::Toast::builder()
                            .title("Profile failed validation")
                            .build(),
                    );
                }
            }
            Self::Input::EnvVarChanged(name, value) => {
                self.profile
                    .as_mut()
                    .unwrap()
                    .environment
                    .insert(name, value);
            }
            Self::Input::EnvVarDelete(name) => {
                self.profile.as_mut().unwrap().environment.remove(&name);
                let pos = self
                    .env_rows
                    .guard()
                    .iter()
                    .position(|evr| evr.name == name);
                if pos.is_some() {
                    self.env_rows.guard().remove(pos.unwrap());
                }
            }
            Self::Input::PathChanged(key, value) => {
                let prof = self.profile.as_mut().unwrap();
                match key.as_str() {
                    "xrservice_path" => prof.xrservice_path = value.unwrap_or("".to_string()),
                    "opencomposite_path" => {
                        prof.opencomposite_path = value.unwrap_or("".to_string())
                    }
                    "libsurvive_path" => prof.libsurvive_path = value,
                    "basalt_path" => prof.basalt_path = value,
                    "mercury_path" => prof.mercury_path = value,
                    "prefix" => prof.prefix = value.unwrap_or("".to_string()),
                    _ => panic!("Unknown profile path key"),
                }
            }
            Self::Input::SwitchChanged(key, value) => {
                let prof = self.profile.as_mut().unwrap();
                match key.as_str() {
                    "libsurvive_enabled" => prof.libsurvive_enabled = value,
                    "basalt_enabled" => prof.basalt_enabled = value,
                    "mercury_enabled" => prof.mercury_enabled = value,
                    "pull_on_build" => prof.pull_on_build = value,
                    _ => panic!("Unknown profile switch key"),
                }
            }
            Self::Input::EntryChanged(key, value) => {
                let prof = self.profile.as_mut().unwrap();
                match key.as_str() {
                    "name" => prof.name = value,
                    "xrservice_repo" => {
                        prof.xrservice_repo = match value.trim() {
                            "" => None,
                            s => Some(s.to_string()),
                        }
                    }
                    "opencomposite_repo" => {
                        prof.opencomposite_repo = match value.trim() {
                            "" => None,
                            s => Some(s.to_string()),
                        }
                    }
                    "libsurvive_repo" => {
                        prof.libsurvive_repo = match value.trim() {
                            "" => None,
                            s => Some(s.to_string()),
                        }
                    }
                    "basalt_repo" => {
                        prof.basalt_repo = match value.trim() {
                            "" => None,
                            s => Some(s.to_string()),
                        }
                    }
                    "mercury_repo" => {
                        prof.mercury_repo = match value.trim() {
                            "" => None,
                            s => Some(s.to_string()),
                        }
                    }
                    _ => panic!("Unknown profile text key"),
                }
            }
            Self::Input::ComboChanged(key, value) => {
                let prof = self.profile.as_mut().unwrap();
                match key.as_str() {
                    "xrservice_type" => prof.xrservice_type = XRServiceType::from_string(value),
                    _ => panic!("Unknown profile text key"),
                }
            }
            Self::Input::AddEnvVar(name) => {
                let prof = self.profile.as_mut().unwrap();
                if !prof.environment.contains_key(&name) {
                    prof.environment.insert(name.clone(), "".to_string());
                    self.env_rows.guard().push_back(EnvVarModelInit {
                        name,
                        value: "".to_string(),
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
        let add_env_popover = gtk::Popover::builder().build();
        let add_env_popover_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .css_classes(["linked"])
            .build();
        let add_env_name_entry = gtk::Entry::builder()
            .placeholder_text("Env Var Name...")
            .build();
        let add_env_btn = gtk::Button::builder()
            .css_classes(["suggested-action"])
            .icon_name("list-add-symbolic")
            .tooltip_text("Add Env Var")
            .build();
        {
            let aeb_sender = sender.clone();
            let entry = add_env_name_entry.clone();
            let popover = add_env_popover.clone();
            add_env_btn.connect_clicked(move |_| {
                let name_gstr = entry.text();
                let name = name_gstr.trim();
                if !name.is_empty() {
                    popover.popdown();
                    entry.set_text("");
                    aeb_sender.input(Self::Input::AddEnvVar(name.to_string()));
                }
            });
        }
        add_env_popover_box.append(&add_env_name_entry);
        add_env_popover_box.append(&add_env_btn);
        add_env_popover.set_child(Some(&add_env_popover_box));

        let add_env_var_btn = gtk::MenuButton::builder()
            .icon_name("list-add-symbolic")
            .tooltip_text("Add Environment Variable")
            .css_classes(["flat"])
            .popover(&add_env_popover)
            .valign(gtk::Align::Start)
            .halign(gtk::Align::End)
            .build();

        let mut model = Self {
            profile: None,
            win: None,
            name_row: adw::EntryRow::builder().title("Name").build(),
            type_row: adw::ComboRow::builder()
                .title("XR Service Type")
                .subtitle(
                    "Monado is for PCVR headsets, while WiVRn is for Andorid standalone headsets",
                )
                .model(&gtk::StringList::new(
                    XRServiceType::iter()
                        .map(|st| st.to_string())
                        .collect::<Vec<String>>()
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<&str>>()
                        .as_slice(),
                ))
                .build(),
            pull_on_build_switch: None,
            env_rows: FactoryVecDeque::new(
                adw::PreferencesGroup::builder()
                    .title("Environment Variables")
                    .description(env_var_descriptions_as_paragraph())
                    .header_suffix(&add_env_var_btn)
                    .build(),
                sender.input_sender(),
            ),
            switch_rows: FactoryVecDeque::new(
                adw::PreferencesGroup::builder()
                    .title("Components")
                    .description("Enable or disable features")
                    .build(),
                sender.input_sender(),
            ),
            path_rows: FactoryVecDeque::new(
                adw::PreferencesGroup::builder()
                    .title("Paths")
                    .description(concat!(
                        "Where the various components' repositories will be cloned to.\n\n",
                        "\"Install Prefix\" is the path where the components are ",
                        "installed after building."
                    ))
                    .build(),
                sender.input_sender(),
            ),
            repo_rows: FactoryVecDeque::new(
                adw::PreferencesGroup::builder()
                    .title("Repositories")
                    .description(concat!(
                        "Change the repositories from which various components are pulled.\n\n",
                        "Leave empty to use the default repository."
                    ))
                    .build(),
                sender.input_sender(),
            ),
            tracker: 0,
        };

        {
            let name_sender = sender.clone();
            model.name_row.connect_changed(move |nr| {
                name_sender.input(Self::Input::EntryChanged(
                    "name".to_string(),
                    nr.text().to_string(),
                ));
            });
        }

        {
            let type_sender = sender.clone();
            model.type_row.connect_selected_item_notify(move |tr| {
                type_sender.input(Self::Input::ComboChanged(
                    "xrservice_type".to_string(),
                    match tr.selected() {
                        0 => "monado".to_string(),
                        1 => "wivrn".to_string(),
                        _ => panic!("XRServiceType combo row cannot have more than 2 choices"),
                    },
                ));
            });
        }

        let widgets = view_output!();
        model.win = Some(widgets.win.clone());
        model.pull_on_build_switch = Some(widgets.pull_on_build_switch.clone());

        ComponentParts { model, widgets }
    }
}
