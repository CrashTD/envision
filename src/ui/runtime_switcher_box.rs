use relm4::prelude::*;
use gtk::prelude::*;
use crate::{constants::APP_NAME, file_builders::{active_runtime_json::{get_current_active_runtime, self, set_current_active_runtime_to_profile, set_current_active_runtime_to_steam}, openvrpaths_vrpath::{set_current_openvrpaths_to_profile, set_current_openvrpaths_to_steam}}, profile::Profile};

pub struct RuntimeSwitcherBox {
    selected_profile: Profile,
}

#[derive(Debug)]
pub enum RuntimeSwitcherBoxMsg {
    UpdateSelectedProfile(Profile),
    SetXRServiceRuntime(bool),
}

pub struct RuntimeSwitcherBoxInit {
    pub selected_profile: Profile,
}

#[relm4::component(pub)]
impl SimpleComponent for RuntimeSwitcherBox {
    type Init = RuntimeSwitcherBoxInit;
    type Input = RuntimeSwitcherBoxMsg;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            set_spacing: 12,
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
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            Self::Input::UpdateSelectedProfile(prof) => {
                self.selected_profile = prof;
            }
            Self::Input::SetXRServiceRuntime(is_rex) => {
                if is_rex {
                    set_current_active_runtime_to_profile(self.selected_profile.clone());
                    set_current_openvrpaths_to_profile(self.selected_profile.clone());
                } else {
                    set_current_active_runtime_to_steam();
                    set_current_openvrpaths_to_steam();
                }
            }
        }
    }

    fn init(
            init: Self::Init,
            root: &Self::Root,
            sender: ComponentSender<Self>,
        ) -> ComponentParts<Self> {
        let model = Self {
            selected_profile: init.selected_profile,
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
                sender.input(Self::Input::SetXRServiceRuntime(state));
                gtk::Inhibit(false)
            });
        }

        ComponentParts { model, widgets }
    }
}
