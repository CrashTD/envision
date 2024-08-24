use super::{
    alert::alert,
    job_worker::{
        internal_worker::JobWorkerOut,
        job::{FuncWorkerOut, WorkerJob},
        JobWorker,
    },
};
use crate::{
    dependencies::adb_dep::adb_dep,
    downloader::download_file_sync,
    paths::wivrn_apk_download_path,
    profile::{Profile, XRServiceType},
};
use gtk::{glib::clone, prelude::*};
use relm4::{new_action_group, new_stateless_action, prelude::*};
use std::collections::VecDeque;

const DOWNLOAD_ERROR: &str = "download error";

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum InstallWivrnStatus {
    Success,
    Done(Option<String>),
    InProgress,
}

#[tracker::track]
pub struct InstallWivrnBox {
    selected_profile: Profile,
    install_wivrn_status: InstallWivrnStatus,
    #[tracker::do_not_track]
    install_runner: Option<JobWorker>,
    #[tracker::do_not_track]
    install_runner_log: String,
    #[tracker::do_not_track]
    root_win: gtk::Window,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum InstallWivrnBoxMsg {
    UpdateSelectedProfile(Profile),
    InstallWivrnApk(String),
    OnInstallRunnerLog(Vec<String>),
    OnInstallRunnerExit(i32),
}

#[derive(Debug)]
pub struct InstallWivrnBoxInit {
    pub selected_profile: Profile,
    pub root_win: gtk::Window,
}

#[relm4::component(pub async)]
impl AsyncComponent for InstallWivrnBox {
    type Init = InstallWivrnBoxInit;
    type Input = InstallWivrnBoxMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 12,
            add_css_class: "card",
            add_css_class: "padded",
            #[track = "model.changed(Self::selected_profile())"]
            set_visible: model.selected_profile.xrservice_type == XRServiceType::Wivrn,
            gtk::Label {
                add_css_class: "heading",
                set_hexpand: true,
                set_xalign: 0.0,
                set_label: "Install WiVRn APK",
                set_wrap: true,
                set_wrap_mode: gtk::pango::WrapMode::Word,
            },
            gtk::Label {
                add_css_class: "dim-label",
                set_hexpand: true,
                set_label: concat!(
                    "Install the WiVRn APK on your standalong Android headset. ",
                    "You will need to enable Developer Mode on your headset, ",
                    "then press the \"Install WiVRn\" button."
                ),
                set_xalign: 0.0,
                set_wrap: true,
                set_wrap_mode: gtk::pango::WrapMode::Word,
            },
            gtk::Button {
                add_css_class: "suggested-action",
                set_label: "Install WiVRn",
                set_halign: gtk::Align::Start,
                #[track = "model.changed(Self::install_wivrn_status())"]
                set_sensitive: model.install_wivrn_status != InstallWivrnStatus::InProgress,
                connect_clicked[sender] => move |_| {
                     sender.input(Self::Input::InstallWivrnApk(
                        "https://github.com/WiVRn/WiVRn/releases/latest/download/WiVRn-standard-release.apk".into()
                     ))
                },
            },
            gtk::Label {
                add_css_class: "error",
                set_xalign: 0.0,
                set_wrap: true,
                set_wrap_mode: gtk::pango::WrapMode::Word,
                #[track = "model.changed(Self::install_wivrn_status())"]
                set_visible: matches!(&model.install_wivrn_status, InstallWivrnStatus::Done(Some(_))),
                #[track = "model.changed(Self::install_wivrn_status())"]
                set_label: match &model.install_wivrn_status {
                    InstallWivrnStatus::Done(Some(err)) => err.as_str(),
                    _ => "",
                },
            },
            gtk::Label {
                add_css_class: "success",
                set_xalign: 0.0,
                #[track = "model.changed(Self::install_wivrn_status())"]
                set_visible: model.install_wivrn_status == InstallWivrnStatus::Success,
                set_label: "WiVRn Installed Successfully",
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
            Self::Input::InstallWivrnApk(link) => {
                if !adb_dep().check() {
                    alert("ADB is not installed", Some("Please install ADB on your computer to install WiVRn on your Android headset"), Some(&self.root_win));
                    return;
                }
                self.set_install_wivrn_status(InstallWivrnStatus::InProgress);
                self.install_runner_log = String::default();
                let mut jobs = VecDeque::new();
                let wivrn_apk_path = wivrn_apk_download_path();
                jobs.push_back(WorkerJob::new_func(Box::new(clone!(
                    #[strong]
                    wivrn_apk_path,
                    move || {
                        if download_file_sync(&link, &wivrn_apk_path).is_err() {
                            FuncWorkerOut {
                                out: vec![DOWNLOAD_ERROR.into()],
                                success: false,
                            }
                        } else if wivrn_apk_path.is_file() {
                            FuncWorkerOut {
                                out: Vec::default(),
                                success: true,
                            }
                        } else {
                            FuncWorkerOut {
                                out: vec![format!(
                                    "Provided apk path {} does not exist",
                                    wivrn_apk_path.to_string_lossy()
                                )],
                                success: false,
                            }
                        }
                    }
                ))));
                jobs.push_back(WorkerJob::new_cmd(
                    None,
                    "adb".into(),
                    Some(vec![
                        "install".into(),
                        wivrn_apk_path.to_string_lossy().to_string(),
                    ]),
                ));
                let runner = JobWorker::new(
                    jobs,
                    sender.input_sender(),
                    Box::new(move |e| match e {
                        JobWorkerOut::Log(log) => Self::Input::OnInstallRunnerLog(log),
                        JobWorkerOut::Exit(code) => Self::Input::OnInstallRunnerExit(code),
                    }),
                );
                runner.start();
                self.install_runner = Some(runner);
            }
            Self::Input::UpdateSelectedProfile(p) => {
                self.set_selected_profile(p);
            }
            Self::Input::OnInstallRunnerLog(rows) => {
                self.install_runner_log.push('\n');
                self.install_runner_log.push_str(&rows.join("\n"));
            }
            Self::Input::OnInstallRunnerExit(code) => {
                self.set_install_wivrn_status(match code {
                    0 => InstallWivrnStatus::Success,
                    255 => InstallWivrnStatus::Done(Some(
                        concat!(
                            "You need to authorize this computer to run developer ",
                            "commands in your headset. Authorize it and try again.",
                        )
                        .into(),
                    )),
                    1 if self.install_runner_log.contains(DOWNLOAD_ERROR) => {
                        InstallWivrnStatus::Done(Some("Error downloading WiVRn APK".into()))
                    }
                    1 if self
                        .install_runner_log
                        .contains("no devices/emulators found") =>
                    {
                        InstallWivrnStatus::Done(Some(
                            concat!(
                                "No devices connected. Make sure your headset is connected ",
                                "to this computer and USB debugging is turned on."
                            )
                            .into(),
                        ))
                    }
                    _ => {
                        InstallWivrnStatus::Done(Some(format!("ADB exited with code \"{}\"", code)))
                    }
                });
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            selected_profile: init.selected_profile,
            install_wivrn_status: InstallWivrnStatus::Done(None),
            install_runner: None,
            install_runner_log: String::default(),
            root_win: init.root_win,
            tracker: 0,
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }
}

new_action_group!(pub InstallWivrnActionGroup, "installwivrn");
new_stateless_action!(pub WivrnApkStandardAction, InstallWivrnActionGroup, "apkoculus");
new_stateless_action!(pub WivrnApkPicoAction, InstallWivrnActionGroup, "apkpico");
