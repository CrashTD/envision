use crate::{constants::APP_NAME, xdg::XDG};
use tracing::{debug, error};
use relm4::{
    gtk::{self, prelude::*},
    ComponentParts, ComponentSender, SimpleComponent,
};

#[tracker::track]
pub struct OpenHmdCalibrationBox {
    visible: bool,
    xrservice_active: bool,
}

#[derive(Debug)]
pub enum OpenHmdCalibrationBoxMsg {
    SetVisible(bool),
    XRServiceActiveChanged(bool),
}

#[relm4::component(pub)]
impl SimpleComponent for OpenHmdCalibrationBox {
    type Init = ();
    type Input = OpenHmdCalibrationBoxMsg;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            set_vexpand: false,
            set_spacing: 12,
            add_css_class: "card",
            add_css_class: "padded",
            #[track = "model.changed(Self::visible())"]
            set_visible: model.visible,
            gtk::Label {
                add_css_class: "heading",
                set_hexpand: true,
                set_xalign: 0.0,
                set_label: "Rift CV1 Calibration",
                set_wrap: true,
                set_wrap_mode: gtk::pango::WrapMode::Word,
            },
            gtk::Label {
                add_css_class: "dim-label",
                set_hexpand: true,
                set_label: &format!(
                    "On the first run, start {APP_NAME} with your headset in full clear view of both camera bases for a functioning and effective calibration.\n\nYou can reset the calibration with the button below.",
                ),
                set_xalign: 0.0,
                set_wrap: true,
                set_wrap_mode: gtk::pango::WrapMode::Word,
            },
            gtk::Button {
                set_label: "Reset Calibration",
                set_halign: gtk::Align::Start,
                #[track = "model.changed(Self::xrservice_active())"]
                set_sensitive: !model.xrservice_active,
                connect_clicked => move |_| {
                    let target = XDG.get_config_home().join("openhmd/rift-room-config.json");
                    if target.is_file() {
                        if let Err(e) = std::fs::remove_file(target) {
                            error!("Failed to remove openhmd config: {e}");
                        }
                    } else {
                        debug!("trying to delete openhmd calibration config, but file is missing")
                    }
                }
            },
        },
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        self.reset();

        match message {
            Self::Input::SetVisible(state) => {
                self.set_visible(state);
            }
            Self::Input::XRServiceActiveChanged(active) => {
                self.set_xrservice_active(active);
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            tracker: 0,
            visible: false,
            xrservice_active: false,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
