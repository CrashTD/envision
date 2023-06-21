use super::install_wivrn_box::{InstallWivrnBox, InstallWivrnBoxInit, InstallWivrnBoxMsg};
use super::runtime_switcher_box::{
    RuntimeSwitcherBox, RuntimeSwitcherBoxInit, RuntimeSwitcherBoxMsg,
};
use super::steam_launch_options_box::{SteamLaunchOptionsBox, SteamLaunchOptionsBoxMsg};
use crate::config::Config;
use crate::constants::APP_NAME;
use crate::profile::{Profile, XRServiceType};
use crate::ui::app::{
    AboutAction, BuildProfileAction, DebugViewToggleAction, LibsurviveSetupAction,
};
use gtk::prelude::*;
use relm4::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use relm4_icons::icon_name;

#[tracker::track]
pub struct MainView {
    xrservice_active: bool,
    xrservice_type: XRServiceType,
    enable_debug_view: bool,
    profile_names: Vec<String>,
    #[tracker::do_not_track]
    profiles_dropdown: Option<gtk::DropDown>,
    #[tracker::do_not_track]
    install_wivrn_box: Controller<InstallWivrnBox>,
    #[tracker::do_not_track]
    steam_launch_options_box: Controller<SteamLaunchOptionsBox>,
    #[tracker::do_not_track]
    runtime_switcher_box: Controller<RuntimeSwitcherBox>,
}

#[derive(Debug)]
pub enum MainViewMsg {
    ClockTicking,
    StartStopClicked,
    XRServiceActiveChanged(bool, Option<Profile>),
    EnableDebugViewChanged(bool),
    UpdateProfileNames(Vec<String>, Config),
    SetSelectedProfile(u32),
    ProfileSelected(u32),
    UpdateSelectedProfile(Profile),
}

#[derive(Debug)]
pub enum MainViewOutMsg {
    EnableDebugViewChanged(bool),
    DoStartStopXRService,
    ProfileSelected(String),
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
                    model.runtime_switcher_box.widget(),
                    model.steam_launch_options_box.widget(),
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
            Self::Input::XRServiceActiveChanged(active, profile) => {
                self.set_xrservice_active(active);
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
            }
            Self::Input::UpdateSelectedProfile(prof) => {
                self.set_xrservice_type(prof.xrservice_type.clone());
                self.install_wivrn_box
                    .sender()
                    .emit(InstallWivrnBoxMsg::UpdateSelectedProfile(prof.clone()));
                self.runtime_switcher_box
                    .sender()
                    .emit(RuntimeSwitcherBoxMsg::UpdateSelectedProfile(prof.clone()));
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
            steam_launch_options_box: SteamLaunchOptionsBox::builder().launch(()).detach(),
            install_wivrn_box: InstallWivrnBox::builder()
                .launch(InstallWivrnBoxInit {
                    selected_profile: init.selected_profile.clone(),
                })
                .detach(),
            runtime_switcher_box: RuntimeSwitcherBox::builder()
                .launch(RuntimeSwitcherBoxInit {
                    selected_profile: init.selected_profile.clone(),
                })
                .detach(),
            tracker: 0,
        };
        let widgets = view_output!();

        model.profiles_dropdown = Some(widgets.profiles_dropdown.clone());

        ComponentParts { model, widgets }
    }
}
