use crate::xr_devices::{XRDevice, XRDevices};
use gtk::prelude::*;
use relm4::prelude::*;

#[tracker::track]
pub struct DevicesBox {
    devices: Option<XRDevices>,
}

#[derive(Debug)]
pub enum DevicesBoxMsg {
    UpdateDevices(Option<XRDevices>),
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
        }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        self.reset();

        match message {
            Self::Input::UpdateDevices(devs) => {
                self.set_devices(devs);
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            tracker: 0,
            devices: None,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
