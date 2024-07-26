use super::about_dialog::AboutDialog;
use super::alert::{alert, alert_w_widget};
use super::build_window::{BuildStatus, BuildWindow};
use super::cmdline_opts::CmdLineOpts;
use super::debug_view::{DebugView, DebugViewMsg};
use super::job_worker::internal_worker::JobWorkerOut;
use super::job_worker::job::{FuncWorkerOut, WorkerJob};
use super::job_worker::JobWorker;
use super::libsurvive_setup_window::LibsurviveSetupWindow;
use super::main_view::MainViewMsg;
use super::util::{copy_text, open_with_default_handler};
use super::wivrn_conf_editor::{WivrnConfEditor, WivrnConfEditorInit, WivrnConfEditorMsg};
use crate::builders::build_basalt::get_build_basalt_jobs;
use crate::builders::build_libsurvive::get_build_libsurvive_jobs;
use crate::builders::build_mercury::get_build_mercury_jobs;
use crate::builders::build_monado::get_build_monado_jobs;
use crate::builders::build_opencomposite::get_build_opencomposite_jobs;
use crate::builders::build_openhmd::get_build_openhmd_jobs;
use crate::builders::build_wivrn::get_build_wivrn_jobs;
use crate::config::Config;
use crate::constants::APP_NAME;
use crate::dependencies::basalt_deps::get_missing_basalt_deps;
use crate::dependencies::common::dep_pkexec;
use crate::dependencies::libsurvive_deps::get_missing_libsurvive_deps;
use crate::dependencies::mercury_deps::get_missing_mercury_deps;
use crate::dependencies::monado_deps::get_missing_monado_deps;
use crate::dependencies::openhmd_deps::get_missing_openhmd_deps;
use crate::dependencies::wivrn_deps::get_missing_wivrn_deps;
use crate::file_builders::active_runtime_json::{
    set_current_active_runtime_to_profile, set_current_active_runtime_to_steam,
};
use crate::file_builders::openvrpaths_vrpath::{
    set_current_openvrpaths_to_profile, set_current_openvrpaths_to_steam,
};
use crate::file_utils::setcap_cap_sys_nice_eip;
use crate::is_appimage::IS_APPIMAGE;
use crate::linux_distro::LinuxDistro;
use crate::openxr_prober::is_openxr_ready;
use crate::paths::{get_data_dir, get_ipc_file_path};
use crate::profile::{Profile, XRServiceType};
use crate::stateless_action;
use crate::steam_linux_runtime_injector::{
    restore_runtime_entrypoint, set_runtime_entrypoint_launch_opts_from_profile,
};
use crate::ui::build_window::{BuildWindowMsg, BuildWindowOutMsg};
use crate::ui::debug_view::{DebugViewInit, DebugViewOutMsg};
use crate::ui::libsurvive_setup_window::LibsurviveSetupMsg;
use crate::ui::main_view::{MainView, MainViewInit, MainViewOutMsg};
use crate::xr_devices::XRDevice;
use gtk::glib::clone;
use gtk::prelude::*;
use relm4::actions::{AccelsPlus, ActionGroupName, RelmAction, RelmActionGroup};
use relm4::adw::prelude::MessageDialogExt;
use relm4::adw::ResponseAppearance;
use relm4::gtk::glib;
use relm4::{new_action_group, new_stateful_action, new_stateless_action, prelude::*};
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use std::collections::VecDeque;
use std::fs::remove_file;
use std::time::Duration;

#[tracker::track]
pub struct App {
    enable_debug_view: bool,

    #[tracker::do_not_track]
    application: adw::Application,
    #[tracker::do_not_track]
    app_win: adw::ApplicationWindow,
    #[tracker::do_not_track]
    inhibit_id: Option<u32>,

    #[tracker::do_not_track]
    main_view: Controller<MainView>,
    #[tracker::do_not_track]
    debug_view: Controller<DebugView>,
    #[tracker::do_not_track]
    split_view: Option<adw::NavigationSplitView>,
    #[tracker::do_not_track]
    about_dialog: Controller<AboutDialog>,
    #[tracker::do_not_track]
    build_window: Controller<BuildWindow>,
    #[tracker::do_not_track]
    setcap_confirm_dialog: adw::MessageDialog,
    #[tracker::do_not_track]
    libsurvive_setup_window: Controller<LibsurviveSetupWindow>,

    #[tracker::do_not_track]
    config: Config,
    #[tracker::do_not_track]
    xrservice_worker: Option<JobWorker>,
    #[tracker::do_not_track]
    autostart_worker: Option<JobWorker>,
    #[tracker::do_not_track]
    restart_xrservice: bool,
    #[tracker::do_not_track]
    build_worker: Option<JobWorker>,
    #[tracker::do_not_track]
    profiles: Vec<Profile>,
    #[tracker::do_not_track]
    xr_devices: Vec<XRDevice>,
    #[tracker::do_not_track]
    libmonado: Option<libmonado_rs::Monado>,

    #[tracker::do_not_track]
    wivrn_conf_editor: Option<Controller<WivrnConfEditor>>,

    #[tracker::do_not_track]
    skip_depcheck: bool,

    #[tracker::do_not_track]
    wivrn_appimage_warn_shown: bool,
    #[tracker::do_not_track]
    configure_wivrn_action: gtk::gio::SimpleAction,

    #[tracker::do_not_track]
    openxr_prober_worker: Option<JobWorker>,
}

#[derive(Debug)]
pub enum Msg {
    OnServiceLog(Vec<String>),
    OnServiceExit(i32),
    OnAutostartExit(i32),
    OnBuildLog(Vec<String>),
    OnBuildExit(i32),
    ClockTicking,
    BuildProfile(bool),
    CancelBuild,
    EnableDebugViewChanged(bool),
    DoStartStopXRService,
    StartWithDebug,
    RestartXRService,
    ProfileSelected(Profile),
    DeleteProfile,
    SaveProfile(Profile),
    RunSetCap,
    OpenLibsurviveSetup,
    SaveWinSize(i32, i32),
    Quit,
    DebugOpenPrefix,
    DebugOpenData,
    DebugCopyEnvVars,
    OpenWivrnConfig,
    HandleCommandLine(CmdLineOpts),
    StartProber,
    OnProberExit(bool),
    NoOp,
}

impl App {
    pub fn get_selected_profile(&self) -> Profile {
        self.config.get_selected_profile(&self.profiles)
    }

    pub fn set_inhibit_session(&mut self, state: bool) {
        if state {
            if self.inhibit_id.is_some() {
                return;
            }
            self.inhibit_id = Some(self.application.inhibit(
                Some(&self.app_win),
                gtk::ApplicationInhibitFlags::all(),
                Some("XR Session running"),
            ));
        } else {
            if self.inhibit_id.is_none() {
                return;
            }
            self.application.uninhibit(self.inhibit_id.unwrap());
            self.inhibit_id = None;
        }
    }

    pub fn start_xrservice(&mut self, sender: ComponentSender<Self>, debug: bool) {
        let prof = self.get_selected_profile();
        if let Err(e) = set_current_active_runtime_to_profile(&prof) {
            alert(
                "Failed to start XR Service",
                Some(&format!(
                    "Error setting current active runtime to profile: {e}"
                )),
                Some(&self.app_win.clone().upcast::<gtk::Window>()),
            );
            return;
        }
        if let Err(e) = set_current_openvrpaths_to_profile(&prof) {
            alert(
                "Failed to start XR Service",
                Some(&format!(
                    "Error setting current openvrpaths file to profile: {e}"
                )),
                Some(&self.app_win.clone().upcast::<gtk::Window>()),
            );
            return;
        };
        self.debug_view.sender().emit(DebugViewMsg::ClearLog);
        self.xr_devices = vec![];
        if prof.can_start() {
            remove_file(get_ipc_file_path(&prof.xrservice_type))
                .is_err()
                .then(|| println!("Failed to remove xrservice IPC file"));
            let worker = JobWorker::xrservice_worker_wrap_from_profile(
                &prof,
                sender.input_sender(),
                |msg| match msg {
                    JobWorkerOut::Log(rows) => Msg::OnServiceLog(rows),
                    JobWorkerOut::Exit(code) => Msg::OnServiceExit(code),
                },
                debug,
            );
            worker.start();
            self.xrservice_worker = Some(worker);
            self.main_view
                .sender()
                .emit(MainViewMsg::XRServiceActiveChanged(
                    true,
                    Some(self.get_selected_profile()),
                    // show launch opts only if setting the runtime entrypoint fails
                    set_runtime_entrypoint_launch_opts_from_profile(&prof).is_err(),
                ));
            self.debug_view
                .sender()
                .emit(DebugViewMsg::XRServiceActiveChanged(true));
            self.set_inhibit_session(true);
            sender.input(Msg::StartProber);
        } else {
            alert(
                "Failed to start profile",
                Some(concat!(
                    "You need to build the current profile before starting it.",
                    "\n\nYou can do this from the menu."
                )),
                Some(&self.app_win.clone().upcast::<gtk::Window>()),
            );
        }
    }

    pub fn run_autostart(&mut self, sender: ComponentSender<Self>) {
        let prof = self.get_selected_profile();
        if let Some(autostart_cmd) = &prof.autostart_command {
            let mut jobs = VecDeque::new();
            jobs.push_back(WorkerJob::new_cmd(
                Some(prof.environment.clone()),
                "sh".into(),
                Some(vec!["-c".into(), autostart_cmd.clone()]),
            ));
            let autostart_worker = JobWorker::new(jobs, sender.input_sender(), |msg| match msg {
                JobWorkerOut::Log(rows) => Msg::OnServiceLog(rows),
                JobWorkerOut::Exit(code) => Msg::OnAutostartExit(code),
            });
            autostart_worker.start();
            self.autostart_worker = Some(autostart_worker);
        }
    }

    pub fn restore_openxr_openvr_files(&self) {
        restore_runtime_entrypoint();
        if let Err(e) = set_current_active_runtime_to_steam() {
            alert(
                "Could not restore Steam active runtime",
                Some(&format!("{e}")),
                Some(&self.app_win.clone().upcast::<gtk::Window>()),
            );
        }
        if let Err(e) = set_current_openvrpaths_to_steam() {
            alert(
                "Could not restore Steam openvrpaths",
                Some(&format!("{e}")),
                Some(&self.app_win.clone().upcast::<gtk::Window>()),
            );
        };
    }

    pub fn shutdown_xrservice(&mut self) {
        if let Some(w) = self.openxr_prober_worker.as_ref() {
            w.stop();
            // this can cause threads to remain hanging...
            self.openxr_prober_worker = None;
        }
        self.set_inhibit_session(false);
        if let Some(worker) = self.xrservice_worker.as_ref() {
            worker.stop();
        }
        if let Some(worker) = self.autostart_worker.as_ref() {
            worker.stop();
        }
        self.libmonado = None;
        self.main_view
            .sender()
            .emit(MainViewMsg::XRServiceActiveChanged(false, None, false));
        self.debug_view
            .sender()
            .emit(DebugViewMsg::XRServiceActiveChanged(false));
        self.xr_devices = vec![];
    }
}

#[derive(Debug)]
pub struct AppInit {
    pub application: adw::Application,
}

#[relm4::component(pub)]
impl SimpleComponent for App {
    type Init = AppInit;
    type Input = Msg;
    type Output = ();

    view! {
        #[root]
        adw::ApplicationWindow {
            set_title: Some(APP_NAME),
            set_default_size: (win_size[0], win_size[1]),
            set_width_request: 390,
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: true,
                set_vexpand: true,
                #[name = "split_view"]
                adw::NavigationSplitView {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_sidebar: Some(&adw::NavigationPage::new(model.main_view.widget(), APP_NAME)),
                    set_content: Some(&adw::NavigationPage::new(model.debug_view.widget(), "Debug View")),
                    set_show_content: false,
                    set_collapsed: !model.config.debug_view_enabled,
                }
            },
            connect_close_request[sender] => move |win| {
                sender.input(Msg::SaveWinSize(win.width(), win.height()));
                gtk::glib::Propagation::Proceed
            }
        }
    }

    fn shutdown(&mut self, _widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
        if let Some(worker) = self.xrservice_worker.as_ref() {
            worker.stop();
        }
        self.restore_openxr_openvr_files();
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        self.reset();

        match message {
            Msg::NoOp => {}
            Msg::OnServiceLog(rows) => {
                if !rows.is_empty() {
                    self.debug_view
                        .sender()
                        .emit(DebugViewMsg::LogUpdated(rows));
                }
            }
            Msg::OnServiceExit(code) => {
                self.restore_openxr_openvr_files();
                self.main_view
                    .sender()
                    .emit(MainViewMsg::XRServiceActiveChanged(false, None, false));
                self.debug_view
                    .sender()
                    .emit(DebugViewMsg::XRServiceActiveChanged(false));
                if code != 0 && code != 15 {
                    // 15 is SIGTERM
                    sender.input(Msg::OnServiceLog(vec![format!(
                        "{} exited with code {}",
                        self.get_selected_profile().xrservice_type,
                        code
                    )]));
                }
                self.xrservice_worker = None;
                if self.restart_xrservice {
                    self.restart_xrservice = false;
                    self.start_xrservice(sender, false);
                }
            }
            Msg::OnAutostartExit(_) => self.autostart_worker = None,
            Msg::ClockTicking => {
                self.main_view.sender().emit(MainViewMsg::ClockTicking);
                if let Some(w) = self.xrservice_worker.as_ref() {
                    let stop_condition = {
                        let state = w.state.lock().unwrap();
                        state.exit_status.is_none() && !state.stop_requested
                    };
                    if stop_condition {
                        if let Some(monado) = self.libmonado.as_ref() {
                            self.xr_devices = XRDevice::merge(
                                &self.xr_devices,
                                &XRDevice::from_libmonado(monado),
                            );
                            self.main_view
                                .sender()
                                .emit(MainViewMsg::UpdateDevices(self.xr_devices.clone()));
                        } else if let Some(so) = self.get_selected_profile().libmonado_so() {
                            self.libmonado = libmonado_rs::Monado::create(so).ok();
                            if self.libmonado.is_some() {
                                sender.input(Msg::ClockTicking);
                            }
                        }
                    }
                }
            }
            Msg::EnableDebugViewChanged(val) => {
                self.set_enable_debug_view(val);
                self.config.debug_view_enabled = val;
                self.config.save();
                self.split_view.clone().unwrap().set_collapsed(!val);
                self.main_view
                    .sender()
                    .emit(MainViewMsg::EnableDebugViewChanged(val));
            }
            Msg::DoStartStopXRService => match &mut self.xrservice_worker {
                None => {
                    self.start_xrservice(sender, false);
                }
                Some(_) => {
                    self.shutdown_xrservice();
                }
            },
            Msg::StartWithDebug => self.start_xrservice(sender, true),
            Msg::RestartXRService => match &mut self.xrservice_worker {
                None => {
                    self.start_xrservice(sender, false);
                }
                Some(worker) => {
                    let status = worker.state.lock().unwrap().exit_status;
                    match status {
                        Some(_) => {
                            self.start_xrservice(sender, false);
                        }
                        None => {
                            self.shutdown_xrservice();
                            self.restart_xrservice = true;
                        }
                    }
                }
            },
            Msg::BuildProfile(clean_build) => {
                let profile = self.get_selected_profile();
                let mut missing_deps = vec![];
                let mut jobs = VecDeque::<WorkerJob>::new();
                // profile per se can't be built, but we still need opencomp
                if profile.can_be_built {
                    missing_deps.extend(match profile.xrservice_type {
                        XRServiceType::Monado => get_missing_monado_deps(),
                        XRServiceType::Wivrn => get_missing_wivrn_deps(),
                    });
                    if profile.features.libsurvive.enabled {
                        missing_deps.extend(get_missing_libsurvive_deps());
                        jobs.extend(get_build_libsurvive_jobs(&profile, clean_build));
                    }
                    if profile.features.openhmd.enabled {
                        missing_deps.extend(get_missing_openhmd_deps());
                        jobs.extend(get_build_openhmd_jobs(&profile, clean_build));
                    }
                    if profile.features.basalt.enabled {
                        missing_deps.extend(get_missing_basalt_deps());
                        jobs.extend(get_build_basalt_jobs(&profile, clean_build));
                    }
                    if profile.features.mercury_enabled {
                        missing_deps.extend(get_missing_mercury_deps());
                        jobs.extend(get_build_mercury_jobs(&profile));
                    }
                    jobs.extend(match profile.xrservice_type {
                        XRServiceType::Monado => get_build_monado_jobs(&profile, clean_build),
                        XRServiceType::Wivrn => get_build_wivrn_jobs(&profile, clean_build),
                    });
                    // no listed deps for opencomp
                }
                jobs.extend(get_build_opencomposite_jobs(&profile, clean_build));
                if !(self.skip_depcheck || missing_deps.is_empty()) {
                    missing_deps.sort_unstable();
                    missing_deps.dedup(); // dedup only works if sorted, hence the above
                    let distro = LinuxDistro::get();
                    let (missing_package_list, install_missing_widget): (
                        String,
                        Option<gtk::Widget>,
                    ) = if let Some(d) = distro {
                        (
                            missing_deps
                                .iter()
                                .map(|dep| dep.package_name_for_distro(Some(&d)))
                                .collect::<Vec<String>>()
                                .join(", "),
                            {
                                let packages = missing_deps
                                    .iter()
                                    .filter_map(|dep| dep.packages.get(&d).cloned())
                                    .collect::<Vec<String>>();
                                if packages.is_empty() {
                                    None
                                } else {
                                    let cmd = d.install_command(&packages);
                                    {
                                        let container = gtk::Box::builder()
                                            .orientation(gtk::Orientation::Horizontal)
                                            .spacing(6)
                                            .build();
                                        let btn = gtk::Button::builder()
                                            .css_classes(["flat", "circular"])
                                            .tooltip_text("Copy")
                                            .icon_name("edit-copy-symbolic")
                                            .vexpand(false)
                                            .hexpand(false)
                                            .valign(gtk::Align::Center)
                                            .halign(gtk::Align::Center)
                                            .build();
                                        btn.connect_clicked(clone!(
                                            #[to_owned]
                                            cmd,
                                            move |_| copy_text(&cmd)
                                        ));
                                        container.append(
                                            &gtk::ScrolledWindow::builder()
                                                .vscrollbar_policy(gtk::PolicyType::Never)
                                                .hscrollbar_policy(gtk::PolicyType::Automatic)
                                                .css_classes(["card"])
                                                .overflow(gtk::Overflow::Hidden)
                                                .child(
                                                    &gtk::TextView::builder()
                                                        .hexpand(true)
                                                        .vexpand(false)
                                                        .monospace(true)
                                                        .editable(false)
                                                        .left_margin(6)
                                                        .right_margin(6)
                                                        .top_margin(6)
                                                        .bottom_margin(18)
                                                        .buffer(
                                                            &gtk::TextBuffer::builder()
                                                                .text(&cmd)
                                                                .enable_undo(false)
                                                                .build(),
                                                        )
                                                        .build(),
                                                )
                                                .build(),
                                        );
                                        container.append(&btn);
                                        Some(container.upcast())
                                    }
                                }
                            },
                        )
                    } else {
                        (
                            missing_deps
                                .iter()
                                .map(|dep| dep.name.clone())
                                .collect::<Vec<String>>()
                                .join(", "),
                            None,
                        )
                    };
                    alert_w_widget(
                        "Missing dependencies:",
                        Some(&format!(
                            "{}{}",
                            missing_package_list,
                            if install_missing_widget.is_some() {
                                "\n\nYou can install them with the following command:"
                            } else {
                                ""
                            }
                        )),
                        install_missing_widget.as_ref(),
                        Some(&self.app_win.clone().upcast::<gtk::Window>()),
                    );
                    return;
                }
                self.build_window
                    .sender()
                    .send(BuildWindowMsg::Present)
                    .unwrap();
                let worker = JobWorker::new(jobs, sender.input_sender(), |msg| match msg {
                    JobWorkerOut::Log(rows) => Msg::OnBuildLog(rows),
                    JobWorkerOut::Exit(code) => Msg::OnBuildExit(code),
                });
                worker.start();
                self.build_window
                    .sender()
                    .emit(BuildWindowMsg::UpdateTitle(format!(
                        "Building Profile {}",
                        profile.name
                    )));
                self.build_window
                    .sender()
                    .emit(BuildWindowMsg::UpdateCanClose(false));
                self.build_worker = Some(worker);
            }
            Msg::OnBuildLog(rows) => {
                self.build_window
                    .sender()
                    .emit(BuildWindowMsg::UpdateContent(rows));
            }
            Msg::OnBuildExit(code) => {
                match code {
                    0 => {
                        self.build_window
                            .sender()
                            .emit(BuildWindowMsg::UpdateBuildStatus(BuildStatus::Done));
                        if self.get_selected_profile().xrservice_type == XRServiceType::Monado {
                            self.setcap_confirm_dialog.present();
                        }
                        self.build_window
                            .sender()
                            .emit(BuildWindowMsg::UpdateCanClose(true));
                    }
                    errcode => {
                        self.build_window
                            .sender()
                            .emit(BuildWindowMsg::UpdateBuildStatus(BuildStatus::Error(
                                format!("Exit status {}", errcode),
                            )));
                    }
                };
            }
            Msg::CancelBuild => {
                if let Some(w) = self.build_worker.as_ref() {
                    w.stop();
                }
            }
            Msg::DeleteProfile => {
                let todel = self.get_selected_profile();
                if todel.editable {
                    self.config.user_profiles.retain(|p| p.uuid != todel.uuid);
                    self.config.save();
                    self.profiles = self.config.profiles();
                    self.main_view
                        .sender()
                        .emit(MainViewMsg::UpdateSelectedProfile(
                            self.get_selected_profile(),
                        ));
                    self.main_view.sender().emit(MainViewMsg::UpdateProfiles(
                        self.profiles.clone(),
                        self.config.clone(),
                    ))
                }
            }
            Msg::SaveProfile(prof) => {
                match self.profiles.iter().position(|p| p.uuid == prof.uuid) {
                    None => {}
                    Some(index) => {
                        self.profiles.remove(index);
                    }
                }
                self.profiles.push(prof.clone());
                self.profiles.sort_unstable_by(|a, b| a.name.cmp(&b.name));
                self.config.set_profiles(&self.profiles);
                self.config.selected_profile_uuid = prof.uuid;
                self.config.save();
                self.main_view.sender().emit(MainViewMsg::UpdateProfiles(
                    self.profiles.clone(),
                    self.config.clone(),
                ));
            }
            Msg::RunSetCap => {
                if !dep_pkexec().check() {
                    println!("pkexec not found, skipping setcap");
                } else {
                    let profile = self.get_selected_profile();
                    setcap_cap_sys_nice_eip(&profile.prefix.join("bin/monado-service"));
                }
            }
            Msg::ProfileSelected(prof) => {
                if *IS_APPIMAGE
                    && !self.wivrn_appimage_warn_shown
                    && prof.xrservice_type == XRServiceType::Wivrn
                {
                    alert(
                        "Envision is running as an AppImage",
                        Some("WiVRn does not support running inside an AppImage, and is likely to fail. You might want to install Envision using a different method."),
                        Some(&self.app_win.clone().upcast()),
                    );
                    self.wivrn_appimage_warn_shown = true;
                }
                self.configure_wivrn_action
                    .set_enabled(prof.xrservice_type == XRServiceType::Wivrn);
                if prof.uuid == self.config.selected_profile_uuid {
                    return;
                }
                self.config.selected_profile_uuid = prof.uuid;
                self.config.save();
                let profile = self.get_selected_profile();
                self.main_view
                    .sender()
                    .emit(MainViewMsg::UpdateSelectedProfile(profile.clone()));
            }
            Msg::OpenLibsurviveSetup => {
                self.libsurvive_setup_window
                    .sender()
                    .send(LibsurviveSetupMsg::Present(
                        self.get_selected_profile().clone(),
                    ))
                    .expect("Failed to present Libsurvive Setup Window");
            }
            Msg::SaveWinSize(w, h) => {
                self.config.win_size = [w, h];
                self.config.save();
            }
            Msg::Quit => {
                sender.input(Msg::SaveWinSize(
                    self.app_win.width(),
                    self.app_win.height(),
                ));
                self.application.quit();
            }
            Msg::DebugOpenData => {
                open_with_default_handler(&format!("file://{}", get_data_dir().to_string_lossy()));
            }
            Msg::DebugOpenPrefix => {
                open_with_default_handler(&format!(
                    "file://{}",
                    self.get_selected_profile().prefix.to_string_lossy()
                ));
            }
            Msg::DebugCopyEnvVars => {
                copy_text(&self.get_selected_profile().env_vars_full().join(" "));
            }
            Msg::OpenWivrnConfig => {
                let editor = WivrnConfEditor::builder()
                    .launch(WivrnConfEditorInit {
                        root_win: self.app_win.clone().upcast::<gtk::Window>(),
                    })
                    .detach();
                editor.emit(WivrnConfEditorMsg::Present);
                self.wivrn_conf_editor = Some(editor);
            }
            Msg::HandleCommandLine(opts) => {
                if let Some(prof_uuid) = opts.profile_uuid {
                    if let Some(index) = self.profiles.iter().position(|p| p.uuid == prof_uuid) {
                        let target = self.profiles.get(index).unwrap();
                        sender.input(Msg::ProfileSelected(target.clone()));
                        self.main_view
                            .sender()
                            .emit(MainViewMsg::SetSelectedProfile(index as u32));
                    }
                }
                if opts.start {
                    sender.input(Msg::DoStartStopXRService)
                }
                if opts.skip_depcheck {
                    self.skip_depcheck = true;
                }
            }
            Msg::StartProber => {
                if self
                    .openxr_prober_worker
                    .as_ref()
                    .is_some_and(|w| w.exit_code().is_none())
                {
                    eprintln!(">>>>>>>>>>>>>>> prober already running!");
                    // prober already running
                    return;
                }
                let worker = JobWorker::new_with_timer(
                    Duration::from_millis(500),
                    WorkerJob::new_func(Box::new(move || {
                        println!(">>>>>>>>>>>>>>> probing now <<<<<<<<<<<<<<<<<<");
                        let ready = is_openxr_ready();
                        println!(">>>>>>>>>>>>>>> ready? {}", ready);
                        FuncWorkerOut {
                            success: ready,
                            ..Default::default()
                        }
                    })),
                    sender.input_sender(),
                    |msg| match msg {
                        JobWorkerOut::Exit(code) => Self::Input::OnProberExit(code == 0),
                        _ => Self::Input::NoOp,
                    },
                );
                worker.start();
                self.openxr_prober_worker = Some(worker);
            }
            Msg::OnProberExit(success) => {
                self.main_view
                    .sender()
                    .emit(MainViewMsg::UpdateXrServiceReady(success));
                if success {
                    self.run_autostart(sender.clone());
                } else if self
                    .xrservice_worker
                    .as_ref()
                    .is_some_and(|w| w.exit_code().is_none())
                {
                    sender.input(Msg::StartProber);
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let config = Config::get_config();
        let win_size = config.win_size;
        let profiles = config.profiles();
        let setcap_confirm_dialog = adw::MessageDialog::builder()
            .modal(true)
            .transient_for(&root)
            .heading("Set Capabilities")
            .body(concat!(
                "We need to set certain capabilities (CAP_SYS_NICE=eip) on the ",
                "OpenXR server executable. This requires your superuser password.\n\n",
                "Do you want to continue?",
            ))
            .hide_on_close(true)
            .build();
        setcap_confirm_dialog.add_response("no", "_No");
        setcap_confirm_dialog.add_response("yes", "_Yes");
        setcap_confirm_dialog.set_response_appearance("no", ResponseAppearance::Destructive);
        setcap_confirm_dialog.set_response_appearance("yes", ResponseAppearance::Suggested);

        {
            let setcap_sender = sender.clone();
            setcap_confirm_dialog.connect_response(None, move |_, res| {
                if res == "yes" {
                    setcap_sender.input(Msg::RunSetCap);
                }
            });
        }

        let mut actions = RelmActionGroup::<AppActionGroup>::new();

        stateless_action!(
            actions,
            BuildProfileAction,
            clone!(
                #[strong]
                sender,
                move |_| {
                    sender.input_sender().emit(Msg::BuildProfile(false));
                }
            )
        );
        stateless_action!(
            actions,
            BuildProfileCleanAction,
            clone!(
                #[strong]
                sender,
                move |_| {
                    sender.input_sender().emit(Msg::BuildProfile(true));
                }
            )
        );
        stateless_action!(
            actions,
            QuitAction,
            clone!(
                #[strong]
                sender,
                move |_| {
                    sender.input(Msg::Quit);
                }
            )
        );
        stateless_action!(
            actions,
            DebugOpenDataAction,
            clone!(
                #[strong]
                sender,
                move |_| {
                    sender.input(Msg::DebugOpenData);
                }
            )
        );
        stateless_action!(
            actions,
            DebugOpenPrefixAction,
            clone!(
                #[strong]
                sender,
                move |_| {
                    sender.input(Msg::DebugOpenPrefix);
                }
            )
        );
        stateless_action!(
            actions,
            DebugCopyEnvVarsAction,
            clone!(
                #[strong]
                sender,
                move |_| {
                    sender.input(Msg::DebugCopyEnvVars);
                }
            )
        );
        // this bypasses the macro because I need the underlying gio action
        // to enable/disable it in update()
        let configure_wivrn_action = {
            let action = RelmAction::<ConfigureWivrnAction>::new_stateless(clone!(
                #[strong]
                sender,
                move |_| {
                    sender.input(Msg::OpenWivrnConfig);
                }
            ));
            let ret = action.gio_action().clone();
            actions.add_action(action);
            ret.set_enabled(false);
            ret
        };

        let mut model = App {
            tracker: 0,
            application: init.application,
            app_win: root.clone(),
            inhibit_id: None,
            main_view: MainView::builder()
                .launch(MainViewInit {
                    config: config.clone(),
                    selected_profile: config.get_selected_profile(&profiles),
                    root_win: root.clone().into(),
                })
                .forward(sender.input_sender(), |message| match message {
                    MainViewOutMsg::DoStartStopXRService => Msg::DoStartStopXRService,
                    MainViewOutMsg::RestartXRService => Msg::RestartXRService,
                    MainViewOutMsg::ProfileSelected(uuid) => Msg::ProfileSelected(uuid),
                    MainViewOutMsg::DeleteProfile => Msg::DeleteProfile,
                    MainViewOutMsg::SaveProfile(p) => Msg::SaveProfile(p),
                    MainViewOutMsg::OpenLibsurviveSetup => Msg::OpenLibsurviveSetup,
                }),
            debug_view: DebugView::builder().launch(DebugViewInit {}).forward(
                sender.input_sender(),
                |message| match message {
                    DebugViewOutMsg::StartWithDebug => Msg::StartWithDebug,
                },
            ),
            about_dialog: AboutDialog::builder()
                .transient_for(&root)
                .launch(())
                .detach(),
            build_window: BuildWindow::builder()
                .transient_for(&root)
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    BuildWindowOutMsg::CancelBuild => Msg::CancelBuild,
                }),
            libsurvive_setup_window: LibsurviveSetupWindow::builder()
                .transient_for(&root)
                .launch(())
                .detach(),
            split_view: None,
            setcap_confirm_dialog,
            enable_debug_view: config.debug_view_enabled,
            config,
            profiles,
            xrservice_worker: None,
            autostart_worker: None,
            build_worker: None,
            xr_devices: vec![],
            restart_xrservice: false,
            libmonado: None,
            wivrn_conf_editor: None,
            skip_depcheck: false,
            wivrn_appimage_warn_shown: false,
            configure_wivrn_action,
            openxr_prober_worker: None,
        };
        let widgets = view_output!();

        {
            let abd_sender = model.about_dialog.sender().clone();
            stateless_action!(actions, AboutAction, move |_| {
                abd_sender.send(()).unwrap();
            });
        }
        actions.add_action(RelmAction::<DebugViewToggleAction>::new_stateful(
            &model.enable_debug_view,
            clone!(
                #[strong]
                sender,
                move |_, state| {
                    let s = *state;
                    *state = !s;
                    sender.input(Msg::EnableDebugViewChanged(*state));
                }
            ),
        ));

        root.insert_action_group(AppActionGroup::NAME, Some(&actions.into_action_group()));

        {
            let app = &model.application;
            app.set_accelerators_for_action::<QuitAction>(&["<Control>q"]);
            app.set_accelerators_for_action::<BuildProfileAction>(&["F5"]);
            app.set_accelerators_for_action::<BuildProfileCleanAction>(&["<Control>F5"]);
        }

        model.main_view.sender().emit(MainViewMsg::UpdateProfiles(
            model.profiles.clone(),
            model.config.clone(),
        ));

        glib::timeout_add_local(
            Duration::from_millis(1000),
            clone!(
                #[strong]
                sender,
                move || {
                    sender.input(Msg::ClockTicking);
                    glib::ControlFlow::Continue
                }
            ),
        );

        model.split_view = Some(widgets.split_view.clone());

        ComponentParts { model, widgets }
    }
}

new_action_group!(pub AppActionGroup, "win");
new_stateless_action!(pub AboutAction, AppActionGroup, "about");
new_stateless_action!(pub BuildProfileAction, AppActionGroup, "buildprofile");
new_stateless_action!(pub BuildProfileCleanAction, AppActionGroup, "buildprofileclean");
new_stateless_action!(pub QuitAction, AppActionGroup, "quit");
new_stateful_action!(pub DebugViewToggleAction, AppActionGroup, "debugviewtoggle", (), bool);
new_stateless_action!(pub ConfigureWivrnAction, AppActionGroup, "configurewivrn");

new_stateless_action!(pub DebugOpenDataAction, AppActionGroup, "debugopendata");
new_stateless_action!(pub DebugOpenPrefixAction, AppActionGroup, "debugopenprefix");
new_stateless_action!(pub DebugCopyEnvVarsAction, AppActionGroup, "debugcopyenvvars");
