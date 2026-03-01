#![allow(unused_assignments)] // vars named in view! macro are not correctly recognized as used

use crate::{
    ui::battery_status::EnvisionBatteryStatus,
    xr_devices::{XRDevice, XRDeviceRole},
};
use adw::prelude::*;
use relm4::{AsyncFactorySender, factory::AsyncFactoryComponent, prelude::*};
use tracing::error;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceRowState {
    #[default]
    Ok,
    Error,
    Warning,
}

impl DeviceRowState {
    pub fn icon(&self) -> &str {
        match &self {
            Self::Ok => "checkmark-symbolic",
            Self::Error => "dialog-question-symbolic",
            Self::Warning => "dialog-warning-symbolic",
        }
    }

    pub fn class_name(&self) -> Option<&str> {
        match &self {
            Self::Ok => None,
            Self::Error => Some("error"),
            Self::Warning => Some("warning"),
        }
    }
}

#[tracker::track]
#[derive(Debug)]
pub struct DeviceRowModel {
    #[tracker::do_not_track]
    title: String,
    #[tracker::do_not_track]
    subtitle: String,
    state: DeviceRowState,
    battery_status: Option<EnvisionBatteryStatus>,
    brightness: Option<f32>,
    #[tracker::do_not_track]
    pub device: Option<XRDevice>,
}

#[derive(Debug, Default, Clone)]
pub struct DeviceRowModelInit {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub state: Option<DeviceRowState>,
    pub battery_status: Option<EnvisionBatteryStatus>,
    pub device: Option<XRDevice>,
    pub brightness: Option<f32>,
    pub sort_index: u32,
}

impl From<&XRDevice> for DeviceRowModelInit {
    fn from(d: &XRDevice) -> Self {
        Self {
            title: Some(if d.roles.is_empty() {
                XRDeviceRole::GenericTracker.to_string()
            } else {
                d.roles
                    .iter()
                    .map(XRDeviceRole::to_string)
                    .collect::<Vec<String>>()
                    .join("\n")
            }),
            subtitle: Some(d.name.clone()),
            battery_status: d.battery.map(EnvisionBatteryStatus::from),
            brightness: d.brightness,
            sort_index: u32::from(
                d.roles
                    .iter()
                    .min()
                    .unwrap_or(&XRDeviceRole::GenericTracker),
            ) * 1000
                + d.index,
            device: Some(d.clone()),
            ..Default::default()
        }
    }
}

impl DeviceRowModelInit {
    pub fn new_missing(t: XRDeviceRole) -> Self {
        DeviceRowModelInit {
            title: Some(t.to_string()),
            subtitle: Some("None".into()),
            state: Some(DeviceRowState::Error),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DeviceRowModelMsg {
    Update,
    BrightnessChanged(f32),
}

#[relm4::factory(async pub)]
impl AsyncFactoryComponent for DeviceRowModel {
    type Init = DeviceRowModelInit;
    type Input = DeviceRowModelMsg;
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        root = adw::ActionRow {
            add_prefix: icon = &gtk::Image {
                set_icon_name: Some(self.state.icon()),
            },
            add_suffix: brightness_btn = &gtk::MenuButton {
                add_css_class: "circular",
                add_css_class: "flat",
                set_icon_name: "display-brightness-symbolic",
                set_valign: gtk::Align::Center,
                #[track = "self.changed(Self::brightness())"]
                set_visible: self.brightness.is_some(),
                set_tooltip_text: Some("Adjust brightness"),
                #[wrap(Some)]
                set_popover: brightness_popover = &gtk::Popover {
                    gtk::Box {
                        set_margin_all: 6,
                        gtk::Scale {
                            set_width_request: 250,
                            set_digits: 3,
                            set_draw_value: true,
                            set_value_pos: gtk::PositionType::Right,
                            set_format_value_func: move |_, v| {
                                format!("{:.0}%", v*100.0)
                            },
                            add_mark: (1.0, gtk::PositionType::Bottom, Some("100%")),
                            set_adjustment: adj = &gtk::Adjustment {
                                set_lower: 0.0,
                                set_upper: 1.4,
                                set_step_increment: 0.01,
                                set_page_increment: 0.1,
                                #[track = "self.changed(Self::brightness())"]
                                set_value: self.brightness.unwrap_or(0.0).into(),
                                connect_value_changed[sender] => move |adj| {
                                    sender.input(Self::Input::BrightnessChanged(adj.value() as f32));
                                }
                            }
                        },
                    },
                },
            },
            add_suffix: batt_info = &gtk::Box {
                #[track = "self.changed(Self::battery_status())"]
                set_visible: self.battery_status.is_some(),
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 6,
                gtk::Image {
                    set_icon_name: self.battery_status
                        .as_ref()
                        .map(|bs| bs.icon()).as_deref(),
                },
                gtk::Label {
                    #[track = "self.changed(Self::battery_status())"]
                    set_text: &self.battery_status
                        .as_ref()
                        .map(|bs| bs.to_string())
                        .unwrap_or_default(),
                },
            },
            set_title: self.title.as_str(),
            set_subtitle: self.subtitle.as_str(),
        }
    }

    async fn update(&mut self, message: Self::Input, _sender: AsyncFactorySender<Self>) -> () {
        self.reset();

        match message {
            Self::Input::Update => {
                if let Some(dev) = self.device.as_mut() {
                    dev.update();
                }
                let mut data: Option<(Option<EnvisionBatteryStatus>, Option<f32>)> = None;
                if let Some(dev) = self.device.as_ref() {
                    data = Some((dev.battery.map(EnvisionBatteryStatus::from), dev.brightness));
                }
                if let Some((batt, brightness)) = data {
                    self.set_battery_status(batt);
                    self.set_brightness(brightness);
                }
            }
            Self::Input::BrightnessChanged(v) => {
                if self.device.is_none() {
                    return;
                }
                if let Some(b) = self.brightness
                    && b == v
                {
                    return;
                }
                self.set_brightness(Some(v));
                if let Some(dev) = self.device.as_ref()
                    && let Err(e) = dev.set_brightness(v)
                {
                    error!("Failed to set device brightness: {e:?}");
                }
            }
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: AsyncFactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();

        if let Some(cls) = self.state.class_name() {
            widgets.root.add_css_class(cls);
            widgets.icon.add_css_class(cls);
        }

        widgets
    }

    async fn init_model(
        init: Self::Init,
        _index: &DynamicIndex,
        _sender: AsyncFactorySender<Self>,
    ) -> Self {
        Self {
            title: init.title.unwrap_or_default(),
            subtitle: init.subtitle.unwrap_or_default(),
            state: init.state.unwrap_or_default(),
            battery_status: init.battery_status,
            brightness: init.brightness,
            device: init.device,
            tracker: 0,
        }
    }
}
