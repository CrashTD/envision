use super::alert::alert;
use super::devices_box::{DevicesBox, DevicesBoxMsg};
use super::install_wivrn_box::{InstallWivrnBox, InstallWivrnBoxInit, InstallWivrnBoxMsg};
use super::profile_editor::{ProfileEditor, ProfileEditorMsg, ProfileEditorOutMsg};
use super::steam_launch_options_box::{SteamLaunchOptionsBox, SteamLaunchOptionsBoxMsg};
use super::steamvr_calibration_box::SteamVrCalibrationBox;
use crate::config::Config;
use crate::dependencies::common::dep_pkexec;
use crate::file_utils::mount_has_nosuid;
use crate::gpu_profile::{get_amd_gpu_power_profile, GpuPowerProfile};
use crate::profile::{LighthouseDriver, Profile, XRServiceType};
use crate::steamvr_utils::chaperone_info_exists;
use crate::ui::app::{
    AboutAction, BuildProfileAction, BuildProfileCleanAction, ConfigFbtAction,
    DebugViewToggleAction,
};
use crate::ui::profile_editor::ProfileEditorInit;
use crate::ui::steamvr_calibration_box::SteamVrCalibrationBoxMsg;
use crate::ui::util::{limit_dropdown_width, warning_heading};
use crate::xr_devices::XRDevice;
use gtk::prelude::*;
use relm4::adw::{prelude::MessageDialogExt, ResponseAppearance};
use relm4::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};

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
    install_wivrn_box: Controller<InstallWivrnBox>,
    #[tracker::do_not_track]
    steam_launch_options_box: Controller<SteamLaunchOptionsBox>,
    #[tracker::do_not_track]
    devices_box: Controller<DevicesBox>,
    #[tracker::do_not_track]
    profile_not_editable_dialog: adw::MessageDialog,
    #[tracker::do_not_track]
    profile_delete_confirm_dialog: adw::MessageDialog,
    #[tracker::do_not_track]
    profile_editor: Option<Controller<ProfileEditor>>,
    #[tracker::do_not_track]
    steamvr_calibration_box: Controller<SteamVrCalibrationBox>,
    #[tracker::do_not_track]
    root_win: gtk::Window,
}

#[derive(Debug)]
pub enum MainViewMsg {
    ClockTicking,
    StartStopClicked,
    RestartXRService,
    XRServiceActiveChanged(bool, Option<Profile>),
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
}

#[derive(Debug)]
pub enum MainViewOutMsg {
    DoStartStopXRService,
    RestartXRService,
    ProfileSelected(Profile),
    DeleteProfile,
    SaveProfile(Profile),
    OpenLibsurviveSetup,
    OpenWivrnConfig,
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
                "Configure Full Body _Tracking" => ConfigFbtAction,
            },
            section! {
                "_About" => AboutAction,
            }
        }
    }

    view! {
        // TODO: refactor with adw.toolbarview
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
                        model.devices_box.widget(),
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_hexpand: true,
                            set_vexpand: false,
                            set_spacing: 12,
                            add_css_class: "card",
                            add_css_class: "padded",
                            #[track = "model.changed(Self::selected_profile())"]
                            set_visible: match mount_has_nosuid(model.selected_profile.prefix.as_str()) {
                                Ok(b) => b,
                                Err(_) => {
                                    // TODO: handle this error better
                                    println!(
                                        "Warning: could not get stat on path {}",
                                        model.selected_profile.prefix);
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
                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_hexpand: true,
                            set_vexpand: false,
                            set_spacing: 12,
                            add_css_class: "card",
                            add_css_class: "padded",
                            #[track = "model.changed(Self::selected_profile())"]
                            set_visible: model.selected_profile.xrservice_type == XRServiceType::Wivrn,
                            gtk::Label {
                                add_css_class: "heading",
                                set_hexpand: true,
                                set_xalign: 0.0,
                                set_label: "Configure WiVRn",
                                set_wrap: true,
                                set_wrap_mode: gtk::pango::WrapMode::Word,
                            },
                            gtk::Button {
                                add_css_class: "suggested-action",
                                set_label: "Configure",
                                set_halign: gtk::Align::End,
                                connect_clicked[sender] => move |_| {
                                    sender.output(Self::Output::OpenWivrnConfig).expect("Sender output failed");
                                }
                            },
                        },
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
                                dd, match model.enable_debug_view {
                                    true => 18,
                                    false => -1,
                                });
                        },
                    },
                    gtk::Button {
                        set_icon_name: "edit-symbolic",
                        set_tooltip_text: Some("Edit Profile"),
                        connect_clicked[sender] => move |_| {
                            sender.input(Self::Input::EditProfile);
                        },
                    },
                    gtk::Button {
                        set_icon_name: "edit-copy-symbolic",
                        set_tooltip_text: Some("Duplicate Profile"),
                        connect_clicked[sender] => move |_| {
                            sender.input(Self::Input::DuplicateProfile);
                        },
                    },
                    gtk::Button {
                        set_icon_name: "list-add-symbolic",
                        set_tooltip_text: Some("Create Profile"),
                        connect_clicked[sender] => move |_| {
                            sender.input(Self::Input::CreateProfile);
                        },
                    },
                    gtk::Button {
                        set_icon_name: "edit-delete-symbolic",
                        add_css_class: "destructive-action",
                        set_tooltip_text: Some("Delete Profile"),
                        #[track = "model.changed(Self::selected_profile())"]
                        set_sensitive: model.selected_profile.editable,
                        connect_clicked[sender] => move |_| {
                            sender.input(Self::Input::DeleteProfile);
                        },
                    },
                },
            }
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        self.reset();

        match message {
            Self::Input::ClockTicking => {
                self.install_wivrn_box
                    .sender()
                    .emit(InstallWivrnBoxMsg::ClockTicking);
            }
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
            Self::Input::XRServiceActiveChanged(active, profile) => {
                self.set_xrservice_active(active);
                self.steamvr_calibration_box
                    .sender()
                    .emit(SteamVrCalibrationBoxMsg::XRServiceActiveChanged(active));
                if !active {
                    sender.input(Self::Input::UpdateDevices(vec![]));
                }
                self.steam_launch_options_box
                    .sender()
                    .emit(SteamLaunchOptionsBoxMsg::UpdateXRServiceActive(active));
                match profile {
                    None => {}
                    Some(prof) => {
                        self.steam_launch_options_box
                            .sender()
                            .emit(SteamLaunchOptionsBoxMsg::UpdateLaunchOptions(prof));
                    }
                }
            }
            Self::Input::EnableDebugViewChanged(val) => {
                self.set_enable_debug_view(val);
                limit_dropdown_width(
                    self.profiles_dropdown.as_ref().unwrap(),
                    match val {
                        true => 18,
                        false => -1,
                    },
                )
            }
            Self::Input::UpdateSelectedProfile(prof) => {
                self.set_selected_profile(prof.clone());
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
                self.set_selected_profile(self.profiles.get(index as usize).unwrap().clone());
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
                    self.create_profile_editor(sender, self.selected_profile.clone());
                    self.profile_editor
                        .as_ref()
                        .unwrap()
                        .emit(ProfileEditorMsg::Present);
                } else {
                    self.profile_not_editable_dialog.present();
                }
            }
            Self::Input::CreateProfile => {
                self.create_profile_editor(sender, Profile::default());
                self.profile_editor
                    .as_ref()
                    .unwrap()
                    .sender()
                    .emit(ProfileEditorMsg::Present);
            }
            Self::Input::DeleteProfile => {
                self.profile_delete_confirm_dialog.present();
            }
            Self::Input::SaveProfile(prof) => {
                sender
                    .output(Self::Output::SaveProfile(prof))
                    .expect("Sender output failed");
            }
            Self::Input::DuplicateProfile => {
                if self.selected_profile.can_be_built {
                    self.create_profile_editor(sender, self.selected_profile.create_duplicate());
                    self.profile_editor
                        .as_ref()
                        .unwrap()
                        .sender()
                        .emit(ProfileEditorMsg::Present);
                } else {
                    alert(
                        "This profile cannot be duplicated",
                        None,
                        Some(&self.root_win),
                    );
                }
            }
            Self::Input::UpdateDevices(devs) => self
                .devices_box
                .sender()
                .emit(DevicesBoxMsg::UpdateDevices(devs)),
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let profile_not_editable_dialog = adw::MessageDialog::builder()
            .modal(true)
            .transient_for(&init.root_win)
            .hide_on_close(true)
            .heading("This profile is not editable")
            .body(concat!(
                "You can duplicate it and edit the new copy. ",
                "Do you want to duplicate the current profile?"
            ))
            .build();
        profile_not_editable_dialog.add_response("no", "_No");
        profile_not_editable_dialog.add_response("yes", "_Yes");
        profile_not_editable_dialog.set_response_appearance("no", ResponseAppearance::Destructive);
        profile_not_editable_dialog.set_response_appearance("yes", ResponseAppearance::Suggested);

        {
            let pne_sender = sender.clone();
            profile_not_editable_dialog.connect_response(None, move |_, res| {
                if res == "yes" {
                    pne_sender.input(Self::Input::DuplicateProfile);
                }
            });
        }

        let profile_delete_confirm_dialog = adw::MessageDialog::builder()
            .modal(true)
            .transient_for(&init.root_win)
            .hide_on_close(true)
            .heading("Are you sure you want to delete this profile?")
            .build();
        profile_delete_confirm_dialog.add_response("no", "_No");
        profile_delete_confirm_dialog.add_response("yes", "_Yes");
        profile_delete_confirm_dialog
            .set_response_appearance("no", ResponseAppearance::Destructive);
        profile_delete_confirm_dialog.set_response_appearance("yes", ResponseAppearance::Suggested);

        {
            let pdc_sender = sender.clone();
            profile_delete_confirm_dialog.connect_response(None, move |_, res| {
                if res == "yes" {
                    pdc_sender
                        .output(Self::Output::DeleteProfile)
                        .expect("Sender output failed");
                }
            });
        }

        let steamvr_calibration_box = SteamVrCalibrationBox::builder().launch(()).detach();
        steamvr_calibration_box
            .sender()
            .emit(SteamVrCalibrationBoxMsg::SetVisible(
                init.selected_profile.lighthouse_driver == LighthouseDriver::SteamVR,
            ));

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
            tracker: 0,
        };
        let widgets = view_output!();

        model.profiles_dropdown = Some(widgets.profiles_dropdown.clone());

        ComponentParts { model, widgets }
    }
}
