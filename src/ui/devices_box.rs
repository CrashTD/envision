use crate::{
    file_builders::monado_config_v0::dump_generic_trackers,
    xr_devices::{XRDevice, XRDevices},
};
use adw::prelude::*;
use relm4::prelude::*;

use super::alert::alert;

#[tracker::track]
pub struct DevicesBox {
    devices: Option<XRDevices>,
}

#[derive(Debug)]
pub enum DevicesBoxMsg {
    UpdateDevices(Option<XRDevices>),
    DumpGenericTrackers,
}

impl DevicesBox {
    fn get_dev(&self, key: XRDevice) -> Option<String> {
        match &self.devices {
            None => None,
            Some(devs) => match key {
                XRDevice::Head => devs.head.clone(),
                XRDevice::Left => devs.left.clone(),
                XRDevice::Right => devs.right.clone(),
                XRDevice::Gamepad => devs.gamepad.clone(),
                XRDevice::Eyes => devs.eyes.clone(),
                XRDevice::HandTrackingLeft => devs.hand_tracking_left.clone(),
                XRDevice::HandTrackingRight => devs.hand_tracking_right.clone(),
                XRDevice::GenericTracker => {
                    if devs.generic_trackers.is_empty() {
                        return None;
                    } else {
                        return Some(devs.generic_trackers.join(", "));
                    }
                }
            },
        }
    }

    fn get_dev_or_none(&self, key: XRDevice) -> String {
        self.get_dev(key).unwrap_or("None".into())
    }
}

#[relm4::component(pub)]
impl SimpleComponent for DevicesBox {
    type Init = ();
    type Input = DevicesBoxMsg;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            set_vexpand: false,
            set_spacing: 12,
            set_margin_top: 12,
            #[track = "model.changed(Self::devices())"]
            set_visible: model.devices.is_some(),
            gtk::ListBox {
                add_css_class: "boxed-list",
                set_selection_mode: gtk::SelectionMode::None,
                set_margin_all: 12,
                // Head
                adw::ActionRow {
                    #[track = "model.changed(Self::devices())"]
                    set_icon_name: Some(match model.get_dev(XRDevice::Head) {
                        Some(name) => match name.as_str() {
                            "Simulated HMD" => "dialog-warning-symbolic",
                            _ => "emblem-ok-symbolic",
                        },
                        None => "dialog-question-symbolic",
                    }),
                    #[track = "model.changed(Self::devices())"]
                    set_class_active: ("error", model.get_dev(XRDevice::Head).is_none()),
                    #[track = "model.changed(Self::devices())"]
                    set_class_active: ("warning", model.get_dev_or_none(XRDevice::Head) == "Simulated HMD"),
                    set_title: "Head",
                    #[track = "model.changed(Self::devices())"]
                    set_subtitle: match model.get_dev_or_none(XRDevice::Head).as_str() {
                        "Simulated HMD" => "No HMD detected (Simulated HMD)",
                        s => s,
                    },
                    // TODO: status icon with popover
                },
                // Left
                adw::ActionRow {
                    #[track = "model.changed(Self::devices())"]
                    set_icon_name: Some(match model.get_dev(XRDevice::Left) {
                        Some(_) => "emblem-ok-symbolic",
                        None => "dialog-question-symbolic",
                    }),
                    #[track = "model.changed(Self::devices())"]
                    set_class_active: ("error", model.get_dev(XRDevice::Left).is_none()),
                    set_title: "Left",
                    #[track = "model.changed(Self::devices())"]
                    set_subtitle: model.get_dev_or_none(XRDevice::Left).as_str(),
                    // TODO: status icon with popover
                },
                // Right
                adw::ActionRow {
                    #[track = "model.changed(Self::devices())"]
                    set_icon_name: Some(match model.get_dev(XRDevice::Right) {
                        Some(_) => "emblem-ok-symbolic",
                        None => "dialog-question-symbolic",
                    }),
                    #[track = "model.changed(Self::devices())"]
                    set_class_active: ("error", model.get_dev(XRDevice::Right).is_none()),
                    set_title: "Right",
                    #[track = "model.changed(Self::devices())"]
                    set_subtitle: model.get_dev_or_none(XRDevice::Right).as_str(),
                    // TODO: status icon with popover
                },
                // Gamepad
                adw::ActionRow {
                    #[track = "model.changed(Self::devices())"]
                    set_visible: model.get_dev(XRDevice::Gamepad).is_some(),
                    set_icon_name: Some("emblem-ok-symbolic"),
                    set_title: "Gamepad",
                    #[track = "model.changed(Self::devices())"]
                    set_subtitle: model.get_dev_or_none(XRDevice::Gamepad).as_str(),
                },
                // Eyes
                adw::ActionRow {
                    #[track = "model.changed(Self::devices())"]
                    set_visible: model.get_dev(XRDevice::Eyes).is_some(),
                    set_icon_name: Some("emblem-ok-symbolic"),
                    set_title: "Eye Tracking",
                    #[track = "model.changed(Self::devices())"]
                    set_subtitle: model.get_dev_or_none(XRDevice::Eyes).as_str(),
                },
                // Hand Tracking Left
                adw::ActionRow {
                    #[track = "model.changed(Self::devices())"]
                    set_visible: model.get_dev(XRDevice::HandTrackingLeft).is_some(),
                    set_icon_name: Some("emblem-ok-symbolic"),
                    set_title: "Hand Tracking Left",
                    #[track = "model.changed(Self::devices())"]
                    set_subtitle: model.get_dev_or_none(XRDevice::HandTrackingLeft).as_str(),
                },
                // Hand Tracking Right
                adw::ActionRow {
                    #[track = "model.changed(Self::devices())"]
                    set_visible: model.get_dev(XRDevice::HandTrackingRight).is_some(),
                    set_icon_name: Some("emblem-ok-symbolic"),
                    set_title: "Hand Tracking Right",
                    #[track = "model.changed(Self::devices())"]
                    set_subtitle: model.get_dev_or_none(XRDevice::HandTrackingRight).as_str(),
                },
                // Generic Trackers
                adw::ActionRow {
                    #[track = "model.changed(Self::devices())"]
                    set_visible: model.get_dev(XRDevice::GenericTracker).is_some(),
                    set_icon_name: Some("emblem-ok-symbolic"),
                    set_title: "Generic Trackers",
                    #[track = "model.changed(Self::devices())"]
                    set_subtitle: model.get_dev_or_none(XRDevice::GenericTracker).as_str(),
                    add_suffix: save_trackers_btn = &gtk::Button {
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,
                        set_icon_name: "document-save-symbolic",
                        set_tooltip_text: Some("Save current trackers"),
                        set_css_classes: &["circular", "flat"],
                        connect_clicked => move |_| {
                            sender.input(Self::Input::DumpGenericTrackers);
                        }
                    },
                },
            }
        }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        self.reset();

        match message {
            Self::Input::UpdateDevices(devs) => {
                self.set_devices(devs);
            }
            Self::Input::DumpGenericTrackers => {
                if let Some(devs) = self.devices.as_ref() {
                    let added = dump_generic_trackers(&devs.generic_trackers);
                    let multi_title = format!("Added {} new trackers", added);
                    let (title, msg) = match added {
                        0 => (
                            "No new trackers found",
                            "All the currently connected trackers are already present in your configuration"
                        ),
                        1 => (
                            "Added 1 new tracker",
                            "Edit your configuration to make sure that all the trackers have the appropriate roles assigned"
                        ),
                        _ => (
                            multi_title.as_str(),
                            "Edit your configuration to make sure that all the trackers have the appropriate roles assigned"
                        ),
                    };
                    alert(title, Some(msg), None);
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            tracker: 0,
            devices: None,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
