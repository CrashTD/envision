use super::{
    alert::alert,
    factories::device_row_factory::{DeviceRowModel, DeviceRowModelInit, DeviceRowState},
};
use crate::{
    file_builders::monado_config_v0::dump_generic_trackers,
    xr_devices::{XRDevice, XRDeviceType},
};
use adw::prelude::*;
use relm4::{factory::FactoryVecDeque, prelude::*, Sender};

#[tracker::track]
pub struct DevicesBox {
    devices: Vec<XRDevice>,

    #[tracker::do_not_track]
    device_rows: FactoryVecDeque<DeviceRowModel>,
}

#[derive(Debug)]
pub enum DevicesBoxMsg {
    UpdateDevices(Vec<XRDevice>),
    DumpGenericTrackers,
}

impl DevicesBox {
    fn create_save_trackers_btn(sender: Sender<DevicesBoxMsg>) -> gtk::Button {
        let btn = gtk::Button::builder()
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .icon_name("document-save-symbolic")
            .tooltip_text("Save current trackers")
            .css_classes(["circular", "flat"])
            .build();

        btn.connect_clicked(move |_| {
            sender.emit(DevicesBoxMsg::DumpGenericTrackers);
        });

        btn
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
            set_visible: !model.devices.is_empty(),

            append: &devices_listbox,
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
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
                            XRDeviceType::Head => {
                                has_head = true;
                                let mut init = DeviceRowModelInit::from_xr_device(&dev);
                                if dev.name == "Simulated HMD" {
                                    init.state = Some(DeviceRowState::Warning);
                                    init.subtitle = Some("No HMD detected (Simulated HMD)".into());
                                }
                                models.push(init);
                            }
                            XRDeviceType::Left => {
                                has_left = true;
                                models.push(DeviceRowModelInit::from_xr_device(&dev));
                            }
                            XRDeviceType::Right => {
                                has_right = true;
                                models.push(DeviceRowModelInit::from_xr_device(&dev));
                            }
                            XRDeviceType::GenericTracker => {
                                generic.push(dev);
                            }
                            _ => {
                                models.push(DeviceRowModelInit::from_xr_device(&dev));
                            }
                        };
                    }
                    if !generic.is_empty() {
                        models.push(DeviceRowModelInit {
                            title: Some(XRDeviceType::GenericTracker.to_string()),
                            subtitle: Some(
                                generic
                                    .iter()
                                    .map(|d| d.id.as_str())
                                    .collect::<Vec<&str>>()
                                    .join(", "),
                            ),
                            suffix: Some(
                                Self::create_save_trackers_btn(sender.input_sender().clone())
                                    .upcast::<gtk::Widget>(),
                            ),
                            ..Default::default()
                        });
                    }
                    if !has_right {
                        models.push(DeviceRowModelInit::new_missing(XRDeviceType::Right));
                    }
                    if !has_left {
                        models.push(DeviceRowModelInit::new_missing(XRDeviceType::Left));
                    }
                    if !has_head {
                        models.push(DeviceRowModelInit::new_missing(XRDeviceType::Head));
                    }

                    models.sort_by(|m1, m2| {
                        let dt1 = XRDeviceType::from_display_str(
                            m1.title.as_ref().unwrap_or(&String::new()),
                        );
                        let dt2 = XRDeviceType::from_display_str(
                            m2.title.as_ref().unwrap_or(&String::new()),
                        );
                        dt1.cmp(&dt2)
                    });

                    for model in models {
                        guard.push_back(model);
                    }
                }
            }
            Self::Input::DumpGenericTrackers => {
                let added = dump_generic_trackers(
                    &self
                        .devices
                        .iter()
                        .filter(|d| d.dev_type == XRDeviceType::GenericTracker)
                        .map(|d| d.id.clone())
                        .collect::<Vec<String>>(),
                );
                let multi_title = format!("Added {} new trackers", added);
                let (title, msg) = match added {
                    0 => (
                        "No new trackers found",
                        concat!(
                            "All the currently connected trackers ",
                            "are already present in your configuration"
                        ),
                    ),
                    1 => (
                        "Added 1 new tracker",
                        concat!(
                            "Edit your configuration to make sure that ",
                            "all the trackers have the appropriate roles assigned"
                        ),
                    ),
                    _ => (
                        multi_title.as_str(),
                        concat!(
                            "Edit your configuration to make sure that ",
                            "all the trackers have the appropriate roles assigned"
                        ),
                    ),
                };
                alert(title, Some(msg), None);
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let devices_listbox = gtk::ListBox::builder()
            .css_classes(["boxed-list"])
            .selection_mode(gtk::SelectionMode::None)
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(12)
            .build();

        let model = Self {
            tracker: 0,
            devices: vec![],
            device_rows: FactoryVecDeque::new(devices_listbox.clone(), sender.input_sender()),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
