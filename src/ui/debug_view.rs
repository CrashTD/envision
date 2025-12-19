#![allow(unused_assignments)] // vars named in view! macro are not correctly recognized as used

use super::{
    app::{DebugCopyEnvVarsAction, DebugOpenDataAction, DebugOpenPrefixAction},
    term_widget::TermWidget,
    util::copy_text,
};
use crate::{
    config::Config,
    log_parser::{LogLevel, MonadoLog},
    paths::{get_data_dir, get_home_dir},
    profile::Profile,
    stateless_action,
    ui::{
        SENDER_IO_ERR_MSG,
        alert::alert,
        app::{BuildProfileAction, BuildProfileCleanAction},
        profile_editor::{ProfileEditor, ProfileEditorInit, ProfileEditorMsg, ProfileEditorOutMsg},
        util::limit_dropdown_width,
    },
    util::file_utils::get_writer,
};
use adw::{ResponseAppearance, prelude::*};
use gtk::glib::clone;
use relm4::{
    actions::{ActionGroupName, RelmAction, RelmActionGroup},
    new_action_group, new_stateless_action,
    prelude::*,
};
use std::{fs::read_to_string, io::Write};

#[derive(Debug)]
pub enum SearchDirection {
    Forward,
    Backward,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum DebugViewMsg {
    LogUpdated(Vec<String>),
    ClearLog,
    DoSearch(Option<String>),
    SearchFindMatch(SearchDirection),
    LogLevelChanged(LogLevel),
    XRServiceActiveChanged(bool),
    UpdateSelectedProfile(Profile),
    CopyProfileId,
    SetColorScheme,
    ProfileSelected(u32),
    UpdateProfiles(Vec<Profile>, Config),
    SetSelectedProfile(u32),
    EditProfile,
    CreateProfile,
    DeleteProfile,
    SaveProfile(Profile),
    DuplicateProfile,
    ExportProfile,
    ImportProfile,
    OpenProfileEditor(Profile),
    QueryProfileRebuild,
    ProfilesEnabledChanged(bool),
}

#[derive(Debug)]
pub enum DebugViewOutMsg {
    StartWithDebug,
    ProfileSelected(Profile),
    /// bool param: delete files
    DeleteProfile(bool),
    SaveProfile(Profile),
    /// params: clean
    BuildProfile(bool),
    ProfilesEnabledChanged(bool),
}

#[tracker::track]
pub struct DebugView {
    xrservice_active: bool,
    #[tracker::do_not_track]
    dropdown: Option<gtk::DropDown>,
    #[tracker::do_not_track]
    log_level: LogLevel,
    #[tracker::do_not_track]
    term: TermWidget,
    #[tracker::do_not_track]
    profile_id_widget: gtk::Button,
    #[tracker::do_not_track]
    profiles_dropdown: Option<gtk::DropDown>,
    #[tracker::do_not_track]
    profile_delete_action: gtk::gio::SimpleAction,
    #[tracker::do_not_track]
    profile_export_action: gtk::gio::SimpleAction,
    #[tracker::do_not_track]
    profile_editor: Option<Controller<ProfileEditor>>,
    #[no_eq]
    profile: Profile,
    profiles: Vec<Profile>,
    #[tracker::do_not_track]
    root_win: gtk::Window,

    profiles_enabled: bool,
}

pub struct DebugViewInit {
    pub profile: Profile,
    pub root_win: gtk::Window,
    /// grab this from Config
    pub profiles_enabled: bool,
}

impl DebugView {
    fn create_profile_editor(&mut self, sender: AsyncComponentSender<Self>, prof: Profile) {
        self.profile_editor = Some(
            ProfileEditor::builder()
                .launch(ProfileEditorInit {
                    root_win: self.root_win.clone(),
                    profile: prof,
                })
                .forward(sender.input_sender(), |message| match message {
                    ProfileEditorOutMsg::SaveProfile(p) => DebugViewMsg::SaveProfile(p),
                }),
        );
    }
}

#[relm4::component(pub async)]
impl AsyncComponent for DebugView {
    type Init = DebugViewInit;
    type Input = DebugViewMsg;
    type Output = DebugViewOutMsg;
    type CommandOutput = ();

    menu! {
        debug_menu: {
            section! {
                custom: "profile_id_widget",
                "_Build profile" => BuildProfileAction,
                "C_lean build profile" => BuildProfileCleanAction,
            },
            section! {
                "Open _Data Folder" => DebugOpenDataAction,
                "Open _Prefix Folder" => DebugOpenPrefixAction,
                "Copy Profile _Environment Variables" => DebugCopyEnvVarsAction,
            },
        },
        profile_actions_menu: {
            section! {
                "_New profile" => ProfileMenuNewAction,
                "_Edit profile" => ProfileMenuEditAction,
                "Du_plicate profile" => ProfileMenuDuplicateAction,
                "_Delete profile" => ProfileMenuDeleteAction,
            },
            section! {
                "_Import profile" => ProfileMenuImportAction,
                "E_xport profile" => ProfileMenuExportAction,
            },
        }
    }

    view! {
        adw::ToolbarView {
            set_hexpand: true,
            set_vexpand: true,
            add_top_bar: hb = &adw::HeaderBar {
                set_hexpand: true,
                set_vexpand: false,
                add_css_class: "flat",
                pack_end: debug_menu_btn = &gtk::MenuButton {
                    set_icon_name: "view-more-symbolic",
                    set_tooltip_text: Some("Debug Actions"),
                    set_menu_model: Some(&debug_menu),
                },
                pack_end: search_toggle = &gtk::ToggleButton {
                    set_icon_name: "edit-find-symbolic",
                    set_tooltip_text: Some("Filter Log"),
                },
                pack_start: &log_level_dropdown,
                pack_start: run_debug_btn = &gtk::Button {
                    set_icon_name: "run-debug-symbolic",
                    set_tooltip_text: Some("Start with gdbserver"),
                    connect_clicked[sender] => move |_| {
                        sender.output(Self::Output::StartWithDebug)
                            .expect(SENDER_IO_ERR_MSG);
                    },
                    #[track = "model.changed(Self::xrservice_active())"]
                    set_sensitive: !model.xrservice_active
                },
            },
            add_top_bar: _searchbar = &gtk::SearchBar {
                set_margin_start: 1,
                set_hexpand: true,
                #[chain(flags(gtk::glib::BindingFlags::BIDIRECTIONAL).build())]
                bind_property: ("search-mode-enabled", &search_toggle, "active"),
                connect_search_mode_enabled_notify[sender] => move |searchbar| {
                    if !searchbar.is_search_mode() {
                        sender.input(Self::Input::DoSearch(None));
                    }
                },
                #[wrap(Some)]
                set_child: searchbox = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    add_css_class: "linked",
                    set_hexpand: true,
                    #[name(search_entry)]
                    gtk::SearchEntry {
                        set_hexpand: true,
                        connect_changed[sender] => move |e| {
                            let txt = e.text().to_string();
                            sender.input(Self::Input::DoSearch(
                                if txt.is_empty() { None } else { Some(txt) }
                            ));
                        },
                        connect_activate[sender] => move |_| {
                            sender.input(Self::Input::SearchFindMatch(SearchDirection::Forward));
                        },
                    },
                    gtk::Button {
                        set_icon_name: "go-up-symbolic",
                        set_tooltip_text: Some("Previous Match"),
                        connect_clicked[sender] => move |_| {
                            sender.input(Self::Input::SearchFindMatch(SearchDirection::Backward))
                        },
                    },
                    gtk::Button {
                        set_icon_name: "go-down-symbolic",
                        set_tooltip_text: Some("Next Match"),
                        connect_clicked[sender] => move |_| {
                            sender.input(Self::Input::SearchFindMatch(SearchDirection::Forward))
                        },
                    },
                },
                connect_entry: &search_entry,
            },
            add_bottom_bar: prof_toggle_box = &gtk::ListBox{
                set_selection_mode: gtk::SelectionMode::None,
                add_css_class: "background",
                adw::SwitchRow {
                    set_title: "Use development profiles",
                    #[track = "model.changed(Self::profiles_enabled())"]
                    set_active: model.profiles_enabled,
                    #[track = "model.changed(Self::xrservice_active())"]
                    set_sensitive: !model.xrservice_active,

                    connect_active_notify[sender] => move |row| {
                        sender.input(Self::Input::ProfilesEnabledChanged(row.is_active()));
                    }
                }
            },
            add_bottom_bar: bottom_bar = &gtk::Revealer {
                #[track = "model.changed(Self::profiles_enabled())"]
                set_reveal_child: model.profiles_enabled,
                gtk::Box {
                    set_margin_all: 12,
                    set_orientation: gtk::Orientation::Horizontal,
                    add_css_class: "linked",
                    #[name(profiles_dropdown)]
                    gtk::DropDown {
                        set_hexpand: true,
                        #[track = "model.changed(Self::profile())"]
                        set_tooltip_text: Some(format!("Profile: {}", model.profile).as_str()),
                        #[track = "model.changed(Self::profiles())"]
                        set_model: Some(&{
                            let names: Vec<_> = model.profiles.iter().map(|p| p.name.as_str()).collect();
                            gtk::StringList::new(&names)
                        }),
                        connect_selected_item_notify[sender] => move |dd| {
                            sender.input(Self::Input::ProfileSelected(dd.selected()));
                        },
                        connect_realize => move |dd| {
                            limit_dropdown_width(
                                dd,
                            );
                        },
                    },
                    gtk::MenuButton {
                        set_icon_name: "view-more-symbolic",
                        set_tooltip_text: Some("Menu"),
                        set_menu_model: Some(&profile_actions_menu),
                        set_direction: gtk::ArrowType::Up,
                    },
                },
            },
            set_content: Some(&model.term.container),
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
            Self::Input::XRServiceActiveChanged(active) => {
                self.set_xrservice_active(active);
                if let Some(dropdown) = self.dropdown.as_ref() {
                    dropdown.set_sensitive(!active);
                }
            }
            Self::Input::LogLevelChanged(lvl) => {
                self.log_level = lvl;
            }
            Self::Input::DoSearch(txt) => {
                self.term.set_search_term(txt.as_deref());
            }
            Self::Input::SearchFindMatch(direction) => match direction {
                SearchDirection::Forward => {
                    self.term.search_next();
                }
                SearchDirection::Backward => {
                    self.term.search_prev();
                }
            },
            Self::Input::LogUpdated(n_log) => {
                for row in n_log {
                    let txt = match MonadoLog::new_from_str(row.as_str()) {
                        Some(o) => match o.level >= self.log_level {
                            false => None,
                            true => Some(format!(
                                "{lvl}\t[{file}:{func}]\r\n\t{msg}\r\n",
                                lvl = o.level.colored(),
                                file = o.file,
                                func = o.func,
                                msg = o.message.replace('\n', "\r\n")
                            )),
                        },
                        None => Some(row),
                    };
                    if let Some(t) = txt {
                        self.term.feed(&t);
                    }
                }
            }
            Self::Input::ClearLog => {
                self.term.clear();
            }
            Self::Input::SetColorScheme => {
                self.term.set_color_scheme();
            }
            Self::Input::UpdateSelectedProfile(prof) => {
                self.set_profile(prof.clone());
                self.profile_id_widget
                    .set_label(&format!("Profile id: {}", self.profile.uuid));
                self.profile_id_widget.set_visible(!prof.is_system_profile);
                self.profile_delete_action.set_enabled(prof.editable);
                self.profile_export_action.set_enabled(prof.editable);
            }
            Self::Input::CopyProfileId => {
                copy_text(&self.profile.uuid);
            }
            Self::Input::ProfileSelected(position) => {
                if self.profiles_enabled {
                    sender
                        .output(Self::Output::ProfileSelected(
                            self.profiles.get(position as usize).unwrap().clone(),
                        ))
                        .expect("Sender output failed");
                }
            }
            Self::Input::UpdateProfiles(profiles, config) => {
                self.set_profiles(profiles);
                if config.profiles_enabled {
                    // why send another message to set the dropdown selection?
                    // set_* from tracker likely updates the view obj in the next
                    // draw, so selecting here will result in nothing cause the
                    // dropdown is effectively empty
                    sender.input(Self::Input::SetSelectedProfile({
                        match self
                            .profiles
                            .iter()
                            .position(|p| p.uuid == config.selected_profile_uuid)
                        {
                            Some(idx) => idx as u32,
                            None => 0,
                        }
                    }));
                }
            }
            Self::Input::SetSelectedProfile(index) => {
                if let Some(dd) = self.profiles_dropdown.as_ref() {
                    dd.set_selected(index);
                    sender.input(Self::Input::UpdateSelectedProfile(
                        self.profiles.get(index as usize).unwrap().clone(),
                    ));
                }
            }
            Self::Input::EditProfile => {
                if self.profile.editable {
                    sender.input(Self::Input::OpenProfileEditor(self.profile.clone()));
                } else {
                    let dialog = adw::AlertDialog::builder()
                        .heading("This profile is not editable")
                        .body(concat!(
                            "You can duplicate it and edit the new copy. ",
                            "Do you want to duplicate the current profile?"
                        ))
                        .build();
                    dialog.add_response("no", "_No");
                    dialog.add_response("yes", "_Yes");
                    dialog.set_response_appearance("yes", ResponseAppearance::Suggested);

                    dialog.connect_response(
                        None,
                        clone!(
                            #[strong]
                            sender,
                            move |_, res| {
                                if res == "yes" {
                                    sender.input(Self::Input::DuplicateProfile);
                                }
                            }
                        ),
                    );
                    dialog.present(Some(&self.root_win));
                }
            }
            Self::Input::CreateProfile => {
                sender.input(Self::Input::OpenProfileEditor(Profile::default()));
            }
            Self::Input::DeleteProfile => {
                let delete_confirm_dialog = adw::AlertDialog::builder()
                    .heading("Are you sure you want to delete this profile?")
                    .extra_child(
                        &gtk::CheckButton::builder()
                            .label("Delete all files and folders associated with profile")
                            .halign(gtk::Align::Center)
                            .hexpand(true)
                            .build(),
                    )
                    .build();
                delete_confirm_dialog.add_response("no", "_No");
                delete_confirm_dialog.add_response("yes", "_Yes");
                delete_confirm_dialog
                    .set_response_appearance("yes", ResponseAppearance::Destructive);

                delete_confirm_dialog.connect_response(
                    None,
                    clone!(
                        #[strong]
                        sender,
                        move |dialog, res| {
                            let delete_files_checkbox = dialog
                                .extra_child()
                                .and_then(|child| child.downcast::<gtk::CheckButton>().ok());
                            let delete_files = delete_files_checkbox
                                .as_ref()
                                .is_some_and(|c| c.is_active());
                            if let Some(check) = delete_files_checkbox {
                                check.set_active(false);
                            }
                            if res == "yes" {
                                sender
                                    .output(Self::Output::DeleteProfile(delete_files))
                                    .expect("Sender output failed");
                            }
                        }
                    ),
                );
                delete_confirm_dialog.present(Some(&self.root_win));
            }
            Self::Input::SaveProfile(prof) => {
                sender
                    .output(Self::Output::SaveProfile(prof))
                    .expect(SENDER_IO_ERR_MSG);
            }
            Self::Input::DuplicateProfile => {
                if !self.profile.is_system_profile
                    && let Some(dup) = self.profile.create_duplicate()
                {
                    sender.input(Self::Input::OpenProfileEditor(dup));
                } else {
                    alert(
                        "This profile cannot be duplicated",
                        None,
                        Some(&self.root_win),
                    );
                }
            }
            Self::Input::ExportProfile => {
                let prof = self.profile.clone();
                if !prof.editable {
                    return;
                }
                let chooser = gtk::FileDialog::builder()
                    .modal(true)
                    .title("Choose a location for the exported profile")
                    .initial_name(format!("{}.json", &prof.name))
                    .accept_label("Export")
                    .build();
                let root_win = self.root_win.clone();
                chooser.save(
                    Some(&self.root_win),
                    gtk::gio::Cancellable::NONE,
                    move |res| {
                        if let Ok(file) = res {
                            if let Some(path) = file.path()
                                && let Ok(mut writer) = get_writer(&path)
                                && let Ok(s) = serde_json::to_string_pretty(&prof)
                            {
                                let prof_id = prof.uuid;
                                let s = s
                                    .replace(get_data_dir().to_string_lossy().as_ref(), "@DATADIR@")
                                    .replace(get_home_dir().to_string_lossy().as_ref(), "@HOMEDIR@")
                                    .replace(&prof_id, "@UUID@");
                                if writer.write_all(s.as_bytes()).is_ok() {
                                    return;
                                }
                            }
                            alert("Failed to export profile", None, Some(&root_win));
                        }
                    },
                );
            }
            Self::Input::ImportProfile => {
                let confirm_dialog = adw::AlertDialog::builder()
                    .heading("Importing profiles is dangerous!")
                    .body_use_markup(true)
                    .body("Importing a profile from an <b>untrusted source</b> is dangerous and could lead to <b>arbitrary code execution</b>.\n\nInspect the profile JSON manually before importing it and make sure it doesn't contain anything suspicious, including references to untrusted forks.")
                    .build();
                confirm_dialog.add_response("no", "_Cancel");
                confirm_dialog.add_response("yes", "I understand the risk, continue");
                confirm_dialog.set_response_appearance("yes", ResponseAppearance::Destructive);

                let root_win = self.root_win.clone();
                let fd_sender = sender.clone();
                confirm_dialog.connect_response(None, move |_, res| {
                    if res != "yes" {
                        return;
                    }
                    let root_win = root_win.clone();
                    let fd_sender = fd_sender.clone();
                    let chooser = gtk::FileDialog::builder()
                        .modal(true)
                        .title("Choose a profile to import")
                        .accept_label("Import")
                        .filters(&{
                            let filter = gtk::gio::ListStore::builder()
                                .item_type(gtk::FileFilter::static_type())
                                .build();
                            filter.append(&{
                                let f = gtk::FileFilter::new();
                                f.add_mime_type("application/json");
                                f
                            });
                            filter
                        })
                        .build();
                    chooser.open(
                        Some(&root_win.clone()),
                        gtk::gio::Cancellable::NONE,
                        move |res| {
                            if let Ok(file) = res {
                                if let Some(path) = file.path()
                                    && let Ok(s) = read_to_string(path)
                                {
                                    let s = s
                                        .replace(
                                            "@DATADIR@",
                                            get_data_dir().to_string_lossy().as_ref(),
                                        )
                                        .replace(
                                            "@HOMEDIR@",
                                            get_home_dir().to_string_lossy().as_ref(),
                                        )
                                        .replace("@UUID@", &Profile::new_uuid());
                                    if let Ok(nprof) = serde_json::from_str::<Profile>(&s) {
                                        fd_sender.input(Self::Input::OpenProfileEditor(nprof));
                                        return;
                                    }
                                }
                                alert("Failed to import profile", None, Some(&root_win));
                            }
                        },
                    );
                });
                confirm_dialog.present(Some(&self.root_win));
            }
            Self::Input::OpenProfileEditor(profile) => {
                self.create_profile_editor(sender, profile);
                self.profile_editor
                    .as_ref()
                    .unwrap()
                    .sender()
                    .emit(ProfileEditorMsg::Present);
            }
            Self::Input::QueryProfileRebuild => {
                let rebuild_dialog = adw::AlertDialog::builder()
                    .heading("Do you want to build this profile now?")
                    .body("This will trigger a clean build")
                    .build();
                rebuild_dialog.add_response("no", "_No");
                rebuild_dialog.add_response("yes", "_Yes");
                rebuild_dialog.set_response_appearance("yes", ResponseAppearance::Suggested);

                rebuild_dialog.connect_response(
                    None,
                    clone!(
                        #[strong]
                        sender,
                        move |_, res| {
                            if res == "yes" {
                                sender
                                    .output(Self::Output::BuildProfile(true))
                                    .expect(SENDER_IO_ERR_MSG);
                            }
                        }
                    ),
                );
                rebuild_dialog.present(Some(&self.root_win));
            }
            Self::Input::ProfilesEnabledChanged(enabled) => {
                self.set_profiles_enabled(enabled);
                self.profile_id_widget.set_visible(enabled);
                sender
                    .output(Self::Output::ProfilesEnabledChanged(enabled))
                    .expect(SENDER_IO_ERR_MSG);
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let log_level_dropdown = gtk::DropDown::from_strings(
            LogLevel::iter()
                .map(|lvl| lvl.to_string())
                .collect::<Vec<String>>()
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>()
                .as_slice(),
        );
        if let Some(btn) = log_level_dropdown.first_child() {
            btn.add_css_class("flat");
        }
        log_level_dropdown.connect_selected_notify(clone!(
            #[strong]
            sender,
            move |dd| {
                sender.input(Self::Input::LogLevelChanged(
                    *LogLevel::iter()
                        .as_slice()
                        .get(dd.selected() as usize)
                        .unwrap(),
                ));
            }
        ));

        adw::StyleManager::default().connect_dark_notify(clone!(
            #[strong]
            sender,
            move |_| {
                sender.input(Self::Input::SetColorScheme);
            }
        ));

        let mut actions = RelmActionGroup::<ProfileActionGroup>::new();

        let profile_delete_action = {
            let action = RelmAction::<ProfileMenuDeleteAction>::new_stateless(clone!(
                #[strong]
                sender,
                move |_| {
                    sender.input(Self::Input::DeleteProfile);
                }
            ));
            let ret = action.gio_action().clone();
            actions.add_action(action);
            ret.set_enabled(false);
            ret
        };
        let profile_export_action = {
            let action = RelmAction::<ProfileMenuExportAction>::new_stateless(clone!(
                #[strong]
                sender,
                move |_| {
                    sender.input(Self::Input::ExportProfile);
                }
            ));
            let ret = action.gio_action().clone();
            actions.add_action(action);
            ret.set_enabled(false);
            ret
        };

        stateless_action!(
            actions,
            ProfileMenuNewAction,
            clone!(
                #[strong]
                sender,
                move |_| {
                    sender.input(Self::Input::CreateProfile);
                }
            )
        );
        stateless_action!(
            actions,
            ProfileMenuEditAction,
            clone!(
                #[strong]
                sender,
                move |_| {
                    sender.input(Self::Input::EditProfile);
                }
            )
        );
        stateless_action!(
            actions,
            ProfileMenuDuplicateAction,
            clone!(
                #[strong]
                sender,
                move |_| {
                    sender.input(Self::Input::DuplicateProfile);
                }
            )
        );
        stateless_action!(
            actions,
            ProfileMenuImportAction,
            clone!(
                #[strong]
                sender,
                move |_| {
                    sender.input(Self::Input::ImportProfile);
                }
            )
        );

        root.insert_action_group(ProfileActionGroup::NAME, Some(&actions.into_action_group()));

        let mut model = Self {
            xrservice_active: false,
            tracker: 0,
            dropdown: None,
            log_level: LogLevel::Trace,
            term: TermWidget::new(),
            profile_id_widget: gtk::Button::builder()
                .css_classes(["flat", "dim-label", "caption"])
                .label(format!("Profile id: {}", init.profile.uuid))
                .tooltip_text("Copy")
                .visible(init.profiles_enabled)
                .build(),
            profile: init.profile,
            profiles: Vec::new(),
            profiles_dropdown: None,
            profile_delete_action,
            profile_export_action,
            profile_editor: None,
            root_win: init.root_win,
            profiles_enabled: init.profiles_enabled,
        };
        model.term.set_color_scheme();

        model.profile_id_widget.connect_clicked(clone!(
            #[strong]
            sender,
            move |_| {
                sender.input(Self::Input::CopyProfileId);
            }
        ));

        let widgets = view_output!();
        model.dropdown = Some(log_level_dropdown.clone());

        model.profiles_dropdown = Some(widgets.profiles_dropdown.clone());

        widgets
            .debug_menu_btn
            .popover()
            .clone()
            .unwrap()
            .downcast::<gtk::PopoverMenu>()
            .unwrap()
            .add_child(&model.profile_id_widget, "profile_id_widget");

        AsyncComponentParts { model, widgets }
    }
}

new_action_group!(ProfileActionGroup, "profile");
new_stateless_action!(
    ProfileMenuNewAction,
    ProfileActionGroup,
    "profilemenunewaction"
);
new_stateless_action!(
    ProfileMenuEditAction,
    ProfileActionGroup,
    "profilemenueditaction"
);
new_stateless_action!(
    ProfileMenuDuplicateAction,
    ProfileActionGroup,
    "profilemenuduplicateaction"
);
new_stateless_action!(
    ProfileMenuDeleteAction,
    ProfileActionGroup,
    "profilemenudeleteaction"
);
new_stateless_action!(
    ProfileMenuImportAction,
    ProfileActionGroup,
    "profilemenuimportaction"
);
new_stateless_action!(
    ProfileMenuExportAction,
    ProfileActionGroup,
    "profilemenuexportaction"
);
