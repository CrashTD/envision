use super::util::copy_text;
use crate::{constants::APP_NAME, profile::Profile};
use gtk::prelude::*;
use relm4::prelude::*;

#[tracker::track]
pub struct SteamLaunchOptionsBox {
    xrservice_active: bool,
    launch_options: String,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum SteamLaunchOptionsBoxMsg {
    UpdateXRServiceActive(bool),
    UpdateLaunchOptions(Profile),
    CopyLaunchOptions,
}

#[relm4::component(pub)]
impl SimpleComponent for SteamLaunchOptionsBox {
    type Init = ();
    type Input = SteamLaunchOptionsBoxMsg;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            set_vexpand: false,
            set_spacing: 12,
            add_css_class: "card",
            add_css_class: "padded",
            #[track = "model.changed(Self::xrservice_active())"]
            set_visible: model.xrservice_active,
            gtk::Label {
                add_css_class: "heading",
                set_hexpand: true,
                set_xalign: 0.0,
                set_label: "Steam Launch Options",
                set_wrap: true,
                set_wrap_mode: gtk::pango::WrapMode::Word,
            },
            gtk::Label {
                add_css_class: "dim-label",
                set_hexpand: true,
                set_label: format!(
                    "Set this string in the launch options of Steam games, so that they can pick up the {app} runtime correctly",
                    app = APP_NAME)
                    .as_str(),
                set_xalign: 0.0,
                set_wrap: true,
                set_wrap_mode: gtk::pango::WrapMode::Word,
            },
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 6,
                gtk::ScrolledWindow {
                    add_css_class: "card",
                    set_vscrollbar_policy: gtk::PolicyType::Never,
                    set_overflow: gtk::Overflow::Hidden,
                    gtk::TextView {
                        set_hexpand: true,
                        set_vexpand: false,
                        set_monospace: true,
                        set_editable: false,
                        set_left_margin: 6,
                        set_right_margin: 6,
                        set_top_margin: 6,
                        set_bottom_margin: 18,
                        #[wrap(Some)]
                        set_buffer: cmdbuf = &gtk::TextBuffer {
                            #[track = "model.changed(Self::launch_options())"]
                            set_text: model.launch_options.as_str(),
                            set_enable_undo: false,
                        }
                    }
                },
                gtk::Button {
                    add_css_class: "flat",
                    add_css_class: "circular",
                    set_tooltip_text: Some("Copy"),
                    set_icon_name: "edit-copy-symbolic",
                    set_vexpand: false,
                    set_valign: gtk::Align::Center,
                    connect_clicked[sender] => move |_| {
                        sender.input(Self::Input::CopyLaunchOptions)
                    }
                },
            },
        }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        self.reset();

        match message {
            Self::Input::UpdateXRServiceActive(val) => {
                self.set_xrservice_active(val);
            }
            Self::Input::UpdateLaunchOptions(prof) => {
                self.set_launch_options(prof.get_steam_launch_options());
            }
            Self::Input::CopyLaunchOptions => {
                copy_text(self.launch_options.as_str());
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            xrservice_active: false,
            launch_options: "".into(),
            tracker: 0,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
