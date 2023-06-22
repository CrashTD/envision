use super::factories::{entry_row_factory::{EntryModel, EntryModelInit}, switch_row_factory::{SwitchModel, SwitchModelInit}, path_row_factory::{PathModel, PathModelInit}};
use crate::{env_var_descriptions::env_var_descriptions_as_paragraph, profile::Profile};
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
    AddEnvVar,
    SaveProfile, // ?
}

pub struct ProfileEditorInit {}

#[relm4::component(pub)]
impl SimpleComponent for ProfileEditor {
    type Init = ProfileEditorInit;
    type Input = ProfileEditorMsg;
    type Output = ();

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
                },
                add: model.env_rows.widget(),
                add: model.switch_rows.widget(),
                add: model.path_rows.widget(),
            }
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        self.reset();

        match message {
            Self::Input::Present(prof) => {
                let p = prof.clone();

                self.name_row.set_text(p.name.as_str());

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
                    value: Some(p.xrservice_path)
                });
                self.path_rows.guard().push_back(PathModelInit {
                    name: "OpenComposite Path".into(),
                    key: "opencomposite_path".into(),
                    value: Some(p.opencomposite_path)
                });
                self.path_rows.guard().push_back(PathModelInit {
                    name: "Libsurvive Path".into(),
                    key: "libsurvive_path".into(),
                    value: p.libsurvive_path
                });
                self.path_rows.guard().push_back(PathModelInit {
                    name: "Basalt Path".into(),
                    key: "basalt_path".into(),
                    value: p.basalt_path
                });
                self.path_rows.guard().push_back(PathModelInit {
                    name: "Mercury Path".into(),
                    key: "mercury_path".into(),
                    value: p.mercury_path
                });

                self.set_profile(Some(prof.clone()));

                self.win.as_ref().unwrap().present();
            }
            Self::Input::SaveProfile => {}
            Self::Input::EntryChanged(name, value) => {
                println!("{}: {}", name, value);
                self.profile
                    .as_mut()
                    .unwrap()
                    .environment
                    .insert(name, value);
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

        let widgets = view_output!();
        model.win = Some(widgets.win.clone());

        ComponentParts { model, widgets }
    }
}
