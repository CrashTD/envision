use super::factories::{
    entry_row_factory::{EntryModel, EntryModelInit},
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
use relm4_icons::icon_name;

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
    env_rows: FactoryVecDeque<EntryModel>,
    #[tracker::do_not_track]
    switch_rows: FactoryVecDeque<SwitchModel>,
    #[tracker::do_not_track]
    path_rows: FactoryVecDeque<PathModel>,
}

#[derive(Debug)]
pub enum ProfileEditorMsg {
    Present(Profile),
    EntryChanged(String, String),
    TextChanged(String, String),
    PathChanged(String, Option<String>),
    SwitchChanged(String, bool),
    ComboChanged(String, String),
    AddEnvVar,
    SaveProfile,
}

#[derive(Debug)]
pub enum ProfileEditorOutMsg {
    SaveProfile(Profile),
}

pub struct ProfileEditorInit {}

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
            #[track = "model.changed(Self::profile())"]
            set_title: match model.profile.as_ref() {
                Some(p) => Some(p.name.as_str()),
                None => None,
            },
            add: mainpage = &adw::PreferencesPage {
                add: maingrp = &adw::PreferencesGroup {
                    set_title: "Profile Info",
                    model.name_row.clone(),
                    model.type_row.clone(),
                },
                add: model.env_rows.widget(),
                add: model.switch_rows.widget(),
                add: model.path_rows.widget(),
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

                self.env_rows.guard().clear();
                for (k, v) in p.environment.iter() {
                    self.env_rows.guard().push_back(EntryModelInit {
                        name: k.clone(),
                        value: v.clone(),
                    });
                }

                self.switch_rows.guard().clear();
                self.switch_rows.guard().push_back(SwitchModelInit {
                    name: "Libsurvive".into(),
                    description: Some("Lighthouse based spacial tracking".into()),
                    key: "libsurvive_enabled".into(),
                    value: p.libsurvive_enabled,
                });
                self.switch_rows.guard().push_back(SwitchModelInit {
                    name: "Basalt".into(),
                    description: Some("Camera based SLAM tracking".into()),
                    key: "basalt_enabled".into(),
                    value: p.basalt_enabled,
                });
                self.switch_rows.guard().push_back(SwitchModelInit {
                    name: "Mercury".into(),
                    description: Some("Camera based hand tracking".into()),
                    key: "mercury_enabled".into(),
                    value: p.mercury_enabled,
                });

                self.path_rows.guard().clear();
                self.path_rows.guard().push_back(PathModelInit {
                    name: "XR Service Path".into(),
                    key: "xrservice_path".into(),
                    value: Some(p.xrservice_path),
                });
                self.path_rows.guard().push_back(PathModelInit {
                    name: "OpenComposite Path".into(),
                    key: "opencomposite_path".into(),
                    value: Some(p.opencomposite_path),
                });
                self.path_rows.guard().push_back(PathModelInit {
                    name: "Libsurvive Path".into(),
                    key: "libsurvive_path".into(),
                    value: p.libsurvive_path,
                });
                self.path_rows.guard().push_back(PathModelInit {
                    name: "Basalt Path".into(),
                    key: "basalt_path".into(),
                    value: p.basalt_path,
                });
                self.path_rows.guard().push_back(PathModelInit {
                    name: "Mercury Path".into(),
                    key: "mercury_path".into(),
                    value: p.mercury_path,
                });
                self.path_rows.guard().push_back(PathModelInit {
                    name: "Install Prefix".into(),
                    key: "prefix".into(),
                    value: Some(p.prefix),
                });

                self.set_profile(Some(prof.clone()));

                self.win.as_ref().unwrap().present();
            }
            Self::Input::SaveProfile => {
                let prof = self.profile.as_ref().unwrap();
                if prof.validate() {
                    sender.output(ProfileEditorOutMsg::SaveProfile(prof.clone()));
                    self.win.as_ref().unwrap().close();
                } else {
                    self.win.as_ref().unwrap().add_toast(adw::Toast::builder().title("Profile failed validation").build());
                }
            }
            // TODO: rename to reflect this is only for env
            // + make entryfactory take a signal to send to parent
            Self::Input::EntryChanged(name, value) => {
                self.profile
                    .as_mut()
                    .unwrap()
                    .environment
                    .insert(name, value);
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
                    _ => panic!("Unknown profile switch key"),
                }
            }
            Self::Input::TextChanged(key, value) => {
                let prof = self.profile.as_mut().unwrap();
                match key.as_str() {
                    "name" => prof.name = value,
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
            Self::Input::AddEnvVar => {
                println!("Add env var");
            }
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let add_env_var_btn = gtk::Button::builder()
            .icon_name(icon_name::PLUS)
            .tooltip_text("Add Environment Variable")
            .css_classes(["flat"])
            .valign(gtk::Align::Start)
            .halign(gtk::Align::End)
            .build();
        {
            let btn_sender = sender.clone();
            add_env_var_btn.connect_clicked(move |_| {
                btn_sender.input(Self::Input::AddEnvVar);
            });
        }

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
                    // .description("")
                    .build(),
                sender.input_sender(),
            ),
            tracker: 0,
        };

        {
            let name_sender = sender.clone();
            model.name_row.connect_changed(move |nr| {
                name_sender.input(Self::Input::TextChanged(
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

        ComponentParts { model, widgets }
    }
}
