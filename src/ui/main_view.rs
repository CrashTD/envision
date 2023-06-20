use crate::config::Config;
use crate::constants::APP_NAME;
use crate::file_builders::active_runtime_json::{self, get_current_active_runtime};
use crate::profile::{Profile, XRServiceType};
use crate::ui::app::{
    AboutAction, BuildProfileAction, DebugViewToggleAction, LibsurviveSetupAction,
};
use expect_dialog::ExpectDialog;
use gtk::prelude::*;
use relm4::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use relm4_icons::icon_name;

use super::install_wivrn_box::{
    InstallWivrnBox, InstallWivrnBoxInit, InstallWivrnBoxMsg
};

#[tracker::track]
pub struct MainView {
    xrservice_active: bool,
    xrservice_type: XRServiceType,
    enable_debug_view: bool,
    profile_names: Vec<String>,
    steam_launch_options: String,
    #[tracker::do_not_track]
    profiles_dropdown: Option<gtk::DropDown>,
    #[tracker::do_not_track]
    install_wivrn_box: Controller<InstallWivrnBox>,
}

#[derive(Debug)]
pub enum MainViewMsg {
    ClockTicking,
    StartStopClicked,
    XRServiceActiveChanged(bool),
    EnableDebugViewChanged(bool),
    UpdateProfileNames(Vec<String>, Config),
    SetSelectedProfile(u32),
    ProfileSelected(u32),
    SteamLaunchOptionsChanged(String),
    CopySteamLaunchOptions,
    UpdateSelectedProfile(Profile),
}

#[derive(Debug)]
pub enum MainViewOutMsg {
    UpdateView,
    EnableDebugViewChanged(bool),
    DoStartStopXRService,
    ProfileSelected(String),
    SetXRServiceRuntime(bool),
}

pub struct MainViewInit {
    pub config: Config,
    pub selected_profile: Profile,
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
                "_Calibrate Lighthouses" => LibsurviveSetupAction,
            },
            section! {
                "_About" => AboutAction,
            }
        }
    }

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            #[track = "model.changed(Self::enable_debug_view())"]
            set_hexpand: !model.enable_debug_view,
            set_vexpand: true,
            set_size_request: (270, 350),
            gtk::WindowHandle {
                set_hexpand: true,
                set_vexpand: false,
                adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget: title_label = &gtk::Label {
                        set_label: APP_NAME,
                        add_css_class: "title",
                    },
                    pack_end: menu_btn = &gtk::MenuButton {
                        set_icon_name: icon_name::MENU,
                        set_menu_model: Some(&app_menu),
                    },
                    pack_start: profiles_dropdown = &gtk::DropDown {
                        set_tooltip_text: Some("Profiles"),
                        #[track = "model.changed(Self::profile_names())"]
                        set_model: Some(&{
                            let names: Vec<_> = model.profile_names.iter().map(String::as_str).collect();
                            gtk::StringList::new(&names)
                        }),
                        connect_selected_item_notify[sender] => move |this| {
                            sender.input(MainViewMsg::ProfileSelected(this.selected()));
                        },
                    },
                    #[track = "model.changed(Self::enable_debug_view())"]
                    set_show_end_title_buttons: !model.enable_debug_view,
                },
            },
            gtk::ScrolledWindow {
                set_hscrollbar_policy: gtk::PolicyType::Never,
                set_hexpand: true,
                set_vexpand: true,
                gtk::Box {
                    set_spacing: 12,
                    set_margin_top: 12,
                    set_margin_bottom: 12,
                    set_orientation: gtk::Orientation::Vertical,
                    gtk::Button {
                        add_css_class: "pill",
                        add_css_class: "suggested-action",
                        add_css_class: "destructive-action",
                        set_hexpand: true,
                        set_halign: gtk::Align::Center,
                        #[track = "model.changed(Self::xrservice_active())"]
                        set_class_active: ("suggested-action", !model.xrservice_active),
                        #[track = "model.changed(Self::xrservice_active())"]
                        set_label: match model.xrservice_active {
                            true => "Stop",
                            false => "Start",
                        },
                        connect_clicked[sender] => move |_| {
                            sender.input(MainViewMsg::StartStopClicked)
                        },
                    },
                    gtk::Separator {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_hexpand: true,
                    },
                    gtk::Label {
                        add_css_class: "heading",
                        set_hexpand: true,
                        set_xalign: 0.0,
                        set_margin_start: 12,
                        set_margin_end: 12,
                        set_label: "OpenXR/OpenVR Runtime",
                        set_wrap: true,
                        set_wrap_mode: gtk::pango::WrapMode::Word,
                    },
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_hexpand: true,
                        set_halign: gtk::Align::Start,
                        set_margin_start: 12,
                        set_margin_end: 12,
                        set_spacing: 12,
                        gtk::Label {
                            set_label: "Steam",
                        },
                        #[name(runtime_switch)]
                        gtk::Switch {
                        },
                        gtk::Label {
                            set_label: APP_NAME,
                        },
                    },
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_hexpand: true,
                        set_vexpand: false,
                        set_spacing: 12,
                        set_margin_top: 12,
                        set_margin_bottom: 12,
                        #[track = "model.changed(Self::xrservice_active())"]
                        set_visible: model.xrservice_active,
                        gtk::Separator {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_hexpand: true,
                        },
                        gtk::Label {
                            add_css_class: "heading",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_margin_start: 12,
                            set_margin_end: 12,
                            set_label: "Steam Launch Options",
                            set_wrap: true,
                            set_wrap_mode: gtk::pango::WrapMode::Word,
                        },
                        gtk::Label {
                            add_css_class: "dim-label",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_margin_start: 12,
                            set_margin_end: 12,
                            set_label: format!(
                                "Set this string in the launch options of Steam games, so that they can pick up the {app} runtime correctly",
                                app = APP_NAME)
                                .as_str(),
                            set_wrap: true,
                            set_wrap_mode: gtk::pango::WrapMode::Word,
                        },
                        gtk::Box {
                            set_margin_start: 12,
                            set_margin_end: 12,
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 6,
                            gtk::TextView {
                                add_css_class: "card",
                                set_hexpand: true,
                                set_vexpand: false,
                                set_monospace: true,
                                set_editable: false,
                                set_wrap_mode: gtk::WrapMode::Word,
                                set_left_margin: 6,
                                set_right_margin: 6,
                                set_top_margin: 6,
                                set_bottom_margin: 6,
                                set_size_request: (-1, 150),
                                #[wrap(Some)]
                                set_buffer: cmdbuf = &gtk::TextBuffer {
                                    #[track = "model.changed(Self::steam_launch_options())"]
                                    set_text: model.steam_launch_options.as_str(),
                                }
                            },
                            gtk::Button {
                                add_css_class: "flat",
                                add_css_class: "circular",
                                set_tooltip_text: Some("Copy"),
                                set_icon_name: icon_name::COPY,
                                set_vexpand: false,
                                set_valign: gtk::Align::Center,
                                connect_clicked[sender] => move |_| {
                                    sender.input(MainViewMsg::CopySteamLaunchOptions)
                                }
                            },
                        },
                    },
                    model.install_wivrn_box.widget(),
                }
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
                sender.output(MainViewOutMsg::DoStartStopXRService);
            }
            Self::Input::XRServiceActiveChanged(active) => {
                self.set_xrservice_active(active);
            }
            Self::Input::EnableDebugViewChanged(val) => {
                self.set_enable_debug_view(val);
            }
            Self::Input::UpdateSelectedProfile(prof) => {
                self.set_xrservice_type(prof.xrservice_type.clone());
                self.install_wivrn_box
                    .sender()
                    .emit(InstallWivrnBoxMsg::UpdateSelectedProfile(prof.clone()));
            }
            Self::Input::UpdateProfileNames(names, config) => {
                self.set_profile_names(names);
                // why send another message to set the dropdown selection?
                // set_* from tracker likely updates the view obj in the next
                // draw, so selecting here will result in nothing cause the
                // dropdown is effectively empty
                sender.input(MainViewMsg::SetSelectedProfile({
                    let pos = self
                        .profile_names
                        .iter()
                        .position(|p| p.clone() == config.selected_profile_name);
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
            }
            Self::Input::SteamLaunchOptionsChanged(lo) => self.set_steam_launch_options(lo),
            Self::Input::CopySteamLaunchOptions => gtk::gdk::Display::default()
                .expect_dialog("Could not find default display")
                .clipboard()
                .set_text(self.steam_launch_options.as_str()),
            Self::Input::ProfileSelected(position) => {
                // self.install_wivrn_box.sender.emit(InstallWivrnBoxMsg::Upda);
                // TODO: send profile to install_wivrn_box
                sender.output(MainViewOutMsg::ProfileSelected(
                    self.profile_names.get(position as usize).unwrap().clone(),
                ));
            }
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = Self {
            xrservice_active: false,
            xrservice_type: XRServiceType::Monado,
            enable_debug_view: init.config.debug_view_enabled,
            profiles_dropdown: None,
            profile_names: vec![],
            steam_launch_options: "".into(),
            install_wivrn_box: InstallWivrnBox::builder()
                .launch(InstallWivrnBoxInit {
                    selected_profile: init.selected_profile,
                })
                .detach(),
            tracker: 0,
        };
        let widgets = view_output!();

        {
            match get_current_active_runtime() {
                None => {}
                Some(runtime) => {
                    widgets
                        .runtime_switch
                        .set_active(!active_runtime_json::is_steam(runtime));
                }
            }
            widgets.runtime_switch.connect_state_set(move |_, state| {
                sender.output(MainViewOutMsg::SetXRServiceRuntime(state));
                gtk::Inhibit(false)
            });
        }

        model.profiles_dropdown = Some(widgets.profiles_dropdown.clone());

        ComponentParts { model, widgets }
    }
}
