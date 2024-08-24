use super::alert::alert;
use crate::{
    async_process::async_process,
    dependencies::adb_dep::adb_dep,
    downloader::{cache_file, cache_file_path},
    profile::{Profile, XRServiceType},
};
use gtk::prelude::*;
use relm4::{new_action_group, new_stateless_action, prelude::*};
use std::fs::remove_file;

const ADB_ERR_NO_DEV: &str = "no devices/emulators found";

const WIVRN_LATEST_RELEASE_APK_URL: &str =
    "https://github.com/WiVRn/WiVRn/releases/latest/download/WiVRn-standard-release.apk";

#[derive(Debug)]
enum GetWivrnApkRefErr {
    NotWivrn,
    RepoDirNotFound,
    RepoManipulationFailed(git2::Error),
}

#[derive(Debug, Clone)]
enum WivrnApkRef {
    Tag(String),
    Commit(String),
}

fn get_wivrn_apk_ref(p: &Profile) -> Result<WivrnApkRef, GetWivrnApkRefErr> {
    if p.xrservice_type != XRServiceType::Wivrn {
        Err(GetWivrnApkRefErr::NotWivrn)
    } else if !p.xrservice_path.is_dir() {
        Err(GetWivrnApkRefErr::RepoDirNotFound)
    } else {
        let repo = git2::Repository::open(&p.xrservice_path)
            .map_err(GetWivrnApkRefErr::RepoManipulationFailed)?;
        if let Ok(tag) = repo
            .describe(
                git2::DescribeOptions::new()
                    .describe_tags()
                    .max_candidates_tags(0),
            )
            .and_then(|d| d.format(None))
        {
            Ok(WivrnApkRef::Tag(tag))
        } else {
            Ok(WivrnApkRef::Commit(
                repo.head()
                    .map_err(GetWivrnApkRefErr::RepoManipulationFailed)?
                    .peel_to_commit()
                    .map_err(GetWivrnApkRefErr::RepoManipulationFailed)?
                    .id()
                    .to_string(),
            ))
        }
    }
}

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
    root_win: gtk::Window,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum InstallWivrnBoxMsg {
    UpdateSelectedProfile(Profile),
    /// prepares state for async action, calls DoInstall
    InstallWivrnApk,
    /// dowloads, installs, sets state back to what it needs to be
    DoInstall(String),
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
                     sender.input(Self::Input::InstallWivrnApk)
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
            Self::Input::InstallWivrnApk => {
                if !adb_dep().check() {
                    alert("ADB is not installed", Some("Please install ADB on your computer to install WiVRn on your Android headset"), Some(&self.root_win));
                    return;
                }
                self.set_install_wivrn_status(InstallWivrnStatus::InProgress);

                match get_wivrn_apk_ref(&self.selected_profile) {
                    Err(GetWivrnApkRefErr::NotWivrn) => {
                        eprintln!("This is not a WiVRn profile, how did you get here?");
                    }
                    Err(GetWivrnApkRefErr::RepoDirNotFound) => {
                        self.set_install_wivrn_status(InstallWivrnStatus::Done(Some(
                            "Could not open WiVRn repository. Please build the profile before attempting to install the client APK".into()
                        )));
                    }
                    Err(GetWivrnApkRefErr::RepoManipulationFailed(giterr)) => {
                        eprintln!("Error: failed to manipulate WiVRn repo: {giterr}, falling back to latest release APK");
                        let existing = cache_file_path(WIVRN_LATEST_RELEASE_APK_URL, Some("apk"));
                        if existing.is_file() {
                            if let Err(e) = remove_file(&existing) {
                                eprintln!(
                                    "Failed to remove file {}: {e}",
                                    existing.to_string_lossy()
                                );
                            }
                        }
                        sender.input(Self::Input::DoInstall(WIVRN_LATEST_RELEASE_APK_URL.into()));
                    }
                    Ok(WivrnApkRef::Tag(tag)) => {
                        sender.input(Self::Input::DoInstall(
                            format!("https://github.com/WiVRn/WiVRn/releases/download/{tag}/WiVRn-standard-release.apk")
                        ));
                    }
                    Ok(WivrnApkRef::Commit(commit)) => {
                        sender.input(Self::Input::DoInstall(
                            format!("https://github.com/WiVRn/WiVRn-APK/releases/download/apk-${commit}/org.meumeu.wivrn-standard-release.apk")
                        ));
                    }
                };
            }
            Self::Input::DoInstall(url) => {
                // TODO: we gonna cache or just download async every time?
                match cache_file(&url, Some("apk")).await {
                    Err(e) => {
                        eprintln!("Failed to download apk: {e}");
                        self.set_install_wivrn_status(InstallWivrnStatus::Done(Some(
                            "Error downloading WiVRn client APK".into(),
                        )));
                    }
                    Ok(apk_path) => {
                        self.set_install_wivrn_status(match async_process(
                            "adb",
                            Some(&["install", apk_path.to_string_lossy().to_string().as_str()]),
                            None,
                        )
                        .await
                        {
                            Ok(out) if out.exit_code == 0 => {
                                InstallWivrnStatus::Success
                            }
                            Ok(out) => {
                                if out.stdout.contains(ADB_ERR_NO_DEV)
                                    || out.stderr.contains(ADB_ERR_NO_DEV)
                                {
                                    InstallWivrnStatus::Done(Some(
                                        concat!(
                                            "No devices connected. Make sure your headset is connected ",
                                            "to this computer and USB debugging is turned on."
                                        )
                                        .into(),
                                    ))
                                } else {
                                    eprintln!("Error: ADB failed with code {}.\nstdout:\n{}\n======\nstderr:\n{}", out.exit_code, out.stdout, out.stderr);
                                    InstallWivrnStatus::Done(Some(
                                        format!("ADB exited with code \"{}\"", out.exit_code)
                                    ))
                                }
                            }
                            Err(e) => {
                                eprintln!("Error: failed to run ADB: {e}");
                                InstallWivrnStatus::Done(Some(
                                    "Failed to run ADB".into()
                                ))
                            }
                        });
                    }
                };
            }
            Self::Input::UpdateSelectedProfile(p) => {
                self.set_selected_profile(p);
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
            root_win: init.root_win,
            tracker: 0,
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }
}

new_action_group!(pub InstallWivrnActionGroup, "installwivrn");
new_stateless_action!(pub WivrnApkStandardAction, InstallWivrnActionGroup, "apkoculus");
