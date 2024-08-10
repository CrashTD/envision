use crate::{
    ui::battery_status::EnvisionBatteryStatus,
    xr_devices::{XRDevice, XRDeviceRole},
};
use adw::prelude::*;
use relm4::{factory::AsyncFactoryComponent, prelude::*, AsyncFactorySender};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceRowState {
    Ok,
    Error,
    Warning,
}

impl Default for DeviceRowState {
    fn default() -> Self {
        Self::Ok
    }
}

impl DeviceRowState {
    pub fn icon(&self) -> &str {
        match &self {
            Self::Ok => "emblem-ok-symbolic",
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

#[derive(Debug)]
pub struct DeviceRowModel {
    title: String,
    subtitle: String,
    state: DeviceRowState,
    suffix: Option<gtk::Widget>,
    battery_status: Option<EnvisionBatteryStatus>,
}

#[derive(Debug, Default)]
pub struct DeviceRowModelInit {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub state: Option<DeviceRowState>,
    pub suffix: Option<gtk::Widget>,
    pub battery_status: Option<EnvisionBatteryStatus>,
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
            sort_index: d
                .roles
                .iter()
                .min()
                .unwrap_or(&XRDeviceRole::GenericTracker)
                .as_number(),
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

#[relm4::factory(async pub)]
impl AsyncFactoryComponent for DeviceRowModel {
    type Init = DeviceRowModelInit;
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        root = adw::ActionRow {
            add_prefix: icon = &gtk::Image {
                set_icon_name: Some(self.state.icon()),
            },
            add_suffix: batt_info = &gtk::Box {
                set_visible: self.battery_status.is_some(),
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 6,
                gtk::Image {
                    set_icon_name: self.battery_status
                        .as_ref()
                        .map(|bs| bs.icon()).as_deref(),
                },
                gtk::Label {
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

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        _sender: AsyncFactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();

        if let Some(suffix) = self.suffix.as_ref() {
            widgets.root.add_suffix(suffix);
        }
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
            suffix: init.suffix,
        }
    }
}
