use crate::{
    file_builders::monado_config_v0::dump_generic_trackers,
    xr_devices::{XRDevice, XRDevices},
};
use gtk::prelude::*;
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
            gtk::Separator {
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,
            },
            // Head
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,
                set_spacing: 12,
                set_margin_start: 12,
                set_margin_end: 12,
                gtk::Image {
                    #[track = "model.changed(Self::devices())"]
                    set_icon_name: Some(match model.get_dev(XRDevice::Head) {
                        Some(_) => "emblem-ok-symbolic",
                        None => "dialog-question-symbolic",
                    }),
                    #[track = "model.changed(Self::devices())"]
                    set_class_active: ("error", model.get_dev(XRDevice::Head).is_none()),
                },
                gtk::Label {
                    set_xalign: 0.0,
                    set_hexpand: true,
                    #[track = "model.changed(Self::devices())"]
                    set_label: format!("Head: {}", match model.get_dev(XRDevice::Head) {
                        Some(v) => v.clone(),
                        None => "None".to_string(),
                    }).as_str(),
                    #[track = "model.changed(Self::devices())"]
                    set_class_active: ("error", model.get_dev(XRDevice::Head).is_none()),
                    // TODO: status icon with popover
                },
            },
            // Left
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,
                set_spacing: 12,
                set_margin_start: 12,
                set_margin_end: 12,
                gtk::Image {
                    #[track = "model.changed(Self::devices())"]
                    set_icon_name: Some(match model.get_dev(XRDevice::Left) {
                        Some(_) => "emblem-ok-symbolic",
                        None => "dialog-question-symbolic",
                    }),
                    #[track = "model.changed(Self::devices())"]
                    set_class_active: ("error", model.get_dev(XRDevice::Left).is_none()),
                },
                gtk::Label {
                    set_xalign: 0.0,
                    set_hexpand: true,
                    #[track = "model.changed(Self::devices())"]
                    set_label: format!("Left: {}", match model.get_dev(XRDevice::Left) {
                        Some(v) => v.clone(),
                        None => "None".to_string(),
                    }).as_str(),
                    #[track = "model.changed(Self::devices())"]
                    set_class_active: ("error", model.get_dev(XRDevice::Left).is_none()),
                    // TODO: status icon with popover
                },
            },
            // Right
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,
                set_spacing: 12,
                set_margin_start: 12,
                set_margin_end: 12,
                gtk::Image {
                    #[track = "model.changed(Self::devices())"]
                    set_icon_name: Some(match model.get_dev(XRDevice::Right) {
                        Some(_) => "emblem-ok-symbolic",
                        None => "dialog-question-symbolic",
                    }),
                    #[track = "model.changed(Self::devices())"]
                    set_class_active: ("error", model.get_dev(XRDevice::Right).is_none()),
                },
                gtk::Label {
                    set_xalign: 0.0,
                    set_hexpand: true,
                    #[track = "model.changed(Self::devices())"]
                    set_label: format!("Right: {}", match model.get_dev(XRDevice::Right) {
                        Some(v) => v.clone(),
                        None => "None".to_string(),
                    }).as_str(),
                    #[track = "model.changed(Self::devices())"]
                    set_class_active: ("error", model.get_dev(XRDevice::Right).is_none()),
                    // TODO: status icon with popover
                },
            },
            // Gamepad
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,
                set_spacing: 12,
                set_margin_start: 12,
                set_margin_end: 12,
                #[track = "model.changed(Self::devices())"]
                set_visible: model.get_dev(XRDevice::Gamepad).is_some(),
                gtk::Image {
                    set_icon_name: Some("emblem-ok-symbolic"),
                },
                gtk::Label {
                    set_xalign: 0.0,
                    set_hexpand: true,
                    #[track = "model.changed(Self::devices())"]
                    set_label: format!("Gamepad: {}", match model.get_dev(XRDevice::Gamepad) {
                        Some(v) => v.clone(),
                        None => "None".to_string(),
                    }).as_str(),
                },
            },
            // Eyes
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,
                set_spacing: 12,
                set_margin_start: 12,
                set_margin_end: 12,
                #[track = "model.changed(Self::devices())"]
                set_visible: model.get_dev(XRDevice::Eyes).is_some(),
                gtk::Image {
                    set_icon_name: Some("emblem-ok-symbolic"),
                },
                gtk::Label {
                    set_xalign: 0.0,
                    set_hexpand: true,
                    #[track = "model.changed(Self::devices())"]
                    set_label: format!("Eye Tracking: {}", match model.get_dev(XRDevice::Eyes) {
                        Some(v) => v.clone(),
                        None => "None".to_string(),
                    }).as_str(),
                },
            },
            // Hand Tracking Left
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,
                set_spacing: 12,
                set_margin_start: 12,
                set_margin_end: 12,
                #[track = "model.changed(Self::devices())"]
                set_visible: model.get_dev(XRDevice::HandTrackingLeft).is_some(),
                gtk::Image {
                    set_icon_name: Some("emblem-ok-symbolic"),
                },
                gtk::Label {
                    set_xalign: 0.0,
                    set_hexpand: true,
                    #[track = "model.changed(Self::devices())"]
                    set_label: format!("Hand Tracking Left: {}", match model.get_dev(XRDevice::HandTrackingLeft) {
                        Some(v) => v.clone(),
                        None => "None".to_string(),
                    }).as_str(),
                },
            },
            // Hand Tracking Right
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,
                set_spacing: 12,
                set_margin_start: 12,
                set_margin_end: 12,
                #[track = "model.changed(Self::devices())"]
                set_visible: model.get_dev(XRDevice::HandTrackingRight).is_some(),
                gtk::Image {
                    set_icon_name: Some("emblem-ok-symbolic"),
                },
                gtk::Label {
                    set_xalign: 0.0,
                    set_hexpand: true,
                    #[track = "model.changed(Self::devices())"]
                    set_label: format!("Hand Tracking Left: {}", match model.get_dev(XRDevice::HandTrackingRight) {
                        Some(v) => v.clone(),
                        None => "None".to_string(),
                    }).as_str(),
                },
            },
            // Generic Trackers
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,
                set_spacing: 12,
                set_margin_start: 12,
                set_margin_end: 12,
                #[track = "model.changed(Self::devices())"]
                set_visible: model.get_dev(XRDevice::GenericTracker).is_some(),
                gtk::Image {
                    set_icon_name: Some("emblem-ok-symbolic"),
                },
                gtk::Label {
                    set_xalign: 0.0,
                    set_hexpand: true,
                    #[track = "model.changed(Self::devices())"]
                    set_label: format!("Generic Trackers: {}", match model.get_dev(XRDevice::GenericTracker) {
                        Some(v) => v.clone(),
                        None => "None".to_string(),
                    }).as_str(),
                },
                gtk::Button {
                    set_halign: gtk::Align::Start,
                    set_hexpand: false,
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
