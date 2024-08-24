use super::{
    alert::alert,
    app::{
        AboutAction, BuildProfileAction, BuildProfileCleanAction, ConfigureWivrnAction,
        DebugViewToggleAction,
    },
    devices_box::{DevicesBox, DevicesBoxMsg},
    install_wivrn_box::{InstallWivrnBox, InstallWivrnBoxInit, InstallWivrnBoxMsg},
    profile_editor::{ProfileEditor, ProfileEditorInit, ProfileEditorMsg, ProfileEditorOutMsg},
    steam_launch_options_box::{SteamLaunchOptionsBox, SteamLaunchOptionsBoxMsg},
    steamvr_calibration_box::{SteamVrCalibrationBox, SteamVrCalibrationBoxMsg},
    util::{limit_dropdown_width, warning_heading},
};
use crate::{
    config::Config,
    dependencies::common::dep_pkexec,
    file_utils::{get_writer, mount_has_nosuid},
    gpu_profile::{get_amd_gpu_power_profile, GpuPowerProfile},
    paths::{get_data_dir, get_home_dir},
    profile::{LighthouseDriver, Profile, XRServiceType},
    stateless_action,
    steamvr_utils::chaperone_info_exists,
    xr_devices::XRDevice,
};
use adw::{prelude::*, ResponseAppearance};
use gtk::glib::clone;
use relm4::{
    actions::{ActionGroupName, RelmAction, RelmActionGroup},
    new_action_group, new_stateless_action,
    prelude::*,
    ComponentParts, ComponentSender, SimpleComponent,
};
use std::{fs::read_to_string, io::Write};

#[tracker::track]
pub struct MainView {
    xrservice_active: bool,
    enable_debug_view: bool,
    profiles: Vec<Profile>,
    #[no_eq]
    selected_profile: Profile,
    #[tracker::do_not_track]
    profiles_dropdown: Option<gtk::DropDown>,
    #[tracker::do_not_track]
    install_wivrn_box: AsyncController<InstallWivrnBox>,
    #[tracker::do_not_track]
    steam_launch_options_box: Controller<SteamLaunchOptionsBox>,
    #[tracker::do_not_track]
    devices_box: Controller<DevicesBox>,
    #[tracker::do_not_track]
    profile_not_editable_dialog: adw::AlertDialog,
    #[tracker::do_not_track]
    profile_delete_confirm_dialog: adw::AlertDialog,
    #[tracker::do_not_track]
    profile_editor: Option<Controller<ProfileEditor>>,
    #[tracker::do_not_track]
    steamvr_calibration_box: Controller<SteamVrCalibrationBox>,
    #[tracker::do_not_track]
    root_win: gtk::Window,
    #[tracker::do_not_track]
    profile_delete_action: gtk::gio::SimpleAction,
    #[tracker::do_not_track]
    profile_export_action: gtk::gio::SimpleAction,
    xrservice_ready: bool,
}

#[derive(Debug)]
pub enum MainViewMsg {
    ClockTicking,
    StartStopClicked,
    RestartXRService,
    XRServiceActiveChanged(bool, Option<Profile>, bool),
    EnableDebugViewChanged(bool),
    UpdateProfiles(Vec<Profile>, Config),
    SetSelectedProfile(u32),
    ProfileSelected(u32),
    UpdateSelectedProfile(Profile),
    EditProfile,
    CreateProfile,
    DeleteProfile,
    DuplicateProfile,
    SaveProfile(Profile),
    UpdateDevices(Vec<XRDevice>),
    UpdateXrServiceReady(bool),
    ExportProfile,
    ImportProfile,
    OpenProfileEditor(Profile),
}

#[derive(Debug)]
pub enum MainViewOutMsg {
    DoStartStopXRService,
    RestartXRService,
    ProfileSelected(Profile),
    DeleteProfile,
    SaveProfile(Profile),
    OpenLibsurviveSetup,
}

pub struct MainViewInit {
    pub config: Config,
    pub selected_profile: Profile,
    pub root_win: gtk::Window,
}

impl MainView {
    fn create_profile_editor(&mut self, sender: ComponentSender<MainView>, prof: Profile) {
        self.profile_editor = Some(
            ProfileEditor::builder()
                .launch(ProfileEditorInit {
                    root_win: self.root_win.clone(),
                    profile: prof,
                })
                .forward(sender.input_sender(), |message| match message {
                    ProfileEditorOutMsg::SaveProfile(p) => MainViewMsg::SaveProfile(p),
                }),
        );
    }
}

#[relm4::component(pub)]
impl SimpleComponent for MainView {
    type Init = MainViewInit;
    type Input = MainViewMsg;
    type Output = MainViewOutMsg;

    menu! {
        app_menu: {
            section! {
                // value inside action is ignored
                "_Debug View" => DebugViewToggleAction,
                "_Build Profile" => BuildProfileAction,
                "C_lean Build Profile" => BuildProfileCleanAction,
                "Configure _WiVRn" => ConfigureWivrnAction,
            },
            section! {
                "_About" => AboutAction,
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
            set_top_bar_style: adw::ToolbarStyle::Flat,
            set_bottom_bar_style: adw::ToolbarStyle::Flat,
            #[track = "model.changed(Self::enable_debug_view())"]
            set_hexpand: !model.enable_debug_view,
            set_vexpand: true,
            set_size_request: (360, 350),
            add_top_bar: top_bar = &adw::HeaderBar {
                set_hexpand: true,
                set_vexpand: false,
                pack_end: menu_btn = &gtk::MenuButton {
                    set_icon_name: "open-menu-symbolic",
                    set_tooltip_text: Some("Menu"),
                    set_menu_model: Some(&app_menu),
                },
            },
            #[wrap(Some)]
            set_content: content = &gtk::ScrolledWindow {
                set_hscrollbar_policy: gtk::PolicyType::Never,
                set_hexpand: true,
                set_vexpand: true,
                adw::Clamp {
                    set_maximum_size: 600,
                    gtk::Box {
                        set_spacing: 12,
                        set_margin_all: 12,
                        set_orientation: gtk::Orientation::Vertical,
                        gtk::Box {
                            set_hexpand: true,
                            set_orientation: gtk::Orientation::Horizontal,
                            add_css_class: "linked",
                            gtk::Button {
                                add_css_class: "suggested-action",
                                add_css_class: "destructive-action",
                                set_hexpand: true,
                                #[track = "model.changed(Self::xrservice_active())"]
                                set_class_active: ("suggested-action", !model.xrservice_active),
                                #[track = "model.changed(Self::xrservice_active())"]
                                set_label: match model.xrservice_active {
                                    true => "Stop",
                                    false => "Start",
                                },
                                connect_clicked[sender] => move |_| {
                                    sender.input(MainViewMsg::StartStopClicked);
                                },
                            },
                            gtk::Button {
                                set_halign: gtk::Align::Center,
                                set_valign: gtk::Align::Center,
                                set_icon_name: "view-refresh-symbolic",
                                set_tooltip_text: Some("Restart"),
                                #[track = "model.changed(Self::xrservice_active())"]
                                set_visible: model.xrservice_active,
                                connect_clicked[sender] => move |_| {
                                    sender.input(MainViewMsg::RestartXRService)
                                },
                            },
                        },
                        adw::Bin {
                            #[track = "model.changed(Self::xrservice_active())"]
                            set_visible: model.xrservice_active,
                            add_css_class: "card",
                            gtk::Label {
                                #[track = "model.changed(Self::xrservice_active()) || model.changed(Self::xrservice_ready())"]
                                set_label: if model.xrservice_ready {
                                    "Service ready, you can launch XR apps"
                                } else {
                                    match model.selected_profile.xrservice_type {
                                        XRServiceType::Monado =>
                                            "Starting…",
                                        XRServiceType::Wivrn =>
                                            "Starting, please connect your client device…",
                                    }
                                },
                                set_margin_all: 18,
                                add_css_class: "heading",
                                add_css_class: "success",
                                add_css_class: "warning",
                                #[track = "model.changed(Self::xrservice_active()) || model.changed(Self::xrservice_ready())"]
                                set_class_active: ("warning", !model.xrservice_ready),
                                set_wrap: true,
                                set_justify: gtk::Justification::Center,
                            },
                        },
                        model.devices_box.widget(),
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_hexpand: true,
                            set_vexpand: false,
                            set_spacing: 12,
                            add_css_class: "card",
                            add_css_class: "padded",
                            #[track = "model.changed(Self::selected_profile())"]
                            set_visible: match mount_has_nosuid(&model.selected_profile.prefix) {
                                Ok(b) => b,
                                Err(_) => {
                                    eprintln!(
                                        "Warning (nosuid detection): could not get stat on path {}",
                                        model.selected_profile.prefix.to_string_lossy());
                                    false
                                },
                            },
                            warning_heading(),
                            gtk::Label {
                                set_label: concat!(
                                    "Your current prefix is inside a partition ",
                                    "mounted with the nosuid option.\nThis will prevent ",
                                    "the XR runtime from acquiring certain privileges ",
                                    "and will cause noticeable stutter when running XR ",
                                    "applications."
                                ),
                                add_css_class: "warning",
                                set_xalign: 0.0,
                                set_wrap: true,
                                set_wrap_mode: gtk::pango::WrapMode::Word,
                            }
                        },
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_hexpand: true,
                            set_vexpand: false,
                            set_spacing: 12,
                            add_css_class: "card",
                            add_css_class: "padded",
                            #[track = "model.changed(Self::selected_profile())"]
                            set_visible: !dep_pkexec().check(),
                            warning_heading(),
                            gtk::Label {
                                set_label: &format!(
                                    "Pkexec wasn't found on your system.\nThis will prevent the XR runtime from acquiring certain priviledges and will cause noticeable stutter when running XR applications.\nYou can fix this by installing the following package on your system: <tt>{}</tt>", dep_pkexec().package_name()
                                ),
                                set_use_markup: true,
                                add_css_class: "warning",
                                set_xalign: 0.0,
                                set_wrap: true,
                                set_wrap_mode: gtk::pango::WrapMode::Word,
                            }
                        },
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_hexpand: true,
                            set_vexpand: false,
                            set_spacing: 12,
                            add_css_class: "card",
                            add_css_class: "padded",
                            #[track = "model.changed(Self::selected_profile())"]
                            set_visible: model.selected_profile.lighthouse_driver == LighthouseDriver::SteamVR && !chaperone_info_exists(),
                            warning_heading(),
                            gtk::Label {
                                set_label: concat!(
                                    "SteamVR room configuration not found.\n",
                                    "To use the SteamVR lighthouse driver, you ",
                                    "will need to run SteamVR and perform the room setup.",
                                ),
                                add_css_class: "warning",
                                set_xalign: 0.0,
                                set_wrap: true,
                                set_wrap_mode: gtk::pango::WrapMode::Word,
                            }
                        },
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_hexpand: true,
                            set_vexpand: false,
                            set_spacing: 12,
                            add_css_class: "card",
                            add_css_class: "padded",
                            #[track = "model.changed(Self::selected_profile())"]
                            set_visible: match get_amd_gpu_power_profile() {
                                None => false,
                                Some(GpuPowerProfile::VR) => false,
                                Some(_) => true,
                            },
                            warning_heading(),
                            gtk::Label {
                                set_use_markup: true,
                                set_markup: concat!(
                                    "Your AMD GPU Power Profile is not set to VR. ",
                                    "This will cause noticeable stutter when running XR ",
                                    "applications.\n\n",
                                    "You can activate the VR Power Profile using ",
                                    "GPU overclocking utilities such as ",
                                    "<a href=\"https://gitlab.com/corectrl/corectrl\">CoreCtrl</a>."
                                ),
                                add_css_class: "warning",
                                set_xalign: 0.0,
                                set_wrap: true,
                                set_wrap_mode: gtk::pango::WrapMode::Word,
                            },
                            gtk::Button {
                                set_halign: gtk::Align::Start,
                                set_label: "Refresh",
                                connect_clicked[sender, profiles_dropdown] => move |_| {
                                    sender.input(Self::Input::SetSelectedProfile(profiles_dropdown.selected()));
                                }
                            },
                        },

                        model.steam_launch_options_box.widget(),
                        model.install_wivrn_box.widget(),
                        model.steamvr_calibration_box.widget(),

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_hexpand: true,
                            set_vexpand: false,
                            set_spacing: 12,
                            add_css_class: "card",
                            add_css_class: "padded",
                            #[track = "model.changed(Self::selected_profile())"]
                            set_visible: model.selected_profile.lighthouse_driver == LighthouseDriver::Survive,
                            gtk::Label {
                                add_css_class: "heading",
                                set_hexpand: true,
                                set_xalign: 0.0,
                                set_label: "Libsurvive Calibration",
                                set_wrap: true,
                                set_wrap_mode: gtk::pango::WrapMode::Word,
                            },
                            gtk::Label {
                                add_css_class: "dim-label",
                                set_hexpand: true,
                                set_label: concat!(
                                    "Libsurvive needs to import your SteamVR calibration to work ",
                                    "properly. You need to have used SteamVR with this setup ",
                                    "before to be able to import its calibration."
                                ),
                                set_xalign: 0.0,
                                set_wrap: true,
                                set_wrap_mode: gtk::pango::WrapMode::Word,
                            },
                            gtk::Button {
                                add_css_class: "suggested-action",
                                set_label: "Calibrate",
                                set_halign: gtk::Align::Start,
                                connect_clicked[sender] => move |_| {
                                    sender.output(Self::Output::OpenLibsurviveSetup).expect("Sender output failed");
                                }
                            },
                        },
                    }
                }
            },
            add_bottom_bar: bottom_bar = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 12,
                set_margin_all: 12,
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    add_css_class: "linked",
                    #[name(profiles_dropdown)]
                    gtk::DropDown {
                        set_hexpand: true,
                        #[track = "model.changed(Self::selected_profile())"]
                        set_tooltip_text: Some(format!("Profile: {}", model.selected_profile).as_str()),
                        #[track = "model.changed(Self::profiles())"]
                        set_model: Some(&{
                            let names: Vec<_> = model.profiles.iter().map(|p| p.name.as_str()).collect();
                            gtk::StringList::new(&names)
                        }),
                        connect_selected_item_notify[sender] => move |dd| {
                            sender.input(MainViewMsg::ProfileSelected(dd.selected()));
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
                    },
                },
            }
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        self.reset();

        match message {
            Self::Input::ClockTicking => {}
            Self::Input::StartStopClicked => {
                sender
                    .output(Self::Output::DoStartStopXRService)
                    .expect("Sender output failed");
            }
            Self::Input::RestartXRService => {
                sender
                    .output(Self::Output::RestartXRService)
                    .expect("Sender output failed");
            }
            Self::Input::XRServiceActiveChanged(active, profile, show_launch_opts) => {
                if !active {
                    self.set_xrservice_ready(false);
                }
                self.set_xrservice_active(active);
                self.steamvr_calibration_box
                    .sender()
                    .emit(SteamVrCalibrationBoxMsg::XRServiceActiveChanged(active));
                if !active {
                    sender.input(Self::Input::UpdateDevices(vec![]));
                }
                self.steam_launch_options_box.sender().emit(
                    SteamLaunchOptionsBoxMsg::UpdateXRServiceActive(show_launch_opts),
                );
                if let Some(prof) = profile {
                    self.steam_launch_options_box
                        .sender()
                        .emit(SteamLaunchOptionsBoxMsg::UpdateLaunchOptions(prof));
                }
            }
            Self::Input::EnableDebugViewChanged(val) => {
                self.set_enable_debug_view(val);
            }
            Self::Input::UpdateSelectedProfile(prof) => {
                self.set_selected_profile(prof.clone());
                self.profile_delete_action.set_enabled(prof.editable);
                self.profile_export_action.set_enabled(prof.editable);
                self.steamvr_calibration_box
                    .sender()
                    .emit(SteamVrCalibrationBoxMsg::SetVisible(
                        prof.lighthouse_driver == LighthouseDriver::SteamVR,
                    ));
                self.install_wivrn_box
                    .sender()
                    .emit(InstallWivrnBoxMsg::UpdateSelectedProfile(prof.clone()));
            }
            Self::Input::UpdateProfiles(profiles, config) => {
                self.set_profiles(profiles);
                // why send another message to set the dropdown selection?
                // set_* from tracker likely updates the view obj in the next
                // draw, so selecting here will result in nothing cause the
                // dropdown is effectively empty
                sender.input(MainViewMsg::SetSelectedProfile({
                    let pos = self
                        .profiles
                        .iter()
                        .position(|p| p.uuid == config.selected_profile_uuid);
                    match pos {
                        Some(idx) => idx as u32,
                        None => 0,
                    }
                }));
            }
            Self::Input::SetSelectedProfile(index) => {
                self.profiles_dropdown
                    .as_ref()
                    .unwrap()
                    .clone()
                    .set_selected(index);
                sender.input(Self::Input::UpdateSelectedProfile(
                    self.profiles.get(index as usize).unwrap().clone(),
                ));
            }
            Self::Input::ProfileSelected(position) => {
                sender
                    .output(MainViewOutMsg::ProfileSelected(
                        self.profiles.get(position as usize).unwrap().clone(),
                    ))
                    .expect("Sender output failed");
            }
            Self::Input::EditProfile => {
                if self.selected_profile.editable {
                    sender.input(Self::Input::OpenProfileEditor(
                        self.selected_profile.clone(),
                    ));
                } else {
                    self.profile_not_editable_dialog
                        .present(Some(&self.root_win));
                }
            }
            Self::Input::CreateProfile => {
                sender.input(Self::Input::OpenProfileEditor(Profile::default()));
            }
            Self::Input::DeleteProfile => {
                self.profile_delete_confirm_dialog
                    .present(Some(&self.root_win));
            }
            Self::Input::SaveProfile(prof) => {
                sender
                    .output(Self::Output::SaveProfile(prof))
                    .expect("Sender output failed");
            }
            Self::Input::DuplicateProfile => {
                if self.selected_profile.can_be_built {
                    sender.input(Self::Input::OpenProfileEditor(
                        self.selected_profile.create_duplicate(),
                    ));
                } else {
                    alert(
                        "This profile cannot be duplicated",
                        None,
                        Some(&self.root_win),
                    );
                }
            }
            Self::Input::ExportProfile => {
                let prof = self.selected_profile.clone();
                if !prof.editable {
                    return;
                }
                let chooser = gtk::FileDialog::builder()
                    .modal(true)
                    .title("Choose a location for the exported profile")
                    .initial_name(&format!("{}.json", &prof.name))
                    .accept_label("Export")
                    .build();
                let root_win = self.root_win.clone();
                chooser.save(
                    Some(&self.root_win),
                    gtk::gio::Cancellable::NONE,
                    move |res| {
                        if let Ok(file) = res {
                            if let Some(path) = file.path() {
                                if let Ok(mut writer) = get_writer(&path) {
                                    if let Ok(s) = serde_json::to_string_pretty(&prof) {
                                        let prof_id = prof.uuid;
                                        let s = s
                                            .replace(
                                                get_data_dir().to_string_lossy().as_ref(),
                                                "@DATADIR@",
                                            )
                                            .replace(
                                                get_home_dir().to_string_lossy().as_ref(),
                                                "@HOMEDIR@",
                                            )
                                            .replace(&prof_id, "@UUID@");
                                        if writer.write_all(s.as_bytes()).is_ok() {
                                            return;
                                        }
                                    }
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
                                if let Some(path) = file.path() {
                                    if let Ok(s) = read_to_string(path) {
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
                                }
                                alert("Failed to import profile", None, Some(&root_win));
                            }
                        },
                    );
                });
                confirm_dialog.present(Some(&self.root_win));
            }
            Self::Input::UpdateDevices(devs) => {
                self.devices_box
                    .sender()
                    .emit(DevicesBoxMsg::UpdateDevices(devs));
            }
            Self::Input::UpdateXrServiceReady(ready) => {
                self.set_xrservice_ready(ready);
            }
            Self::Input::OpenProfileEditor(profile) => {
                self.create_profile_editor(sender, profile);
                self.profile_editor
                    .as_ref()
                    .unwrap()
                    .sender()
                    .emit(ProfileEditorMsg::Present);
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let profile_not_editable_dialog = adw::AlertDialog::builder()
            .heading("This profile is not editable")
            .body(concat!(
                "You can duplicate it and edit the new copy. ",
                "Do you want to duplicate the current profile?"
            ))
            .build();
        profile_not_editable_dialog.add_response("no", "_No");
        profile_not_editable_dialog.add_response("yes", "_Yes");
        profile_not_editable_dialog.set_response_appearance("yes", ResponseAppearance::Suggested);

        profile_not_editable_dialog.connect_response(
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

        let profile_delete_confirm_dialog = adw::AlertDialog::builder()
            .heading("Are you sure you want to delete this profile?")
            .build();
        profile_delete_confirm_dialog.add_response("no", "_No");
        profile_delete_confirm_dialog.add_response("yes", "_Yes");
        profile_delete_confirm_dialog
            .set_response_appearance("yes", ResponseAppearance::Destructive);

        profile_delete_confirm_dialog.connect_response(
            None,
            clone!(
                #[strong]
                sender,
                move |_, res| {
                    if res == "yes" {
                        sender
                            .output(Self::Output::DeleteProfile)
                            .expect("Sender output failed");
                    }
                }
            ),
        );

        let steamvr_calibration_box = SteamVrCalibrationBox::builder().launch(()).detach();
        steamvr_calibration_box
            .sender()
            .emit(SteamVrCalibrationBoxMsg::SetVisible(
                init.selected_profile.lighthouse_driver == LighthouseDriver::SteamVR,
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

        let mut model = Self {
            xrservice_active: false,
            enable_debug_view: init.config.debug_view_enabled,
            profiles_dropdown: None,
            profiles: vec![],
            steam_launch_options_box: SteamLaunchOptionsBox::builder().launch(()).detach(),
            install_wivrn_box: InstallWivrnBox::builder()
                .launch(InstallWivrnBoxInit {
                    selected_profile: init.selected_profile.clone(),
                    root_win: init.root_win.clone(),
                })
                .detach(),
            devices_box: DevicesBox::builder().launch(()).detach(),
            selected_profile: init.selected_profile.clone(),
            profile_not_editable_dialog,
            profile_delete_confirm_dialog,
            root_win: init.root_win.clone(),
            steamvr_calibration_box,
            profile_editor: None,
            xrservice_ready: false,
            profile_delete_action,
            profile_export_action,
            tracker: 0,
        };
        let widgets = view_output!();

        model.profiles_dropdown = Some(widgets.profiles_dropdown.clone());

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

        ComponentParts { model, widgets }
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
