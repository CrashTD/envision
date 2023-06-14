use gtk::prelude::*;
use relm4::prelude::*;
use relm4::{SimpleComponent, ComponentSender, ComponentParts};
use relm4_icons::icon_name;
use crate::config::Config;
use crate::constants::APP_NAME;
use crate::ui::app::{AboutAction, DebugViewToggleAction, BuildProfileAction};

#[tracker::track]
pub struct MainView {
    monado_active: bool,
    enable_debug_view: bool,
    profile_names: Vec<String>,
}

#[derive(Debug)]
pub enum MainViewMsg {
    StartStopClicked,
    MonadoActiveChanged(bool),
    EnableDebugViewChanged(bool),
    UpdateProfileNames(Vec<String>),
}

#[derive(Debug)]
pub enum MainViewOutMsg {
    UpdateView,
    EnableDebugViewChanged(bool),
    DoStartStopMonado,
    ProfileSelected(String),
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
            gtk::Box {
                set_spacing: 12,
                set_margin_all: 12,
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
                    }
                },
            }
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        self.reset();

        match message {
            MainViewMsg::StartStopClicked => {
                sender.output(MainViewOutMsg::DoStartStopMonado);
            },
            MainViewMsg::MonadoActiveChanged(active) => {
                self.set_monado_active(active);
            }
            MainViewMsg::EnableDebugViewChanged(val) => {
                self.set_enable_debug_view(val);
            }
            MainViewMsg::UpdateProfileNames(names) => {
                self.set_profile_names(names);
            }
        }
    }

    fn init(init: Self::Init, root: &Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = MainView {
            monado_active: false,
            enable_debug_view: init.config.debug_view_enabled,
            profile_names: vec![],
            tracker: 0,
        };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
