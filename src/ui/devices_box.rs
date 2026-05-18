use super::factories::device_row_factory::{DeviceRowModel, DeviceRowModelInit, DeviceRowState};
use crate::{
    ui::factories::device_row_factory::DeviceRowModelMsg,
    xr_devices::{XRDevice, XRDeviceRole},
};
use adw::prelude::*;
use relm4::{factory::AsyncFactoryVecDeque, prelude::*};

#[tracker::track]
pub struct DevicesBox {
    has_devices: bool,

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
            #[track = "model.changed(Self::has_devices())"]
            set_visible: model.has_devices,
            append: &devices_listbox,
        }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        self.reset();

        match message {
            Self::Input::UpdateDevices(devs) => {
                self.set_has_devices(!devs.is_empty());
                let mut guard = self.device_rows.guard();
                let existing_devs: Vec<XRDevice> = guard
                    .iter()
                    .filter_map(|optrow| optrow.and_then(|row| row.device.clone()))
                    .collect();
                let mut new_devs = devs.clone();
                new_devs.retain(|d| !existing_devs.iter().any(|ed| ed == d));
                // remove stale rows
                {
                    let mut indexes_to_rm: Vec<usize> = Vec::new();
                    for (i, row) in guard.iter().enumerate().rev() {
                        if let Some(row) = row
                            && let Some(dev) = row.device.as_ref()
                            && devs.iter().any(|d| d == dev)
                        {
                            // row can be retained
                        } else {
                            indexes_to_rm.push(i);
                        }
                    }
                    for i in indexes_to_rm {
                        guard.remove(i);
                    }
                }
                if !self.has_devices {
                    return;
                }
                guard.broadcast(DeviceRowModelMsg::Update);
                let mut has_head = existing_devs
                    .iter()
                    .any(|d| d.roles.contains(&XRDeviceRole::Head));
                let mut has_left = existing_devs
                    .iter()
                    .any(|d| d.roles.contains(&XRDeviceRole::Left));
                let mut has_right = existing_devs
                    .iter()
                    .any(|d| d.roles.contains(&XRDeviceRole::Right));
                let mut models: Vec<DeviceRowModelInit> = Vec::new();
                for dev in new_devs {
                    let mut row_model = DeviceRowModelInit::from(&dev);
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
                if !has_head {
                    models.push(DeviceRowModelInit::new_missing(XRDeviceRole::Head));
                }
                if !has_right {
                    models.push(DeviceRowModelInit::new_missing(XRDeviceRole::Right));
                }
                if !has_left {
                    models.push(DeviceRowModelInit::new_missing(XRDeviceRole::Left));
                }

                models.sort_by_key(|m1| m1.sort_index);

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
            has_devices: false,
            device_rows: AsyncFactoryVecDeque::builder()
                .launch(devices_listbox.clone())
                .detach(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
