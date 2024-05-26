use super::job_worker::{
    internal_worker::JobWorkerOut,
    job::{FuncWorkerOut, WorkerJob},
    JobWorker,
};
use crate::paths::get_steamvr_bin_dir_path;
use relm4::{
    binding::{Binding, ConnectBinding, StringBinding},
    gtk::{self, prelude::*},
    ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent,
};
use std::{
    collections::{HashMap, VecDeque},
    path::Path,
    thread::sleep,
    time::Duration,
};

#[tracker::track]
pub struct SteamVrCalibrationBox {
    calibration_running: bool,
    calibration_result: Option<String>,
    calibration_success: bool,
    visible: bool,
    xrservice_active: bool,
    #[tracker::do_not_track]
    server_worker: Option<JobWorker>,
    #[tracker::do_not_track]
    calibration_worker: Option<JobWorker>,
}

#[derive(Debug)]
pub enum SteamVrCalibrationBoxMsg {
    SetVisible(bool),
    RunCalibration,
    OnServerWorkerExit(i32),
    OnCalWorkerExit(i32),
    XRServiceActiveChanged(bool),
    NoOp,
}

#[relm4::component(pub)]
impl SimpleComponent for SteamVrCalibrationBox {
    type Init = ();
    type Input = SteamVrCalibrationBoxMsg;
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
                set_label: "SteamVR Quick Calibration",
                set_wrap: true,
                set_wrap_mode: gtk::pango::WrapMode::Word,
            },
            gtk::Button {
                add_css_class: "flat",
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,
                    set_vexpand: false,
                    set_spacing: 6,
                    set_halign: gtk::Align::Start,
                    #[name(revealer_icon)]
                    gtk::Image {
                        set_icon_name: Some("pan-end-symbolic"),
                    },
                    gtk::Label {
                        set_label: "Details"
                    }
                },
                connect_clicked[revealer] => move |_| {
                    revealer.set_reveal_child(!revealer.reveals_child())
                },
            },
            #[name(revealer)]
            gtk::Revealer {
                gtk::Label {
                    add_css_class: "dim-label",
                    set_hexpand: true,
                    set_label: concat!(
                        "Run a quick SteamVR calibration.\n\n",
                        " \u{2022} Plug in your HMD and place it on the floor, ",
                        "approximately in the middle of your play area\n",
                        " \u{2022} Make sure your controllers and other VR devices ",
                        "are turned off\n",
                        " \u{2022} Click the Calibrate button and wait for the ",
                        "process to finish\n\n",
                        "Note that the orientation of your HMD during this process ",
                        "will dictate the forward direction of your play area.",
                    ),
                    set_xalign: 0.0,
                    set_wrap: true,
                    set_wrap_mode: gtk::pango::WrapMode::Word,
                },
            },
            gtk::Label {
                add_css_class: "error",
                add_css_class: "success",
                set_hexpand: true,
                #[track = "model.changed(Self::calibration_result())"]
                set_visible: model.calibration_result.is_some(),
                #[track = "model.changed(Self::calibration_result())"]
                set_label: model.calibration_result.as_ref().unwrap_or(&String::new()),
                #[track = "model.changed(Self::calibration_success())"]
                set_class_active: ("error", !model.calibration_success),
                #[track = "model.changed(Self::calibration_success())"]
                set_class_active: ("success", model.calibration_success),
                set_xalign: 0.0,
                set_wrap: true,
                set_wrap_mode: gtk::pango::WrapMode::Word,
            },
            gtk::Button {
                add_css_class: "suggested-action",
                set_label: "Calibrate",
                set_halign: gtk::Align::Start,
                #[track = "model.changed(Self::calibration_running()) || model.changed(Self::xrservice_active())"]
                set_sensitive: !model.xrservice_active && !model.calibration_running,
                connect_clicked[sender] => move |_| {
                    sender.input(Self::Input::RunCalibration);
                }
            },
        },
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        self.reset();

        match message {
            Self::Input::SetVisible(state) => {
                self.set_visible(state);
            }
            Self::Input::XRServiceActiveChanged(active) => {
                self.set_xrservice_active(active);
            }
            Self::Input::RunCalibration => {
                self.set_calibration_result(None);
                let steamvr_bin_dir = get_steamvr_bin_dir_path();
                if !Path::new(&steamvr_bin_dir).is_dir() {
                    self.set_calibration_success(false);
                    self.set_calibration_result(Some("SteamVR not found".into()));
                    return;
                }
                let mut env: HashMap<String, String> = HashMap::new();
                env.insert("LD_LIBRARY_PATH".into(), steamvr_bin_dir.clone());
                let vrcmd = format!("{steamvr_bin_dir}/vrcmd");
                let mut server_worker = {
                    let mut jobs: VecDeque<WorkerJob> = VecDeque::new();
                    jobs.push_back(WorkerJob::new_cmd(
                        Some(env.clone()),
                        vrcmd.clone(),
                        Some(vec!["--pollposes".into()]),
                    ));
                    JobWorker::new(jobs, sender.input_sender(), |msg| match msg {
                        JobWorkerOut::Log(_) => Self::Input::NoOp,
                        JobWorkerOut::Exit(code) => Self::Input::OnServerWorkerExit(code),
                    })
                };
                let mut cal_worker = {
                    let mut jobs: VecDeque<WorkerJob> = VecDeque::new();
                    jobs.push_back(WorkerJob::new_func(Box::new(move || {
                        sleep(Duration::from_secs(2));
                        FuncWorkerOut {
                            success: true,
                            out: vec![],
                        }
                    })));
                    jobs.push_back(WorkerJob::new_cmd(
                        Some(env),
                        vrcmd,
                        Some(vec!["--resetroomsetup".into()]),
                    ));
                    JobWorker::new(jobs, sender.input_sender(), |msg| match msg {
                        JobWorkerOut::Log(_) => Self::Input::NoOp,
                        JobWorkerOut::Exit(code) => Self::Input::OnCalWorkerExit(code),
                    })
                };

                server_worker.start();
                cal_worker.start();
                self.server_worker = Some(server_worker);
                self.calibration_worker = Some(cal_worker);
            }
            Self::Input::OnServerWorkerExit(_) => {
                self.calibration_running = false;
            }
            Self::Input::OnCalWorkerExit(code) => {
                self.calibration_success = code == 0;
                self.calibration_result = if code == 0 {
                    Some("Calibration completed".into())
                } else {
                    Some(format!("Calibration failed with code {code}"))
                };
                if let Some(sw) = self.server_worker.as_ref() {
                    sw.stop();
                }
            }
            Self::Input::NoOp => {}
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            tracker: 0,
            visible: false,
            calibration_result: None,
            calibration_running: false,
            calibration_success: false,
            xrservice_active: false,
            server_worker: None,
            calibration_worker: None,
        };

        let widgets = view_output!();

        let _ = &widgets
            .revealer
            .bind_property("reveal-child", &widgets.revealer_icon, "icon-name")
            .transform_to(|_, reveal_child| match reveal_child {
                true => Some("pan-down-symbolic"),
                false => Some("pan-end-symbolic"),
            })
            .sync_create()
            .build();

        ComponentParts { model, widgets }
    }
}
