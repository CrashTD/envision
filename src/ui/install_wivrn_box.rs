use crate::{
    adb::get_adb_install_runner,
    downloader::download_file,
    paths::wivrn_apk_download_path,
    profile::{Profile, XRServiceType},
    runner::{Runner, RunnerStatus},
};
use gtk::prelude::*;
use relm4::prelude::*;
use std::thread::JoinHandle;

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
    download_thread: Option<JoinHandle<Result<(), reqwest::Error>>>,
    #[tracker::do_not_track]
    install_runner: Option<Runner>,
}

#[derive(Debug)]
pub enum InstallWivrnBoxMsg {
    ClockTicking,
    UpdateSelectedProfile(Profile),
    DownloadWivrn,
    InstallWivrnApk,
}

#[derive(Debug)]
pub struct InstallWivrnBoxInit {
    pub selected_profile: Profile,
}

#[relm4::component(pub)]
impl SimpleComponent for InstallWivrnBox {
    type Init = InstallWivrnBoxInit;
    type Input = InstallWivrnBoxMsg;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 12,
            set_margin_top: 12,
            set_margin_bottom: 12,
            #[track = "model.changed(Self::selected_profile())"]
            set_visible: match model.selected_profile.xrservice_type {
                XRServiceType::Wivrn => true,
                _ => false,
            },
            gtk::Separator {
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,
            },
            gtk::Label {
                add_css_class: "heading",
                set_hexpand: true,
                set_xalign: 0.0,
                set_margin_start: 12,
                set_margin_end: 12,
                set_label: "Install WiVRn APK",
                set_wrap: true,
                set_wrap_mode: gtk::pango::WrapMode::Word,
            },
            gtk::Label {
                add_css_class: "dim-label",
                set_hexpand: true,
                set_xalign: 0.0,
                set_margin_start: 12,
                set_margin_end: 12,
                set_label: concat!(
                    "Install the WiVRn APK on your standalong Android headset. ",
                    "You will need to enable Developer Mode on your headset, ",
                    "then press the \"Install WiVRn\" button."
                ),
                set_wrap: true,
                set_wrap_mode: gtk::pango::WrapMode::Word,
            },
            gtk::Button {
                add_css_class: "suggested-action",
                set_label: "Install WiVRn",
                set_margin_start: 12,
                set_margin_end: 12,
                set_halign: gtk::Align::Start,
                #[track = "model.changed(Self::install_wivrn_status())"]
                set_sensitive: match model.install_wivrn_status {
                    InstallWivrnStatus::InProgress => false,
                    _ => true,
                },
                connect_clicked[sender] => move |_| {
                    sender.input(Self::Input::DownloadWivrn);
                },
            },
            gtk::Label {
                add_css_class: "error",
                set_margin_start: 12,
                set_margin_end: 12,
                set_xalign: 0.0,
                #[track = "model.changed(Self::install_wivrn_status())"]
                set_visible: match &model.install_wivrn_status {
                    InstallWivrnStatus::Done(Some(_)) => true,
                    _ => false,
                },
                #[track = "model.changed(Self::install_wivrn_status())"]
                set_label: match &model.install_wivrn_status {
                    InstallWivrnStatus::Done(Some(err)) => err.as_str(),
                    _ => "",
                },
            },
            gtk::Label {
                add_css_class: "success",
                set_margin_start: 12,
                set_margin_end: 12,
                set_xalign: 0.0,
                #[track = "model.changed(Self::install_wivrn_status())"]
                set_visible: match &model.install_wivrn_status {
                    InstallWivrnStatus::Success => true,
                    _ => false,
                },
                set_label: "WiVRn Installed Successfully",
            },
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        self.reset();

        match message {
            Self::Input::ClockTicking => {
                if self.download_thread.is_some() {
                    let finished = self.download_thread.as_ref().unwrap().is_finished();
                    if finished {
                        let joinh = self.download_thread.take().unwrap();
                        match joinh.join().unwrap() {
                            Ok(_) => {
                                sender.input(Self::Input::InstallWivrnApk);
                            }
                            Err(_) => self.set_install_wivrn_status(InstallWivrnStatus::Done(
                                Some("Error downloading WiVRn APK".into()),
                            )),
                        }
                    }
                }
                if self.install_runner.is_some() {
                    let runner = self.install_runner.as_mut().unwrap();
                    match runner.status() {
                        RunnerStatus::Running => {}
                        RunnerStatus::Stopped(status) => {
                            self.install_runner.take();
                            self.set_install_wivrn_status(match status {
                                None | Some(0) => InstallWivrnStatus::Success,
                                Some(255) => {
                                    InstallWivrnStatus::Done(Some("You need to authorize this computer to run developer commands in your headset. Authorize it and try again.".into()))
                                }
                                Some(err_code) => InstallWivrnStatus::Done(Some(format!(
                                    "ADB exited with code \"{c}\"",
                                    c = err_code
                                ))),
                            });
                        }
                    };
                }
            }
            Self::Input::DownloadWivrn => {
                self.set_install_wivrn_status(InstallWivrnStatus::InProgress);
                self.download_thread = Some(download_file(
                    "https://github.com/Meumeu/WiVRn/releases/latest/download/WiVRn-oculus-release.apk".into(),
                    wivrn_apk_download_path()
                ));
            }
            Self::Input::InstallWivrnApk => {
                let mut runner = get_adb_install_runner(&wivrn_apk_download_path());
                runner.start();
                self.install_runner = Some(runner);
            }
            Self::Input::UpdateSelectedProfile(p) => {
                self.set_selected_profile(p);
            }
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            selected_profile: init.selected_profile,
            install_wivrn_status: InstallWivrnStatus::Done(None),
            download_thread: None,
            install_runner: None,
            tracker: 0,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
