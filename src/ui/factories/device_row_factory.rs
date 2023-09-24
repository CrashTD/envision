use adw::prelude::*;
use relm4::{factory::AsyncFactoryComponent, prelude::*, AsyncFactorySender};

use crate::{
    ui::devices_box::DevicesBoxMsg,
    xr_devices::{XRDevice, XRDeviceType},
};

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
}

#[derive(Debug, Default)]
pub struct DeviceRowModelInit {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub state: Option<DeviceRowState>,
    pub suffix: Option<gtk::Widget>,
}

impl DeviceRowModelInit {
    pub fn from_xr_device(d: &XRDevice) -> Self {
        Self {
            title: Some(d.dev_type.to_string()),
            subtitle: Some(d.name.clone()),
            ..Default::default()
        }
    }

    pub fn new_missing(t: XRDeviceType) -> Self {
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
    type ParentInput = DevicesBoxMsg;
    type ParentWidget = gtk::ListBox;

    view! {
        root = adw::ActionRow {
            // TODO: replace with flat button that spawns popover
            add_prefix: icon = &gtk::Image {
                set_icon_name: Some(self.state.icon()),
            },
            set_title: self.title.as_str(),
            set_subtitle: self.subtitle.as_str(),
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: &Self::Root,
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
            suffix: init.suffix,
        }
    }
}
