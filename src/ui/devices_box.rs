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
                if !self.devices.is_empty() {
                    let mut has_head = false;
                    let mut has_left = false;
                    let mut has_right = false;
                    let mut models: Vec<DeviceRowModelInit> = vec![];
                    let mut generic: Vec<&XRDevice> = vec![];
                    for dev in &self.devices {
                        match dev.dev_type {
                            XRDeviceRole::Head => {
                                has_head = true;
                                let mut init = DeviceRowModelInit::from_xr_device(dev);
                                if dev.name == "Simulated HMD" {
                                    init.state = Some(DeviceRowState::Warning);
                                    init.subtitle = Some("No HMD detected (Simulated HMD)".into());
                                }
                                models.push(init);
                            }
                            XRDeviceRole::Left => {
                                has_left = true;
                                models.push(DeviceRowModelInit::from_xr_device(dev));
                            }
                            XRDeviceRole::Right => {
                                has_right = true;
                                models.push(DeviceRowModelInit::from_xr_device(dev));
                            }
                            XRDeviceRole::GenericTracker => {
                                generic.push(dev);
                            }
                            _ => {
                                models.push(DeviceRowModelInit::from_xr_device(dev));
                            }
                        };
                    }
                    if !generic.is_empty() {
                        models.push(DeviceRowModelInit {
                            title: Some(XRDeviceRole::GenericTracker.to_string()),
                            subtitle: Some(
                                generic
                                    .iter()
                                    .map(|d| format!("{} ({})", d.name, d.index))
                                    .collect::<Vec<String>>()
                                    .join("\n"),
                            ),
                            ..Default::default()
                        });
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

                    models.sort_by(|m1, m2| {
                        let dt1 = XRDeviceRole::from_display_str(
                            m1.title.as_ref().unwrap_or(&String::new()),
                        );
                        let dt2 = XRDeviceRole::from_display_str(
                            m2.title.as_ref().unwrap_or(&String::new()),
                        );
                        dt1.cmp(&dt2)
                    });

                    for model in models {
                        guard.push_back(model);
                    }
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
