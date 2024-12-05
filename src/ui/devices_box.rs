use super::factories::device_row_factory::{DeviceRowModel, DeviceRowModelInit, DeviceRowState};
use crate::xr_devices::{XRDevice, XRDeviceRole};
use adw::prelude::*;
use relm4::{factory::AsyncFactoryVecDeque, prelude::*};

#[tracker::track]
pub struct DevicesBox {
    #[no_eq]
    devices: Vec<XRDevice>,

    #[tracker::do_not_track]
    device_rows: AsyncFactoryVecDeque<DeviceRowModel>,
}

#[derive(Debug)]
pub enum DevicesBoxMsg {
    UpdateDevices(Vec<XRDevice>),
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
            #[track = "model.changed(Self::devices())"]
            set_visible: !model.devices.is_empty(),
            append: &devices_listbox,
        }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        self.reset();

        match message {
            Self::Input::UpdateDevices(devs) => {
                self.set_devices(devs);
                let mut guard = self.device_rows.guard();
                guard.clear();
                if self.devices.is_empty() {
                    return;
                }
                let mut has_head = false;
                let mut has_left = false;
                let mut has_right = false;
                let mut models: Vec<DeviceRowModelInit> = Vec::new();
                for dev in &self.devices {
                    let mut row_model = DeviceRowModelInit::from(dev);
                    if !has_head && dev.roles.contains(&XRDeviceRole::Head) {
                        has_head = true;
                        if ["Simulated HMD", "Qwerty HMD"].contains(&dev.name.as_str()) {
                            row_model.state = Some(DeviceRowState::Warning);
                            row_model.subtitle = Some(format!("No HMD detected ({})", dev.name));
                        }
                    }
                    if !has_left && dev.roles.contains(&XRDeviceRole::Left) {
                        has_left = true;
                        if ["Qwerty Left Controller"].contains(&dev.name.as_str()) {
                            row_model.state = Some(DeviceRowState::Warning);
                            row_model.subtitle =
                                Some(format!("No left controller detected ({})", dev.name));
                        }
                    }
                    if !has_right && dev.roles.contains(&XRDeviceRole::Right) {
                        has_right = true;
                        if ["Qwerty Right Controller"].contains(&dev.name.as_str()) {
                            row_model.state = Some(DeviceRowState::Warning);
                            row_model.subtitle =
                                Some(format!("No right controller detected ({})", dev.name));
                        }
                    }
                    models.push(row_model);
                }
                if !has_right {
                    models.push(DeviceRowModelInit::new_missing(XRDeviceRole::Right));
                }
                if !has_left {
                    models.push(DeviceRowModelInit::new_missing(XRDeviceRole::Left));
                }
                if !has_head {
                    models.push(DeviceRowModelInit::new_missing(XRDeviceRole::Head));
                }

                models.sort_by(|m1, m2| m1.sort_index.cmp(&m2.sort_index));

                for model in models {
                    guard.push_back(model);
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let devices_listbox = gtk::ListBox::builder()
            .css_classes(["boxed-list"])
            .selection_mode(gtk::SelectionMode::None)
            .build();

        let model = Self {
            tracker: 0,
            devices: vec![],
            device_rows: AsyncFactoryVecDeque::builder()
                .launch(devices_listbox.clone())
                .detach(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
