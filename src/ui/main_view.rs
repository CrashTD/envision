#![allow(unused_assignments)] // vars named in view! macro are not correctly recognized as used

use super::{
    SENDER_IO_ERR_MSG,
    app::{AboutAction, ConfigureWivrnAction, DebugViewToggleAction},
    devices_box::{DevicesBox, DevicesBoxMsg},
    install_wivrn_box::{InstallWivrnBox, InstallWivrnBoxInit, InstallWivrnBoxMsg},
    openhmd_calibration_box::{OpenHmdCalibrationBox, OpenHmdCalibrationBoxMsg},
    steam_launch_options_box::{SteamLaunchOptionsBox, SteamLaunchOptionsBoxMsg},
    steamvr_calibration_box::{SteamVrCalibrationBox, SteamVrCalibrationBoxMsg},
    wivrn_wired_start_box::{WivrnWiredStartBox, WivrnWiredStartBoxInit, WivrnWiredStartBoxMsg},
};
use crate::{
    config::Config,
    depcheck::common::dep_pkexec,
    gpu_profile::{GpuPowerProfile, get_amd_gpu_power_profile},
    profile::{LighthouseDriver, Profile, XRServiceType},
    ui::{
        app::{PreferencesAction, ThemeManagerAction},
        util::warn_card,
    },
    util::{file_utils::mount_has_nosuid, steam_library_folder::chaperone_info_exists},
    wivrn_dbus,
    xr_devices::XRDevice,
};
use adw::prelude::*;
use relm4::prelude::*;
use tracing::{error, warn};

#[tracker::track]
pub struct MainView {
    xrservice_active: bool,
    enable_debug_view: bool,
    #[no_eq]
    selected_profile: Profile,
    #[tracker::do_not_track]
    install_wivrn_box: AsyncController<InstallWivrnBox>,
    #[tracker::do_not_track]
    wivrn_wired_start_box: AsyncController<WivrnWiredStartBox>,
    #[tracker::do_not_track]
    steam_launch_options_box: Controller<SteamLaunchOptionsBox>,
    #[tracker::do_not_track]
    devices_box: Controller<DevicesBox>,
    #[tracker::do_not_track]
    steamvr_calibration_box: Controller<SteamVrCalibrationBox>,
    #[tracker::do_not_track]
    openhmd_calibration_box: Controller<OpenHmdCalibrationBox>,
    xrservice_ready: bool,
    wivrn_pairing_mode: bool,
    wivrn_pin: Option<String>,
    wivrn_supports_pairing: bool,
    show_amd_gpu_power_profile_warn: bool,
}

#[derive(Debug)]
pub enum MainViewMsg {
    ClockTicking,
    StartStopClicked,
    RestartXRService,
    XRServiceActiveChanged(bool, Option<Profile>, bool),
    EnableDebugViewChanged(bool),
    UpdateSelectedProfile(Profile),
    UpdateDevices(Vec<XRDevice>),
    UpdateXrServiceReady(bool),
    SetWivrnSupportsPairing(bool),
    SetWivrnPairingMode(bool),
    StopWivrnPairingMode,
    StartWivrnPairingMode,
    CheckAmdGpuPowerProfile,
}

#[derive(Debug)]
pub enum MainViewOutMsg {
    DoStartStopXRService,
    RestartXRService,
    OpenLibsurviveSetup,
    LaunchMonadoGui,
}

pub struct MainViewInit {
    pub config: Config,
    pub selected_profile: Profile,
    pub root_win: gtk::Window,
}

#[relm4::component(pub async)]
impl AsyncComponent for MainView {
    type Init = MainViewInit;
    type Input = MainViewMsg;
    type Output = MainViewOutMsg;
    type CommandOutput = ();

    menu! {
        app_menu: {
            section! {
                "_Preferences" => PreferencesAction,
                // value inside action is ignored
                "_Developer mode" => DebugViewToggleAction,
                "Configure _WiVRn" => ConfigureWivrnAction,
            },
            section! {
                "Change _theme" => ThemeManagerAction,
                "_About" => AboutAction,
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
                        gtk::Button {
                            #[track = "model.changed(Self::selected_profile())"]
                            set_visible: !model.selected_profile.is_system_profile && !model.selected_profile.can_start(None),
                            add_css_class: "pill",
                            add_css_class: "suggested-action",
                            set_hexpand: true,
                            set_label: "Build profile",
                            set_action_name: Some("win.buildprofileclean"),
                        },
                        gtk::Box {
                            set_hexpand: true,
                            set_orientation: gtk::Orientation::Horizontal,
                            add_css_class: "linked",
                            gtk::Button {
                                add_css_class: "suggested-action",
                                add_css_class: "destructive-action",
                                set_hexpand: true,
                                #[track = "model.changed(Self::selected_profile())"]
                                set_visible: model.selected_profile.is_system_profile || model.selected_profile.can_start(None),
                                #[track = "model.changed(Self::xrservice_active())"]
                                set_class_active: ("suggested-action", !model.xrservice_active),
                                #[track = "model.changed(Self::xrservice_active())"]
                                set_class_active: ("destructive-action", model.xrservice_active),
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
                                #[track = "model.changed(Self::xrservice_active()) || model.changed(Self::xrservice_ready()) || model.changed(Self::wivrn_pairing_mode())"]
                                set_label: {
                                    match model.selected_profile.xrservice_type {
                                        XRServiceType::Monado =>
                                            if model.xrservice_ready {
                                                "Service ready, you can launch XR apps"
                                            } else {
                                                "Starting…"
                                            }
                                        XRServiceType::Wivrn =>
                                            if model.wivrn_pairing_mode {
                                                "Pairing mode"
                                            } else {
                                                "Starting, connect your client device…"
                                            }
                                    }
                                },
                                set_margin_all: 18,
                                add_css_class: "heading",
                                add_css_class: "success",
                                add_css_class: "warning",
                                #[track = "model.changed(Self::xrservice_active()) || model.changed(Self::xrservice_ready()) || model.changed(Self::wivrn_pairing_mode())"]
                                set_class_active: (
                                    "success",
                                    model.xrservice_ready
                                    && (
                                        model.selected_profile.xrservice_type != XRServiceType::Wivrn
                                        || !model.wivrn_pairing_mode
                                    )
                                ),
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
                            #[track = "model.changed(Self::wivrn_supports_pairing()) || model.changed(Self::xrservice_active()) || model.changed(Self::selected_profile()) || model.changed(Self::wivrn_pairing_mode()) || model.changed(Self::wivrn_pin())"]
                            set_visible: model.wivrn_supports_pairing
                                && model.xrservice_active
                                && model.selected_profile.xrservice_type == XRServiceType::Wivrn
                                && !model.wivrn_pairing_mode,
                            gtk::Label {
                                add_css_class: "heading",
                                set_hexpand: true,
                                set_xalign: 0.0,
                                set_label: "Pairing mode",
                                set_wrap: true,
                                set_wrap_mode: gtk::pango::WrapMode::Word,
                            },
                            gtk::Label {
                                add_css_class: "dim-label",
                                set_hexpand: true,
                                set_label: concat!(
                                    "To connect a new device to WiVRn, you ",
                                    "will need to pair it first.\n\n",
                                    "You can do so by starting the pairing mode ",
                                    "with the button below."
                                ),
                                set_xalign: 0.0,
                                set_wrap: true,
                                set_wrap_mode: gtk::pango::WrapMode::Word,
                            },
                            gtk::Button {
                                add_css_class: "suggested-action",
                                set_label: "Start pairing mode",
                                set_halign: gtk::Align::Start,
                                connect_clicked[sender] => move |_| {
                                    sender.input(Self::Input::StartWivrnPairingMode);
                                }
                            },
                        },
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_hexpand: true,
                            set_vexpand: false,
                            set_spacing: 12,
                            add_css_class: "card",
                            add_css_class: "padded",
                            #[track = "model.changed(Self::wivrn_supports_pairing()) || model.changed(Self::xrservice_active()) || model.changed(Self::selected_profile()) || model.changed(Self::wivrn_pairing_mode()) || model.changed(Self::wivrn_pin())"]
                            set_visible: model.wivrn_supports_pairing
                                && model.xrservice_active
                                && model.selected_profile.xrservice_type == XRServiceType::Wivrn
                                && model.wivrn_pairing_mode && model.wivrn_pin.is_some(),
                            gtk::Label {
                                add_css_class: "heading",
                                set_hexpand: true,
                                set_xalign: 0.0,
                                set_label: "Pairing mode",
                                set_wrap: true,
                                set_wrap_mode: gtk::pango::WrapMode::Word,
                            },
                            gtk::Label {
                                add_css_class: "dim-label",
                                set_hexpand: true,
                                set_label: concat!(
                                    "WiVRn is in pairing mode. Pair your client ",
                                    "device with the following PIN:"
                                ),
                                set_xalign: 0.0,
                                set_wrap: true,
                                set_wrap_mode: gtk::pango::WrapMode::Word,
                            },
                            gtk::Label {
                                add_css_class: "title-2",
                                add_css_class: "monospace",
                                set_hexpand: true,
                                set_selectable: true,
                                #[track = "model.changed(Self::wivrn_pin())"]
                                set_label: model.wivrn_pin
                                    .as_deref().unwrap_or(""),
                                set_xalign: 0.5,
                                set_justify: gtk::Justification::Center,
                                set_wrap: true,
                                set_wrap_mode: gtk::pango::WrapMode::Word,
                            },
                            gtk::Button {
                                add_css_class: "destructive-action",
                                set_label: "Stop pairing mode",
                                set_halign: gtk::Align::Start,
                                connect_clicked[sender] => move |_| {
                                    sender.input(Self::Input::StopWivrnPairingMode);
                                }
                            },
                        },
                        gtk::Button {
                            #[track = "model.changed(Self::xrservice_active()) || model.changed(Self::xrservice_ready())"]
                            set_visible: model.xrservice_active
                                && model.xrservice_ready
                                && model.selected_profile.xrservice_type == XRServiceType::Monado
                                && model.selected_profile.monado_gui_binary().is_some(),
                            set_label: "Launch Monado GUI",
                            connect_clicked[sender] => move |_| {
                                sender
                                    .output(Self::Output::LaunchMonadoGui)
                                    .expect(SENDER_IO_ERR_MSG);
                            },
                        },
                        warn_card(None, concat!(
                            "Your current prefix is inside a partition ",
                            "mounted with the nosuid option.\nThis will prevent ",
                            "the XR runtime from acquiring certain privileges ",
                            "and will cause noticeable stutter when running XR ",
                            "applications."
                        )) -> gtk::Box {
                            #[track = "model.changed(Self::selected_profile())"]
                            set_visible: !model.selected_profile.is_system_profile && match mount_has_nosuid(&model.selected_profile.prefix) {
                                Ok(b) => b,
                                Err(_) => {
                                    warn!(
                                        "nosuid detection: could not get stat on path {}",
                                        model.selected_profile.prefix.to_string_lossy());
                                    false
                                },
                            },
                        },
                        warn_card(None, &format!(
                            "Pkexec wasn't found on your system.\nThis will prevent the XR runtime from acquiring certain priviledges and will cause noticeable stutter when running XR applications.\nYou can fix this by installing the following package on your system: <tt>{}</tt>", dep_pkexec().package_name()
                        )) -> gtk::Box {
                            #[track = "model.changed(Self::selected_profile())"]
                            set_visible: !dep_pkexec().check(),
                        },
                        warn_card(None, concat!(
                            "SteamVR room configuration not found.\n",
                            "To use the SteamVR lighthouse driver, you ",
                            "will need to run SteamVR Quick Calibration.",
                        )) -> gtk::Box {
                            #[track = "model.changed(Self::selected_profile())"]
                            set_visible: model.selected_profile.lighthouse_driver == LighthouseDriver::SteamVR && !chaperone_info_exists(),
                        },
                        warn_card(None, concat!(
                            "Your AMD GPU Power Profile is not set to VR. ",
                            "This will cause noticeable stutter when running XR ",
                            "applications.\n\n",
                            "You can activate the VR Power Profile using ",
                            "GPU overclocking utilities such as ",
                            "<a href=\"https://gitlab.com/corectrl/corectrl\">CoreCtrl</a>."
                        )) -> gtk::Box {
                            #[track = "model.changed(Self::show_amd_gpu_power_profile_warn())"]
                            set_visible: model.show_amd_gpu_power_profile_warn,
                            gtk::Button {
                                set_halign: gtk::Align::Start,
                                set_label: "Refresh",
                                connect_clicked[sender] => move |_| {
                                    sender.input(Self::Input::CheckAmdGpuPowerProfile);
                                }
                            },
                        },

                        model.steam_launch_options_box.widget(),
                        model.wivrn_wired_start_box.widget(),
                        model.install_wivrn_box.widget(),
                        model.steamvr_calibration_box.widget(),
                        model.openhmd_calibration_box.widget(),

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
            Self::Input::ClockTicking => {}
            Self::Input::SetWivrnSupportsPairing(supported) => {
                if self.wivrn_supports_pairing != supported {
                    self.set_wivrn_supports_pairing(supported)
                }
            }
            Self::Input::SetWivrnPairingMode(enabled) => {
                if self.wivrn_pairing_mode != enabled {
                    self.set_wivrn_pairing_mode(enabled);
                    if enabled {
                        match wivrn_dbus::pairing_pin().await {
                            Ok(pin) => {
                                self.set_wivrn_pin(Some(pin));
                            }
                            Err(e) => {
                                error!("failed to get wivrn pairing pin: {e}");
                            }
                        };
                    } else {
                        self.set_wivrn_pin(None);
                    }
                }
            }
            Self::Input::StopWivrnPairingMode => {
                if let Err(e) = wivrn_dbus::disable_pairing().await {
                    error!("failed to stop wivrn pairing mode: {e}");
                }
            }
            Self::Input::StartWivrnPairingMode => {
                if let Err(e) = wivrn_dbus::enable_pairing().await {
                    error!("failed to start wivrn pairing mode: {e}");
                }
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
            Self::Input::XRServiceActiveChanged(active, profile, show_launch_opts) => {
                if !active {
                    self.set_xrservice_ready(false);
                    sender.input(Self::Input::SetWivrnPairingMode(false));
                }
                self.set_xrservice_active(active);
                self.steamvr_calibration_box
                    .sender()
                    .emit(SteamVrCalibrationBoxMsg::XRServiceActiveChanged(active));
                self.openhmd_calibration_box
                    .sender()
                    .emit(OpenHmdCalibrationBoxMsg::XRServiceActiveChanged(active));
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
                self.steamvr_calibration_box
                    .sender()
                    .emit(SteamVrCalibrationBoxMsg::SetVisible(
                        prof.lighthouse_driver == LighthouseDriver::SteamVR,
                    ));
                self.openhmd_calibration_box
                    .sender()
                    .emit(OpenHmdCalibrationBoxMsg::SetVisible(
                        prof.features.openhmd.enabled,
                    ));
                self.install_wivrn_box
                    .sender()
                    .emit(InstallWivrnBoxMsg::UpdateSelectedProfile(prof.clone()));
                self.wivrn_wired_start_box
                    .sender()
                    .emit(WivrnWiredStartBoxMsg::UpdateSelectedProfile(prof.clone()));
            }
            Self::Input::UpdateDevices(devs) => {
                self.devices_box
                    .sender()
                    .emit(DevicesBoxMsg::UpdateDevices(devs));
            }
            Self::Input::UpdateXrServiceReady(ready) => {
                self.set_xrservice_ready(ready);
            }
            Self::Input::CheckAmdGpuPowerProfile => {
                self.show_amd_gpu_power_profile_warn = match get_amd_gpu_power_profile() {
                    None => false,
                    Some(GpuPowerProfile::VR) => false,
                    Some(_) => true,
                }
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let steamvr_calibration_box = SteamVrCalibrationBox::builder().launch(()).detach();
        steamvr_calibration_box
            .sender()
            .emit(SteamVrCalibrationBoxMsg::SetVisible(
                init.selected_profile.lighthouse_driver == LighthouseDriver::SteamVR,
            ));
        let openhmd_calibration_box = OpenHmdCalibrationBox::builder().launch(()).detach();
        openhmd_calibration_box
            .sender()
            .emit(OpenHmdCalibrationBoxMsg::SetVisible(
                init.selected_profile.features.openhmd.enabled,
            ));

        let model = Self {
            xrservice_active: false,
            enable_debug_view: init.config.debug_view_enabled,
            steam_launch_options_box: SteamLaunchOptionsBox::builder().launch(()).detach(),
            install_wivrn_box: InstallWivrnBox::builder()
                .launch(InstallWivrnBoxInit {
                    selected_profile: init.selected_profile.clone(),
                    root_win: init.root_win.clone(),
                })
                .detach(),
            wivrn_wired_start_box: WivrnWiredStartBox::builder()
                .launch(WivrnWiredStartBoxInit {
                    selected_profile: init.selected_profile.clone(),
                    root_win: init.root_win.clone(),
                })
                .detach(),
            devices_box: DevicesBox::builder().launch(()).detach(),
            selected_profile: init.selected_profile.clone(),
            steamvr_calibration_box,
            openhmd_calibration_box,
            xrservice_ready: false,
            wivrn_pairing_mode: false,
            wivrn_supports_pairing: false,
            wivrn_pin: None,
            show_amd_gpu_power_profile_warn: false,
            tracker: 0,
        };
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }
}
