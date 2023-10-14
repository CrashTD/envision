use crate::{
    adb::get_adb_install_runner,
    cmd_runner::CmdRunner,
    depcheck::check_dependency,
    dependencies::adb_dep::adb_dep,
    downloader::download_file,
    paths::wivrn_apk_download_path,
    profile::{Profile, XRServiceType},
    runner::{Runner, RunnerStatus},
};
use gtk::prelude::*;
use relm4::{
    actions::{ActionGroupName, RelmAction, RelmActionGroup},
    new_action_group, new_stateless_action,
    prelude::*,
};
use std::thread::JoinHandle;

use super::alert::alert;

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
    install_runner: Option<CmdRunner>,
    #[tracker::do_not_track]
    root_win: gtk::Window,
}

#[derive(Debug)]
pub enum InstallWivrnBoxMsg {
    ClockTicking,
    UpdateSelectedProfile(Profile),
    DownloadWivrn(String),
    InstallWivrnApk,
}

#[derive(Debug)]
pub struct InstallWivrnBoxInit {
    pub selected_profile: Profile,
    pub root_win: gtk::Window,
}

#[relm4::component(pub)]
impl SimpleComponent for InstallWivrnBox {
    type Init = InstallWivrnBoxInit;
    type Input = InstallWivrnBoxMsg;
    type Output = ();

    menu! {
        install_wivrn_menu: {
            section! {
                "_Standard (Oculus and other devices)" => WivrnApkStandardAction,
                "_Pico" => WivrnApkPicoAction,
            }
        }
    }

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
            gtk::MenuButton {
                add_css_class: "suggested-action",
                set_label: "Install WiVRn",
                set_halign: gtk::Align::Start,
                #[track = "model.changed(Self::install_wivrn_status())"]
                set_sensitive: model.install_wivrn_status != InstallWivrnStatus::InProgress,
                set_menu_model: Some(&install_wivrn_menu),
            },
            gtk::Label {
                add_css_class: "error",
                set_xalign: 0.0,
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
            Self::Input::DownloadWivrn(link) => {
                if !check_dependency(adb_dep()) {
                    alert("ADB is not installed", Some("Please install ADB on your computer to install WiVRn on your Android headset"), Some(&self.root_win));
                } else {
                    self.set_install_wivrn_status(InstallWivrnStatus::InProgress);
                    self.download_thread = Some(download_file(link, wivrn_apk_download_path()));
                }
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
            root_win: init.root_win,
            tracker: 0,
        };

        let widgets = view_output!();

        let mut actions = RelmActionGroup::<InstallWivrnActionGroup>::new();

        let apk_oculus_action = {
            let oculus_sender = sender.clone();
            RelmAction::<WivrnApkStandardAction>::new_stateless(move |_| {
                oculus_sender.input(Self::Input::DownloadWivrn(
                    "https://github.com/Meumeu/WiVRn/releases/latest/download/WiVRn-standard-release.apk".into()
                ));
            })
        };

        let apk_pico_action = {
            let pico_sender = sender.clone();
            RelmAction::<WivrnApkPicoAction>::new_stateless(move |_| {
                pico_sender.input(Self::Input::DownloadWivrn(
                    "https://github.com/Meumeu/WiVRn/releases/latest/download/WiVRn-pico-release.apk".into()
                ));
            })
        };

        actions.add_action(apk_oculus_action);
        actions.add_action(apk_pico_action);

        root.insert_action_group(
            InstallWivrnActionGroup::NAME,
            Some(&actions.into_action_group()),
        );

        ComponentParts { model, widgets }
    }
}

new_action_group!(pub InstallWivrnActionGroup, "installwivrn");
new_stateless_action!(pub WivrnApkStandardAction, InstallWivrnActionGroup, "apkoculus");
new_stateless_action!(pub WivrnApkPicoAction, InstallWivrnActionGroup, "apkpico");
