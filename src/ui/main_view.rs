use crate::config::Config;
use crate::constants::APP_NAME;
use crate::file_builders::active_runtime_json::{self, get_current_active_runtime};
use crate::ui::app::{AboutAction, BuildProfileAction, DebugViewToggleAction, LibsurviveSetupAction};
use expect_dialog::ExpectDialog;
use gtk::prelude::*;
use relm4::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use relm4_icons::icon_name;

#[tracker::track]
pub struct MainView {
    monado_active: bool,
    enable_debug_view: bool,
    profile_names: Vec<String>,
    steam_launch_options: String,
}

#[derive(Debug)]
pub enum MainViewMsg {
    StartStopClicked,
    MonadoActiveChanged(bool),
    EnableDebugViewChanged(bool),
    UpdateProfileNames(Vec<String>),
    SteamLaunchOptionsChanged(String),
    CopySteamLaunchOptions,
}

#[derive(Debug)]
pub enum MainViewOutMsg {
    UpdateView,
    EnableDebugViewChanged(bool),
    DoStartStopMonado,
    ProfileSelected(String),
    SetMonadoRuntime(bool),
}

pub struct MainViewInit {
    pub config: Config,
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
            #[track = "model.changed(MainView::enable_debug_view())"]
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
                        #[track = "model.changed(MainView::profile_names())"]
                        set_model: Some(&{
                            let names: Vec<_> = model.profile_names.iter().map(String::as_str).collect();
                            gtk::StringList::new(&names)
                        }),
                    },
                    #[track = "model.changed(MainView::enable_debug_view())"]
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
                        #[track = "model.changed(MainView::monado_active())"]
                        set_class_active: ("suggested-action", !model.monado_active),
                        #[track = "model.changed(MainView::monado_active())"]
                        set_label: match model.monado_active {
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
                            set_label: "Monado"
                        },
                    },
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_hexpand: true,
                        set_vexpand: false,
                        set_spacing: 12,
                        set_margin_top: 12,
                        set_margin_bottom: 12,
                        #[track = "model.changed(MainView::monado_active())"]
                        set_visible: model.monado_active,
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
                            set_label: "Set this string in the launch options of Steam games, so that they can pick up Monado and OpenOVR correctly",
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
                                    #[track = "model.changed(MainView::steam_launch_options())"]
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
                    }
                }
            }
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        self.reset();

        match message {
            MainViewMsg::StartStopClicked => {
                sender.output(MainViewOutMsg::DoStartStopMonado);
            }
            MainViewMsg::MonadoActiveChanged(active) => {
                self.set_monado_active(active);
            }
            MainViewMsg::EnableDebugViewChanged(val) => {
                self.set_enable_debug_view(val);
            }
            MainViewMsg::UpdateProfileNames(names) => {
                self.set_profile_names(names);
            }
            MainViewMsg::SteamLaunchOptionsChanged(lo) => self.set_steam_launch_options(lo),
            MainViewMsg::CopySteamLaunchOptions => gtk::gdk::Display::default()
                .expect_dialog("Could not find default display")
                .clipboard()
                .set_text(self.steam_launch_options.as_str()),
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = MainView {
            monado_active: false,
            enable_debug_view: init.config.debug_view_enabled,
            profile_names: vec![],
            steam_launch_options: "".into(),
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
                sender.output(MainViewOutMsg::SetMonadoRuntime(state));
                gtk::Inhibit(false)
            });
        }

        ComponentParts { model, widgets }
    }
}
