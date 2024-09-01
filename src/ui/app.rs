use super::{
    about_dialog::{create_about_dialog, populate_debug_info},
    alert::{alert, alert_w_widget},
    build_window::{BuildStatus, BuildWindow, BuildWindowInit, BuildWindowMsg, BuildWindowOutMsg},
    cmdline_opts::CmdLineOpts,
    debug_view::{DebugView, DebugViewInit, DebugViewMsg, DebugViewOutMsg},
    job_worker::{
        internal_worker::JobWorkerOut,
        job::{FuncWorkerOut, WorkerJob},
        JobWorker,
    },
    libsurvive_setup_window::{LibsurviveSetupMsg, LibsurviveSetupWindow},
    main_view::{MainView, MainViewInit, MainViewMsg, MainViewOutMsg},
    util::{copiable_code_snippet, copy_text, open_with_default_handler},
    wivrn_conf_editor::{WivrnConfEditor, WivrnConfEditorInit, WivrnConfEditorMsg},
};
use crate::{
    builders::{
        build_basalt::get_build_basalt_jobs, build_libsurvive::get_build_libsurvive_jobs,
        build_mercury::get_build_mercury_jobs, build_monado::get_build_monado_jobs,
        build_opencomposite::get_build_opencomposite_jobs, build_openhmd::get_build_openhmd_jobs,
        build_wivrn::get_build_wivrn_jobs,
    },
    config::Config,
    constants::APP_NAME,
    dependencies::{
        basalt_deps::get_missing_basalt_deps, common::dep_pkexec,
        libsurvive_deps::get_missing_libsurvive_deps, mercury_deps::get_missing_mercury_deps,
        monado_deps::get_missing_monado_deps, openhmd_deps::get_missing_openhmd_deps,
        wivrn_deps::get_missing_wivrn_deps,
    },
    file_builders::{
        active_runtime_json::{
            set_current_active_runtime_to_profile, set_current_active_runtime_to_steam,
        },
        openvrpaths_vrpath::{
            set_current_openvrpaths_to_profile, set_current_openvrpaths_to_steam,
        },
    },
    file_utils::{setcap_cap_sys_nice_eip, setcap_cap_sys_nice_eip_cmd},
    is_appimage::IS_APPIMAGE,
    linux_distro::LinuxDistro,
    openxr_prober::is_openxr_ready,
    paths::get_data_dir,
    profile::{Profile, XRServiceType},
    stateless_action,
    steam_linux_runtime_injector::{
        restore_runtime_entrypoint, set_runtime_entrypoint_launch_opts_from_profile,
    },
    xr_devices::XRDevice,
};
use adw::{prelude::*, ResponseAppearance};
use gtk::glib::{self, clone};
use relm4::{
    actions::{AccelsPlus, ActionGroupName, RelmAction, RelmActionGroup},
    new_action_group, new_stateful_action, new_stateless_action,
    prelude::*,
};
use std::{collections::VecDeque, fs::remove_file, time::Duration};

pub struct App {
    application: adw::Application,
    app_win: adw::ApplicationWindow,
    inhibit_id: Option<u32>,

    main_view: Controller<MainView>,
    debug_view: Controller<DebugView>,
    split_view: Option<adw::NavigationSplitView>,
    about_dialog: adw::AboutDialog,
    build_window: Controller<BuildWindow>,
    setcap_confirm_dialog: adw::AlertDialog,
    libsurvive_setup_window: Controller<LibsurviveSetupWindow>,

    config: Config,
    xrservice_worker: Option<JobWorker>,
    autostart_worker: Option<JobWorker>,
    restart_xrservice: bool,
    build_worker: Option<JobWorker>,
    profiles: Vec<Profile>,
    xr_devices: Vec<XRDevice>,
    libmonado: Option<libmonado_rs::Monado>,

    wivrn_conf_editor: Option<Controller<WivrnConfEditor>>,
    skip_depcheck: bool,
    wivrn_appimage_warn_shown: bool,
    configure_wivrn_action: gtk::gio::SimpleAction,
    openxr_prober_worker: Option<JobWorker>,
    xrservice_ready: bool,
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
            let inhibit_id = self.application.inhibit(
                Some(&self.app_win),
                gtk::ApplicationInhibitFlags::all(),
                Some("XR session running"),
            );
            if inhibit_id == 0 {
                alert(
                    "Failed to inhibit desktop locking",
                    Some(&format!("{APP_NAME} tries to inhibit desktop locking to avoid automatic suspension or screen locking kicking in while the XR session is active, but this process failed.\n\nThe session is still running but you might want to manually disable automatic suspension and screen locking.")),
                    Some(&self.app_win.clone().upcast())
                );
            } else {
                self.inhibit_id = Some(inhibit_id);
            }
        } else if let Some(id) = self.inhibit_id {
            self.application.uninhibit(id);
            self.inhibit_id = None;
        }
    }

    pub fn start_xrservice(&mut self, sender: AsyncComponentSender<Self>, debug: bool) {
        self.xrservice_ready = false;
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
            remove_file(prof.xrservice_type.ipc_file_path())
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

    pub fn run_autostart(&mut self, sender: AsyncComponentSender<Self>) {
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
        if let Some(worker) = self.autostart_worker.as_ref() {
            worker.stop();
        }
        self.xrservice_ready = false;
        if let Some(w) = self.openxr_prober_worker.as_ref() {
            w.stop();
            // this can cause threads to remain hanging...
            self.openxr_prober_worker = None;
        }
        self.set_inhibit_session(false);
        if let Some(worker) = self.xrservice_worker.as_ref() {
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

#[relm4::component(pub async)]
impl AsyncComponent for App {
    type Init = AppInit;
    type Input = Msg;
    type Output = ();
    type CommandOutput = ();

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

    async fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
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
                let should_poll_for_devices = self.xrservice_ready
                    && self
                        .xrservice_worker
                        .as_ref()
                        .is_some_and(JobWorker::is_alive);
                if should_poll_for_devices {
                    if let Some(monado) = self.libmonado.as_ref() {
                        self.xr_devices = XRDevice::from_libmonado(monado);
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
            Msg::EnableDebugViewChanged(val) => {
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
                if !(self.skip_depcheck || profile.skip_dependency_check || missing_deps.is_empty())
                {
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
                                    Some(copiable_code_snippet(&cmd))
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
                        let profile = self.get_selected_profile();
                        if profile.xrservice_type == XRServiceType::Monado {
                            if dep_pkexec().check() {
                                self.setcap_confirm_dialog.present(Some(&self.app_win));
                            } else {
                                alert_w_widget(
                                    "pkexec not found",
                                    Some(&format!(
                                        "The build is complete, but we need to set certain capabilities (CAP_SYS_NICE=eip) on the OpenXR server executable.\n\n{APP_NAME} can do that using pkexec, but it doesn't seem to be installed on your system.\n\nYou can do this step on your own by running the following command:"
                                    )),
                                    Some(&copiable_code_snippet(
                                        &format!("sudo {}", setcap_cap_sys_nice_eip_cmd(&profile).join(" "))
                                    )),
                                    Some(&self.app_win.clone().upcast())
                                );
                            }
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
                self.config.selected_profile_uuid = prof.uuid.clone();
                self.config.save();
                self.main_view.sender().emit(MainViewMsg::UpdateProfiles(
                    self.profiles.clone(),
                    self.config.clone(),
                ));
                self.debug_view
                    .sender()
                    .emit(DebugViewMsg::UpdateSelectedProfile(prof.clone()));
            }
            Msg::RunSetCap => {
                if !dep_pkexec().check() {
                    println!("pkexec not found, skipping setcap");
                } else {
                    let profile = self.get_selected_profile();
                    setcap_cap_sys_nice_eip(&profile).await;
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
                self.debug_view
                    .sender()
                    .emit(DebugViewMsg::UpdateSelectedProfile(profile.clone()));
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
                    // prober already running
                    return;
                }
                let worker = JobWorker::new_with_timer(
                    Duration::from_millis(500),
                    WorkerJob::new_func(Box::new(move || {
                        let ready = is_openxr_ready();
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
                self.xrservice_ready = success;
                self.main_view
                    .sender()
                    .emit(MainViewMsg::UpdateXrServiceReady(true));
                if self
                    .xrservice_worker
                    .as_ref()
                    .is_some_and(JobWorker::is_alive)
                {
                    if success {
                        self.run_autostart(sender.clone());
                    } else {
                        sender.input(Msg::StartProber);
                    }
                }
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let config = Config::get_config();
        let win_size = config.win_size;
        let profiles = config.profiles();
        let setcap_confirm_dialog = adw::AlertDialog::builder()
            .heading("Set Capabilities")
            .body(concat!(
                "We need to set certain capabilities (CAP_SYS_NICE=eip) on the ",
                "OpenXR server executable. This requires your superuser password.\n\n",
                "Do you want to continue?",
            ))
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
        let selected_profile = config.get_selected_profile(&profiles);

        let mut model = App {
            application: init.application,
            app_win: root.clone(),
            inhibit_id: None,
            main_view: MainView::builder()
                .launch(MainViewInit {
                    config: config.clone(),
                    selected_profile: selected_profile.clone(),
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
            debug_view: DebugView::builder()
                .launch(DebugViewInit {
                    profile: selected_profile,
                })
                .forward(sender.input_sender(), |message| match message {
                    DebugViewOutMsg::StartWithDebug => Msg::StartWithDebug,
                }),
            about_dialog: create_about_dialog(),
            build_window: BuildWindow::builder()
                .launch(BuildWindowInit {
                    parent: root.clone().upcast(),
                })
                .forward(sender.input_sender(), |msg| match msg {
                    BuildWindowOutMsg::CancelBuild => Msg::CancelBuild,
                }),
            libsurvive_setup_window: LibsurviveSetupWindow::builder()
                .transient_for(&root)
                .launch(())
                .detach(),
            split_view: None,
            setcap_confirm_dialog,
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
            xrservice_ready: false,
        };

        let widgets = view_output!();

        stateless_action!(
            actions,
            AboutAction,
            clone!(
                #[strong(rename_to = about_dialog)]
                model.about_dialog,
                #[strong(rename_to = app_win)]
                model.app_win,
                move |_| {
                    populate_debug_info(&about_dialog);
                    about_dialog.present(Some(&app_win));
                }
            )
        );
        actions.add_action(RelmAction::<DebugViewToggleAction>::new_stateful(
            &model.config.debug_view_enabled,
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
            app.set_accelerators_for_action::<BuildProfileCleanAction>(&["<Control>F5"]);
            app.set_accelerators_for_action::<BuildProfileAction>(&["F5"]);
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

        AsyncComponentParts { model, widgets }
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
