use super::{alert::alert, ADB_ERR_NO_DEV};
use crate::{
    async_process::async_process,
    constants::APP_NAME,
    depcheck::common::dep_adb,
    profile::{Profile, XRServiceType},
};
use gtk::prelude::*;
use tracing::error;
use relm4::prelude::*;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum StartClientStatus {
    Success,
    Done(Option<String>),
    InProgress,
}

#[tracker::track]
pub struct WivrnWiredStartBox {
    selected_profile: Profile,
    start_client_status: StartClientStatus,
    #[tracker::do_not_track]
    root_win: gtk::Window,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum WivrnWiredStartBoxMsg {
    UpdateSelectedProfile(Profile),
    /// prepares state for async action, calls DoStartClient
    StartWivrnClient,
    /// actually sends the adb commands to start the client
    DoStartClient,
}

#[derive(Debug)]
pub struct WivrnWiredStartBoxInit {
    pub selected_profile: Profile,
    pub root_win: gtk::Window,
}

#[relm4::component(pub async)]
impl AsyncComponent for WivrnWiredStartBox {
    type Init = WivrnWiredStartBoxInit;
    type Input = WivrnWiredStartBoxMsg;
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
                set_label: "Start WiVRn Client (Wired)",
                set_wrap: true,
                set_wrap_mode: gtk::pango::WrapMode::Word,
            },
            gtk::Label {
                add_css_class: "dim-label",
                set_hexpand: true,
                set_label: concat!(
                    "Start the WiVRn client on your Android headset. This only ",
                    "works in wired connection mode."
                ),
                set_xalign: 0.0,
                set_wrap: true,
                set_wrap_mode: gtk::pango::WrapMode::Word,
            },
            gtk::Button {
                add_css_class: "suggested-action",
                set_label: "Start WiVRn Client",
                set_halign: gtk::Align::Start,
                #[track = "model.changed(Self::start_client_status())"]
                set_sensitive: model.start_client_status != StartClientStatus::InProgress,
                connect_clicked[sender] => move |_| {
                     sender.input(Self::Input::StartWivrnClient)
                },
            },
            gtk::Label {
                add_css_class: "error",
                set_xalign: 0.0,
                set_wrap: true,
                set_wrap_mode: gtk::pango::WrapMode::Word,
                #[track = "model.changed(Self::start_client_status())"]
                set_visible: matches!(&model.start_client_status, StartClientStatus::Done(Some(_))),
                #[track = "model.changed(Self::start_client_status())"]
                set_label: match &model.start_client_status {
                    StartClientStatus::Done(Some(err)) => err.as_str(),
                    _ => "",
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
            Self::Input::UpdateSelectedProfile(p) => self.set_selected_profile(p),
            Self::Input::StartWivrnClient => {
                if !dep_adb().check() {
                    alert("ADB is not installed", Some(&format!("Please install ADB on your computer to start the WiVRn client from {}.", APP_NAME)), Some(&self.root_win));
                    return;
                }
                self.set_start_client_status(StartClientStatus::InProgress);
                sender.input(Self::Input::DoStartClient);
            }
            Self::Input::DoStartClient => {
                let n_status = match async_process(
                    "sh",
                    Some(&[
                        "-c",
                        concat!(
                            "set -e;",
                            // package from store is org.meumeu.wivrn
                            // package from github is org.meumeu.wivrn.github
                            // locally built one is org.meumeu.wivrn.local
                            // select by preference "local", then "github" and last store one
                            "PACKAGE=$(adb shell pm list packages org.meumeu.wivrn | sort | tail -1 | sed 's/package://');",
                            "adb reverse tcp:9757 tcp:9757 && ",
                            "adb shell am force-stop $PACKAGE && ",
                            // wait for force-stop
                            "sleep 1 && ",
                            "adb shell am start -a android.intent.action.VIEW -d \"wivrn+tcp://127.0.0.1\" $PACKAGE"
                        )
                    ]),
                    None
                ).await {
                    Ok(out) if out.exit_code == 0 =>
                        StartClientStatus::Success
                    ,
                    Ok(out) => {
                        if out.stdout.contains(ADB_ERR_NO_DEV)
                            || out.stderr.contains(ADB_ERR_NO_DEV)
                        {
                            StartClientStatus::Done(Some(
                                concat!(
                                    "No devices connected. Make sure your headset is connected ",
                                    "to this computer and USB debugging is turned on."
                                )
                                .into(),
                            ))
                        } else {
                            error!("ADB failed with code {}.\nstdout:\n{}\n======\nstderr:\n{}", out.exit_code, out.stdout, out.stderr);
                            StartClientStatus::Done(Some(
                                format!("ADB exited with code \"{}\"", out.exit_code)
                            ))
                        }
                    },
                    Err(e) => {
                        error!("failed to run ADB: {e}");
                        StartClientStatus::Done(Some(
                            "Failed to run ADB".into()
                        ))
                    },
                };
                self.set_start_client_status(n_status);
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
            start_client_status: StartClientStatus::Done(None),
            root_win: init.root_win,
            tracker: 0,
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }
}
