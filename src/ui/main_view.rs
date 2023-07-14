use super::devices_box::{DevicesBox, DevicesBoxMsg};
use super::install_wivrn_box::{InstallWivrnBox, InstallWivrnBoxInit, InstallWivrnBoxMsg};
use super::profile_editor::{ProfileEditor, ProfileEditorMsg, ProfileEditorOutMsg};
use super::steam_launch_options_box::{SteamLaunchOptionsBox, SteamLaunchOptionsBoxMsg};
use crate::config::Config;
use crate::constants::APP_NAME;
use crate::profile::{Profile, XRServiceType};
use crate::ui::app::{
    AboutAction, BuildProfileAction, DebugViewToggleAction, LibsurviveSetupAction,
};
use crate::ui::profile_editor::ProfileEditorInit;
use crate::xr_devices::XRDevices;
use gtk::prelude::*;
use relm4::adw::traits::MessageDialogExt;
use relm4::adw::ResponseAppearance;
use relm4::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};

#[tracker::track]
pub struct MainView {
    xrservice_active: bool,
    enable_debug_view: bool,
    profiles: Vec<Profile>,
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
    profile_editor: Controller<ProfileEditor>,
    #[tracker::do_not_track]
    profile_not_editable_dialog: adw::MessageDialog,
    #[tracker::do_not_track]
    profile_delete_confirm_dialog: adw::MessageDialog,
    #[tracker::do_not_track]
    cannot_duplicate_profile_dialog: adw::MessageDialog,
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
    UpdateDevices(Option<XRDevices>),
}

#[derive(Debug)]
pub enum MainViewOutMsg {
    EnableDebugViewChanged(bool),
    DoStartStopXRService,
    RestartXRService,
    ProfileSelected(Profile),
    DeleteProfile,
    SaveProfile(Profile),
}

pub struct MainViewInit {
    pub config: Config,
    pub selected_profile: Profile,
    pub root_win: gtk::Window,
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
                        set_icon_name: "open-menu-symbolic",
                        set_menu_model: Some(&app_menu),
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
                    gtk::Box {
                        set_halign: gtk::Align::Center,
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 12,
                        gtk::Button {
                            add_css_class: "pill",
                            add_css_class: "suggested-action",
                            add_css_class: "destructive-action",
                            set_halign: gtk::Align::Center,
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
                            add_css_class: "circular",
                            add_css_class: "flat",
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
                    model.steam_launch_options_box.widget(),
                    model.install_wivrn_box.widget(),
                }
            },
            gtk::Separator {
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,
            },
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 12,
                add_css_class: "toolbar",
                add_css_class: "view",
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    add_css_class: "linked",
                    #[name(profiles_dropdown)]
                    gtk::DropDown {
                        set_hexpand: true,
                        set_tooltip_text: Some("Profiles"),
                        #[track = "model.changed(Self::profiles())"]
                        set_model: Some(&{
                            let names: Vec<_> = model.profiles.iter().map(|p| p.name.as_str()).collect();
                            gtk::StringList::new(&names)
                        }),
                        connect_selected_item_notify[sender] => move |this| {
                            sender.input(MainViewMsg::ProfileSelected(this.selected()));
                        },
                    },
                    gtk::Button {
                        set_icon_name: "edit-symbolic",
                        set_tooltip_text: Some("Edit Profile"),
                        connect_clicked[sender] => move |_| {
                            sender.input(Self::Input::EditProfile);
                        }
                    },
                    gtk::Button {
                        set_icon_name: "edit-copy-symbolic",
                        set_tooltip_text: Some("Duplicate Profile"),
                        connect_clicked[sender] => move |_| {
                            sender.input(Self::Input::DuplicateProfile);
                        }
                    },
                    gtk::Button {
                        set_icon_name: "list-add-symbolic",
                        set_tooltip_text: Some("Create Profile"),
                        connect_clicked[sender] => move |_| {
                            sender.input(Self::Input::CreateProfile);
                        }
                    },
                    gtk::Button {
                        set_icon_name: "edit-delete-symbolic",
                        add_css_class: "destructive-action",
                        set_tooltip_text: Some("Delete Profile"),
                        #[track = "model.changed(Self::selected_profile())"]
                        set_sensitive: model.selected_profile.editable,
                        connect_clicked[sender] => move |_| {
                            sender.input(Self::Input::DeleteProfile);
                        }
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
                sender.output(Self::Output::DoStartStopXRService);
            }
            Self::Input::RestartXRService => {
                sender.output(Self::Output::RestartXRService);
            }
            Self::Input::XRServiceActiveChanged(active, profile) => {
                self.set_xrservice_active(active);
                if !active {
                    sender.input(Self::Input::UpdateDevices(None));
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
            }
            Self::Input::UpdateSelectedProfile(prof) => {
                self.set_selected_profile(prof.clone());
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
                sender.output(MainViewOutMsg::ProfileSelected(
                    self.profiles.get(position as usize).unwrap().clone(),
                ));
            }
            Self::Input::EditProfile => {
                if self.selected_profile.editable {
                    self.profile_editor
                        .emit(ProfileEditorMsg::Present(self.selected_profile.clone()));
                } else {
                    self.profile_not_editable_dialog.present();
                }
            }
            Self::Input::CreateProfile => {
                self.profile_editor
                    .sender()
                    .emit(ProfileEditorMsg::Present(Profile::default()));
            }
            Self::Input::DeleteProfile => {
                self.profile_delete_confirm_dialog.present();
            }
            Self::Input::SaveProfile(prof) => {
                sender.output(Self::Output::SaveProfile(prof));
            }
            Self::Input::DuplicateProfile => {
                if self.selected_profile.can_be_built {
                    self.profile_editor.sender().emit(ProfileEditorMsg::Present(
                        self.selected_profile.create_duplicate(),
                    ));
                } else {
                    self.cannot_duplicate_profile_dialog.present();
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
        root: &Self::Root,
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
                    pdc_sender.output(Self::Output::DeleteProfile);
                }
            });
        }

        let cannot_duplicate_profile_dialog = adw::MessageDialog::builder()
            .modal(true)
            .transient_for(&init.root_win)
            .hide_on_close(true)
            .heading("This profile cannot be duplicated")
            .build();
        cannot_duplicate_profile_dialog.add_response("ok", "_Ok");

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
            profile_editor: ProfileEditor::builder()
                .launch(ProfileEditorInit {
                    root_win: init.root_win.clone(),
                })
                .forward(sender.input_sender(), |message| match message {
                    ProfileEditorOutMsg::SaveProfile(p) => Self::Input::SaveProfile(p),
                }),
            devices_box: DevicesBox::builder().launch(()).detach(),
            selected_profile: init.selected_profile.clone(),
            profile_not_editable_dialog,
            profile_delete_confirm_dialog,
            cannot_duplicate_profile_dialog,
            tracker: 0,
        };
        let widgets = view_output!();

        model.profiles_dropdown = Some(widgets.profiles_dropdown.clone());

        ComponentParts { model, widgets }
    }
}
