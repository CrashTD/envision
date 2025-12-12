use crate::{
    config::Config,
    profile::{OvrCompatibilityModuleType, XRServiceType},
    ui::{
        SENDER_IO_ERR_MSG,
        preference_rows::{combo_row, spin_row, switch_row},
    },
};
use adw::prelude::*;
use gtk::glib::clone;
use relm4::prelude::*;
use std::{cell::RefCell, rc::Rc};

#[tracker::track]
pub struct PreferencesWindow {
    config: Rc<RefCell<Config>>,
    #[tracker::do_not_track]
    win: Option<adw::PreferencesDialog>,
    #[tracker::do_not_track]
    parent: gtk::Window,
}

#[derive(Debug)]
pub enum PreferencesWindowMsg {
    Present,
    OnClosed,
}

#[derive(Debug)]
pub enum PreferencesWindowOutMsg {
    OnClosed(Config),
}

pub struct PreferencesWindowInit {
    pub root_win: gtk::Window,
    pub config: Config,
}

#[relm4::component(pub async)]
impl AsyncComponent for PreferencesWindow {
    type Init = PreferencesWindowInit;
    type Input = PreferencesWindowMsg;
    type Output = PreferencesWindowOutMsg;
    type CommandOutput = ();

    view! {
        #[name(win)]
        adw::PreferencesDialog {
            set_search_enabled: true,
            connect_closed[sender] => move |_| {
                sender.input(Self::Input::OnClosed);
            },
            add: page = &adw::PreferencesPage {
                add: grp = &adw::PreferencesGroup {
                    add: &combo_row(
                        "XR Service",
                        Some("Choose Monado for PCVR and WiVRn for standalone"),
                        model.config.borrow().global_preferences.xrservice_type.to_string().as_str(),
                        XRServiceType::iter().map(XRServiceType::to_string).collect(),
                        clone!(#[strong(rename_to=config)] model.config, move |row| {
                            config.borrow_mut().global_preferences.xrservice_type = XRServiceType::from(row.selected());
                        })
                    ),
                    add: &combo_row(
                        "OpenVR Compatibility",
                        None,
                        model.config.borrow().global_preferences.ovr_comp.to_string().as_str(),
                        OvrCompatibilityModuleType::iter().map(OvrCompatibilityModuleType::to_string).collect(),
                        clone!(#[strong(rename_to=config)] model.config, move |row| {
                            config.borrow_mut().global_preferences.ovr_comp = OvrCompatibilityModuleType::from(row.selected());
                        })
                    ),
                    add: &spin_row(
                        "Render scale percentage",
                        None,
                        model.config.borrow().global_preferences.scale_percentage.into(),
                        20.0,
                        500.0,
                        1.0,
                        clone!(#[strong(rename_to=config)] model.config, move |adj| {
                            let raw = adj.value().trunc();
                            // boring f64 re-clamping for sake of correctness
                            let val = if raw >= 500.0 {
                                500
                            } else if raw <= 20.0 {
                                20
                            } else {
                                raw as u16
                            };
                            config.borrow_mut().global_preferences.scale_percentage = val;
                        })
                    ),
                    add: &switch_row(
                        "Compositor GPU acceleration",
                        None,
                        model.config.borrow().global_preferences.use_compositor_gpu_compute,
                        clone!(#[strong(rename_to=config)] model.config, move |_, v| {
                            config.borrow_mut().global_preferences.use_compositor_gpu_compute = v;
                        })
                    ),
                    add: &switch_row(
                        "Use minimum frame period",
                        Some("Disables the refresh rate limit; this can increase performance"),
                        model.config.borrow().global_preferences.use_min_frame_period,
                        clone!(#[strong(rename_to=config)] model.config, move |_, v| {
                            config.borrow_mut().global_preferences.use_min_frame_period = v;
                        })
                    ),
                    add: &switch_row(
                        "Monado UI",
                        None,
                        model.config.borrow().global_preferences.monado_ui,
                        clone!(#[strong(rename_to=config)] model.config, move |_, v| {
                            config.borrow_mut().global_preferences.monado_ui = v;
                        })
                    ),
                },
            },
        }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        self.reset();

        match message {
            Self::Input::Present => {
                if let Some(win) = self.win.as_ref() {
                    win.present(Some(&self.parent));
                }
            }
            Self::Input::OnClosed => {
                sender
                    .output(Self::Output::OnClosed(self.config.borrow().clone()))
                    .expect(SENDER_IO_ERR_MSG);
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let mut model = Self {
            win: None,
            config: Rc::new(RefCell::new(init.config)),
            parent: init.root_win,
            tracker: 0,
        };

        let widgets = view_output!();

        model.win = Some(widgets.win.clone());

        AsyncComponentParts { model, widgets }
    }
}
