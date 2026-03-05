#![allow(unused_assignments)] // vars named in view! macro are not correctly recognized as used

use super::{
    about_dialog::{create_about_dialog, populate_debug_info},
    alert::{alert, alert_w_widget, notification},
    build_window::{BuildStatus, BuildWindow, BuildWindowInit, BuildWindowMsg, BuildWindowOutMsg},
    cmdline_opts::CmdLineOpts,
    debug_view::{DebugView, DebugViewInit, DebugViewMsg, DebugViewOutMsg},
    job_worker::{
        JobWorker,
        internal_worker::JobWorkerOut,
        job::{FuncWorkerOut, WorkerJob},
    },
    libsurvive_setup_window::{LibsurviveSetupMsg, LibsurviveSetupWindow},
    main_view::{MainView, MainViewInit, MainViewMsg, MainViewOutMsg},
    util::{copiable_code_snippet, open_with_default_handler},
};
use crate::{
    builders::{
        build_basalt::get_build_basalt_jobs, build_libsurvive::get_build_libsurvive_jobs,
        build_mercury::get_build_mercury_jobs, build_monado::get_build_monado_jobs,
        build_opencomposite::get_build_opencomposite_jobs, build_openhmd::get_build_openhmd_jobs,
        build_vapor::get_build_vapor_jobs, build_xrizer::get_build_xrizer_jobs,
    },
    config::Config,
    constants::APP_NAME,
    depcheck::common::dep_pkexec,
    file_builders::{
        active_runtime_json::{
            remove_current_active_runtime, restore_active_runtime_backup,
            set_current_active_runtime_to_profile,
        },
        openvrpaths_vrpath::{restore_openvrpaths_backup, set_current_openvrpaths_to_profile},
    },
    linux_distro::LinuxDistro,
    openxr_prober::is_openxr_ready,
    paths::get_data_dir,
    profile::{OvrCompatibilityModuleType, Profile, XRServiceType},
    profiles::system_profile::system_profile,
    stateless_action,
    steam_linux_runtime_injector::{
        restore_sniper_runtime_entrypoint, restore_soldier_runtime_entrypoint,
        set_sniper_runtime_entrypoint_launch_opts_from_profile,
        set_soldier_runtime_entrypoint_launch_opts_from_profile,
    },
    termcolor::TermColor,
    ui::{
        alert::{present_alert, yes_no_dialog},
        preferences_window::{
            PreferencesWindow, PreferencesWindowInit, PreferencesWindowMsg, PreferencesWindowOutMsg,
        },
    },
    util::file_utils::{
        rm_rf, setcap_cap_sys_nice_eip, setcap_cap_sys_nice_eip_cmd, verify_cap_sys_nice_eip,
    },
    vulkaninfo::VulkanInfo,
    xr_clients::XRClient,
    xr_devices::XRDevice,
};
use adw::{ResponseAppearance, prelude::*};
use delicious_adwaita::{ThemeEngine, theme::Theme};
use gtk::glib::{self, clone};
use notify_rust::NotificationHandle;
use relm4::{
    actions::{AccelsPlus, ActionGroupName, RelmAction, RelmActionGroup},
    new_action_group, new_stateful_action, new_stateless_action,
    prelude::*,
};
use std::{collections::VecDeque, fs::remove_file, rc::Rc, time::Duration};
use tracing::error;

pub struct App {
    application: adw::Application,
    app_win: adw::ApplicationWindow,
    inhibit_id: Option<u32>,

    main_view: AsyncController<MainView>,
    debug_view: AsyncController<DebugView>,
    split_view: Option<adw::NavigationSplitView>,
    about_dialog: adw::AboutDialog,
    build_window: Controller<BuildWindow>,
    setcap_confirm_dialog: adw::AlertDialog,
    libsurvive_setup_window: Controller<LibsurviveSetupWindow>,

    config: Config,
    xrservice_worker: Option<JobWorker>,
    plugins_worker: Option<JobWorker>,
    restart_xrservice: bool,
    build_worker: Option<JobWorker>,
    monado_gui_worker: Option<JobWorker>,
    profiles: Vec<Profile>,
    libmonado: Option<Rc<libmonado::Monado>>,

    skip_depcheck: bool,
    build_profile_action: gtk::gio::SimpleAction,
    clean_build_profile_action: gtk::gio::SimpleAction,
    openxr_prober_worker: Option<JobWorker>,
    xrservice_ready: bool,
    vkinfo: Option<VulkanInfo>,

    inhibit_fail_notif: Option<NotificationHandle>,

    preferences_win: Option<AsyncController<PreferencesWindow>>,

    theme_engine: ThemeEngine,
    build_alerts: VecDeque<adw::AlertDialog>,
}

#[derive(Debug)]
pub enum Msg {
    OnServiceLog(Vec<String>),
    OnServiceExit(i32),
    OnPluginsExit(i32),
    OnBuildLog(Vec<String>),
    OnBuildExit(i32),
    ClockTicking,
    BuildProfile(bool),
    BuildProfileDequeueAlerts(bool),
    BuildProfilePostChecks(bool),
    CancelBuild,
    EnableDebugViewChanged(bool),
    DoStartStopXRService,
    StartWithDebug,
    RestartXRService,
    ProfileSelected(Profile),
    /// bool param: delete files
    DeleteProfile(bool),
    SaveProfile(Profile),
    RunSetCap,
    OpenLibsurviveSetup,
    SaveWinSize(i32, i32),
    Quit,
    DebugOpenPrefix,
    DebugOpenData,
    HandleCommandLine(CmdLineOpts),
    StartProber,
    OnProberExit(bool),
    OpenPreferences,
    OnPreferencesWindowClose(Config),
    ShowThemeManager,
    SaveThemeConfig,
    LaunchMonadoGui,
    OnMonadoGuiExit,
    ProfilesEnabledChanged(bool),
    NoOp,
}

impl App {
    pub fn get_selected_profile(&mut self) -> Profile {
        if self.config.profiles_enabled {
            self.config.get_selected_profile(&self.profiles)
        } else {
            system_profile(&self.config)
        }
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
                self.inhibit_fail_notif = match if let Some(notif) =
                    self.inhibit_fail_notif.as_ref()
                {
                    notif.show()
                } else {
                    notification(
                        "Failed to inhibit desktop locking",
                        &format!("{APP_NAME} tries to inhibit desktop locking to avoid automatic suspension or screen locking kicking in while the XR session is active, but this process failed.\n\nThe session is still running but you might want to manually disable automatic suspension and screen locking."),
                    ).show()
                } {
                    Ok(n) => Some(n),
                    Err(e) => {
                        error!("failed to send desktop notification: {e:?}");
                        None
                    }
                }
            } else {
                self.inhibit_id = Some(inhibit_id);
            }
        } else if let Some(id) = self.inhibit_id {
            self.application.uninhibit(id);
            self.inhibit_id = None;
        }
    }

    pub fn start_xrservice(&mut self, sender: AsyncComponentSender<Self>, debug: bool) {
        let prof = if self.config.profiles_enabled {
            self.get_selected_profile()
        } else {
            system_profile(&self.config)
        };
        self.xrservice_ready = false;
        if !prof.can_start(Some(&self.config)) {
            let desc = if prof.is_system_profile {
                format!(
                    "Executable for {} not found, you need to install it throught your package manager",
                    prof.xrservice_type
                )
            } else {
                concat!(
                    "You need to build the current profile before starting it.",
                    "\n\nYou can do this from the menu."
                )
                .into()
            };
            alert(
                "Failed to start profile",
                Some(&desc),
                Some(&self.app_win.clone().upcast::<gtk::Window>()),
            );
            return;
        }
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
        {
            let ipc_file = prof.xrservice_type.ipc_file_path();
            if ipc_file.exists() {
                remove_file(ipc_file)
                    .unwrap_or_else(|e| error!("failed to remove xrservice IPC file: {e}"));
            };
        }
        let Some(worker) = JobWorker::xrservice_worker_wrap_from_profile(
            &prof,
            &self.config,
            sender.input_sender(),
            |msg| match msg {
                JobWorkerOut::Log(rows) => Msg::OnServiceLog(rows),
                JobWorkerOut::Exit(code) => Msg::OnServiceExit(code),
            },
            debug,
        ) else {
            alert(
                "Failed to start XR Service",
                Some("Could not find XR Service executable."),
                Some(&self.app_win.clone().upcast::<gtk::Window>()),
            );
            return;
        };
        worker.start();
        self.xrservice_worker = Some(worker);
        let set_sniper_launch_opts_res =
            set_sniper_runtime_entrypoint_launch_opts_from_profile(&prof);
        let set_soldier_launch_opts_res =
            set_soldier_runtime_entrypoint_launch_opts_from_profile(&prof);
        self.main_view
            .sender()
            .emit(MainViewMsg::XRServiceActiveChanged(
                true,
                Some(prof),
                // show launch opts only if setting the runtime entrypoint fails
                set_sniper_launch_opts_res.is_err() || set_soldier_launch_opts_res.is_err(),
            ));
        self.debug_view
            .sender()
            .emit(DebugViewMsg::XRServiceActiveChanged(true));
        self.set_inhibit_session(true);
        sender.input(Msg::StartProber);
    }

    pub fn run_autostart(&mut self, sender: AsyncComponentSender<Self>) {
        let prof = self.get_selected_profile();
        let plugins_cmd = self
            .config
            .get_xr_plugins()
            .into_iter()
            .map(|plugin| plugin.exec)
            .collect::<Vec<String>>()
            .join(" & ");
        if !plugins_cmd.is_empty() {
            let mut jobs = VecDeque::new();
            jobs.push_back(WorkerJob::new_cmd(
                Some(prof.environment.clone()),
                "sh".into(),
                Some(vec!["-c".into(), plugins_cmd]),
            ));
            let plugins_worker = JobWorker::new(jobs, sender.input_sender(), |msg| match msg {
                JobWorkerOut::Log(rows) => Msg::OnServiceLog(rows),
                JobWorkerOut::Exit(code) => Msg::OnPluginsExit(code),
            });
            plugins_worker.start();
            self.plugins_worker = Some(plugins_worker);
        }
    }

    pub fn restore_openxr_openvr_files(&self) {
        restore_sniper_runtime_entrypoint();
        restore_soldier_runtime_entrypoint();
        if let Err(e) = remove_current_active_runtime() {
            alert(
                "Could not remove profile active runtime",
                Some(&format!("{e}")),
                Some(&self.app_win.clone().upcast::<gtk::Window>()),
            );
        }
        if let Err(e) = restore_active_runtime_backup() {
            alert(
                "Could not restore previous active runtime",
                Some(&format!("{e}")),
                Some(&self.app_win.clone().upcast::<gtk::Window>()),
            );
        }
        if let Err(e) = restore_openvrpaths_backup() {
            alert(
                "Could not restore openvrpaths",
                Some(&format!("{e}")),
                Some(&self.app_win.clone().upcast::<gtk::Window>()),
            );
        };
    }

    pub fn shutdown_xrservice(&mut self) {
        if let Some(w) = self.plugins_worker.as_ref() {
            w.stop();
        }
        if let Some(w) = self.monado_gui_worker.as_ref() {
            w.stop();
        }
        if let Some(w) = self.openxr_prober_worker.as_ref() {
            w.stop();
            // this can cause threads to remain hanging...
            self.openxr_prober_worker = None;
        }
        if let Some(w) = self.xrservice_worker.as_ref() {
            w.stop();
        }
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
                    set_content: Some(&adw::NavigationPage::new(model.debug_view.widget(), "Developer Mode")),
                    set_show_content: false,
                    set_collapsed: !model.config.debug_view_enabled,
                },
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
            Msg::ShowThemeManager => {
                let dialog = self
                    .theme_engine
                    .theme_chooser_dialog(Theme::default_themes().as_ref());
                dialog.set_content_height(2000);
                dialog.present(Some(&self.app_win));
                dialog.connect_closed(clone!(
                    #[strong]
                    sender,
                    move |_| {
                        sender.input(Msg::SaveThemeConfig);
                    }
                ));
            }
            Msg::SaveThemeConfig => {
                let name = self.theme_engine.current_theme_name();
                if self.config.theme_name != name {
                    self.config.theme_name = name;
                    self.config.save();
                }
            }
            Msg::OnServiceLog(rows) => {
                if !rows.is_empty() {
                    self.debug_view
                        .sender()
                        .emit(DebugViewMsg::LogUpdated(rows));
                }
            }
            Msg::OnServiceExit(code) => {
                self.set_inhibit_session(false);
                self.xrservice_ready = false;
                self.restore_openxr_openvr_files();
                self.main_view
                    .sender()
                    .emit(MainViewMsg::XRServiceActiveChanged(false, None, false));
                self.debug_view
                    .sender()
                    .emit(DebugViewMsg::XRServiceActiveChanged(false));
                self.libmonado = None;
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
            Msg::OnPluginsExit(_) => self.plugins_worker = None,
            Msg::ClockTicking => {
                self.main_view.sender().emit(MainViewMsg::ClockTicking);
                let xrservice_worker_is_alive = self
                    .xrservice_worker
                    .as_ref()
                    .is_some_and(JobWorker::is_alive);
                let should_poll_for_devices = self.xrservice_ready && xrservice_worker_is_alive;
                if should_poll_for_devices {
                    if let Some(monado) = self.libmonado.clone() {
                        self.main_view.sender().emit(MainViewMsg::UpdateDevices(
                            XRDevice::from_libmonado(monado.clone()),
                        ));
                        self.main_view
                            .sender()
                            .emit(MainViewMsg::UpdateClients(XRClient::from_libmonado(monado)));
                    } else if let Some(so) = self.get_selected_profile().libmonado_so() {
                        self.libmonado = libmonado::Monado::create(so).map(Rc::new).ok();
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
                // disabling debug view also disables profiles
                if !val {
                    sender.input(Msg::ProfilesEnabledChanged(false));
                    self.debug_view
                        .sender()
                        .emit(DebugViewMsg::ProfilesEnabledChanged(false));
                }
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
                match profile.xrservice_git().check_remote_diverges() {
                    Err(e) => {
                        error!(e);
                    }
                    Ok(false) => {}
                    Ok(true) => {
                        let sender = sender.clone();
                        let path = profile.xrservice_path.clone();
                        self.build_alerts.push_back(yes_no_dialog(
                            &format!("{} remote repository diverged", profile.xrservice_type),
                            Some(&format!(
                                "It is recommended to delete your local repository so that it will be cloned from scratch. Do you want to delete your local repository in `{}`?",
                                path.to_string_lossy()
                            )),
                            move || {
                                rm_rf(&path);
                            },
                            move || {
                                sender.input(Msg::BuildProfileDequeueAlerts(clean_build));
                            }
                        ));
                    }
                }
                match profile.ovr_comp_git().check_remote_diverges() {
                    Err(e) => {
                        error!(e);
                    }
                    Ok(false) => {}
                    Ok(true) => {
                        let sender = sender.clone();
                        let path = profile.ovr_comp.path.clone();
                        self.build_alerts.push_back(yes_no_dialog(
                            &format!("{} remote repository diverged", profile.ovr_comp.mod_type),
                            Some(&format!(
                                "It is recommended to delete your local repository so that it will be cloned from scratch. Do you want to delete your local repository in `{}`?",
                                path.to_string_lossy()
                            )),
                            move || {
                                rm_rf(&path);
                            },
                            move || {
                                sender.input(Msg::BuildProfileDequeueAlerts(clean_build));
                            }
                        ));
                    }
                }
                if let Some(basalt_git) = profile.basalt_git() {
                    match basalt_git.check_remote_diverges() {
                        Err(e) => {
                            error!(e);
                        }
                        Ok(false) => {}
                        Ok(true) => {
                            let sender = sender.clone();
                            // this unwrap is safe because the non_none-ness of path is
                            // guaranteed by the ~_git() call returning Some(Git)
                            let path = profile.features.basalt.path.as_ref().unwrap().clone();
                            self.build_alerts.push_back(yes_no_dialog(
                                "Basalt remote repository diverged",
                                Some(&format!(
                                    "It is recommended to delete your local repository so that it will be cloned from scratch. Do you want to delete your local repository in `{}`?",
                                    path.to_string_lossy()
                                )),
                                move || {
                                    rm_rf(&path);
                                },
                                move || {
                                    sender.input(Msg::BuildProfileDequeueAlerts(clean_build));
                                }
                            ));
                        }
                    }
                }
                if let Some(libsurvive_git) = profile.libsurvive_git() {
                    match libsurvive_git.check_remote_diverges() {
                        Err(e) => {
                            error!(e);
                        }
                        Ok(false) => {}
                        Ok(true) => {
                            let sender = sender.clone();
                            // this unwrap is safe because the non_none-ness of path is
                            // guaranteed by the ~_git() call returning Some(Git)
                            let path = profile.features.libsurvive.path.as_ref().unwrap().clone();
                            self.build_alerts.push_back(yes_no_dialog(
                                "Libsurvive remote repository diverged",
                                Some(&format!(
                                    "It is recommended to delete your local repository so that it will be cloned from scratch. Do you want to delete your local repository in `{}`?",
                                    path.to_string_lossy()
                                )),
                                move || {
                                    rm_rf(&path);
                                },
                                move || {
                                    sender.input(Msg::BuildProfileDequeueAlerts(clean_build));
                                }
                            ));
                        }
                    }
                }
                if let Some(openhmd_git) = profile.openhmd_git() {
                    match openhmd_git.check_remote_diverges() {
                        Err(e) => {
                            error!(e);
                        }
                        Ok(false) => {}
                        Ok(true) => {
                            let sender = sender.clone();
                            // this unwrap is safe because the non_none-ness of path is
                            // guaranteed by the ~_git() call returning Some(Git)
                            let path = profile.features.openhmd.path.as_ref().unwrap().clone();
                            self.build_alerts.push_back(yes_no_dialog(
                                "OpenHMD remote repository diverged",
                                Some(&format!(
                                    "It is recommended to delete your local repository so that it will be cloned from scratch. Do you want to delete your local repository in `{}`?",
                                    path.to_string_lossy()
                                )),
                                move || {
                                    rm_rf(&path);
                                },
                                move || {
                                    sender.input(Msg::BuildProfileDequeueAlerts(clean_build));
                                }
                            ));
                        }
                    }
                }

                sender.input(Msg::BuildProfileDequeueAlerts(clean_build))
            }
            Msg::BuildProfileDequeueAlerts(clean_build) => {
                if let Some(d) = self.build_alerts.pop_front() {
                    let parent = Some(&self.app_win.clone().upcast::<gtk::Window>());
                    present_alert(d, parent);
                } else {
                    sender.input(Msg::BuildProfilePostChecks(clean_build));
                }
            }
            Msg::BuildProfilePostChecks(clean_build) => {
                let profile = self.get_selected_profile();
                let mut jobs = VecDeque::<WorkerJob>::new();
                // system profile can't be built
                if !profile.is_system_profile {
                    if profile.features.libsurvive.enabled {
                        jobs.extend(get_build_libsurvive_jobs(&profile, clean_build));
                    }
                    if profile.features.openhmd.enabled {
                        jobs.extend(get_build_openhmd_jobs(&profile, clean_build));
                    }
                    if profile.features.basalt.enabled {
                        jobs.extend(get_build_basalt_jobs(&profile, clean_build));
                    }
                    if profile.features.mercury_enabled {
                        jobs.extend(get_build_mercury_jobs());
                    }
                    jobs.extend(match profile.xrservice_type {
                        XRServiceType::Monado => get_build_monado_jobs(&profile, clean_build),
                    });
                    jobs.extend(match profile.ovr_comp.mod_type {
                        OvrCompatibilityModuleType::Opencomposite => {
                            get_build_opencomposite_jobs(&profile, clean_build)
                        }
                        OvrCompatibilityModuleType::Xrizer => {
                            get_build_xrizer_jobs(&profile, clean_build)
                        }
                        OvrCompatibilityModuleType::Vapor => {
                            get_build_vapor_jobs(&profile, clean_build)
                        }
                    });
                }
                let missing_deps = profile.missing_dependencies();
                if !(self.skip_depcheck || profile.skip_dependency_check || missing_deps.is_empty())
                {
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
                        if dep_pkexec().check() {
                            self.setcap_confirm_dialog.present(Some(&self.app_win));
                        } else {
                            self.build_window
                                .sender()
                                .emit(BuildWindowMsg::UpdateContent(vec![
                                    TermColor::Red
                                        .colorize("pkexec not found, cannot set capabilities\n"),
                                ]));
                            if let Some(xrservice_binary) = profile.xrservice_binary(None) {
                                alert_w_widget(
                                    "pkexec not found",
                                    Some(&format!(
                                        "The build is complete, but we need to set certain capabilities (CAP_SYS_NICE=eip) on the OpenXR server executable.\n\n{APP_NAME} can do that using pkexec, but it doesn't seem to be installed on your system.\n\nYou can do this step on your own by running the following command:"
                                    )),
                                    Some(&copiable_code_snippet(&format!(
                                        "sudo {}",
                                        setcap_cap_sys_nice_eip_cmd(&xrservice_binary).join(" ")
                                    ))),
                                    Some(&self.app_win.clone().upcast()),
                                );
                            } else {
                                alert(
                                    "XR Service executable not found",
                                    Some(concat!(
                                        "The build seems to have completed successfully, ",
                                        "but the XR Service binary hasn't been found.\n\n",
                                        "Something likely went wrong and you might ",
                                        "want to report this problem."
                                    )),
                                    Some(&self.app_win.clone().upcast()),
                                )
                            }
                        }
                        self.build_window
                            .sender()
                            .emit(BuildWindowMsg::UpdateCanClose(true));
                        self.main_view
                            .sender()
                            .emit(MainViewMsg::UpdateSelectedProfile(profile));
                    }
                    errcode => {
                        self.build_window
                            .sender()
                            .emit(BuildWindowMsg::UpdateBuildStatus(BuildStatus::Error(
                                format!("Exit status {errcode}"),
                            )));
                    }
                };
            }
            Msg::CancelBuild => {
                if let Some(w) = self.build_worker.as_ref() {
                    w.stop();
                }
            }
            Msg::DeleteProfile(delete_files) => {
                let todel = self.get_selected_profile();
                if todel.editable {
                    if delete_files {
                        for res in todel.delete_files() {
                            if let Err(e) = res {
                                error!("Error deleting profile directory: {e}");
                            }
                        }
                    }
                    self.config.user_profiles.retain(|p| p.uuid != todel.uuid);
                    self.config.save();
                    self.profiles = self.config.profiles();
                    let prof = self.get_selected_profile();
                    self.main_view
                        .sender()
                        .emit(MainViewMsg::UpdateSelectedProfile(prof));
                    self.debug_view.sender().emit(DebugViewMsg::UpdateProfiles(
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
                self.debug_view.sender().emit(DebugViewMsg::UpdateProfiles(
                    self.profiles.clone(),
                    self.config.clone(),
                ));
                self.debug_view
                    .sender()
                    .emit(DebugViewMsg::UpdateSelectedProfile(prof.clone()));
                self.debug_view
                    .sender()
                    .emit(DebugViewMsg::QueryProfileRebuild);
            }
            Msg::RunSetCap => {
                if !dep_pkexec().check() {
                    // there's a precheck ahead of this, this should likely never happen
                    error!("pkexec not found, skipping setcap");
                } else {
                    let profile = self.get_selected_profile();
                    let Some(xrservice_binary) = profile.xrservice_binary(Some(&self.config))
                    else {
                        alert(
                            "XR Service executable not found",
                            Some("Cannot set executable capabilities"),
                            Some(&self.app_win.clone().upcast()),
                        );
                        return;
                    };
                    let setcap_failed_dialog = || {
                        alert_w_widget(
                            "Setcap failed to run",
                            Some(
                                "Setting the capabilities automatically failed, you can still try manually using the command below.",
                            ),
                            Some(&copiable_code_snippet(&format!(
                                "sudo {}",
                                setcap_cap_sys_nice_eip_cmd(&xrservice_binary).join(" ")
                            ))),
                            Some(&self.app_win.clone().upcast()),
                        );
                    };
                    if let Err(e) = setcap_cap_sys_nice_eip(&xrservice_binary).await {
                        setcap_failed_dialog();
                        error!("failed running setcap: {e}");
                        self.build_window
                            .sender()
                            .emit(BuildWindowMsg::UpdateContent(vec![
                                TermColor::Red.colorize("Setting capabilities failed\n"),
                            ]));
                    } else if !verify_cap_sys_nice_eip(&xrservice_binary)
                        .await
                        .unwrap_or(false)
                    {
                        setcap_failed_dialog();
                        error!("setcap succeeded but capabilities were reset");
                        self.build_window
                            .sender()
                            .emit(BuildWindowMsg::UpdateContent(vec![TermColor::Red
                            .colorize(
                            "Setting capabilities succeeded, but capabilities have been reset\n",
                        )]));
                    } else {
                        self.build_window
                            .sender()
                            .emit(BuildWindowMsg::UpdateContent(vec![
                                TermColor::Green.colorize("Capabilities set correctly\n"),
                            ]));
                    }
                }
            }
            Msg::ProfileSelected(prof) => {
                if !prof.is_system_profile {
                    self.config.selected_profile_uuid = prof.uuid.clone();
                    self.config.save();
                }
                self.main_view
                    .sender()
                    .emit(MainViewMsg::UpdateSelectedProfile(prof.clone()));
                self.debug_view
                    .sender()
                    .emit(DebugViewMsg::UpdateSelectedProfile(prof.clone()));
            }
            Msg::OpenLibsurviveSetup => {
                let prof = self.get_selected_profile();
                self.libsurvive_setup_window
                    .sender()
                    .send(LibsurviveSetupMsg::Present(prof))
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
            Msg::HandleCommandLine(opts) => {
                if let Some(prof_uuid) = opts.profile_uuid
                    && let Some(index) = self.profiles.iter().position(|p| p.uuid == prof_uuid)
                {
                    let target = self.profiles.get(index).unwrap();
                    sender.input(Msg::ProfileSelected(target.clone()));
                    self.debug_view
                        .sender()
                        .emit(DebugViewMsg::SetSelectedProfile(index as u32));
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
            Msg::OpenPreferences => {
                self.config.prune_xr_plugins();
                let preferences_win = PreferencesWindow::builder()
                    .launch(PreferencesWindowInit {
                        config: self.config.clone(),
                        root_win: self.app_win.clone().upcast(),
                    })
                    .forward(sender.input_sender(), move |msg| match msg {
                        PreferencesWindowOutMsg::OnClosed(config) => {
                            Msg::OnPreferencesWindowClose(config)
                        }
                    });
                preferences_win.sender().emit(PreferencesWindowMsg::Present);
                self.preferences_win = Some(preferences_win);
            }
            Msg::OnPreferencesWindowClose(config) => {
                self.config = config;
                self.config.save();
                sender.input(Msg::ProfileSelected(self.get_selected_profile()));
            }
            Msg::LaunchMonadoGui => {
                if self.monado_gui_worker.is_some() {
                    return;
                }
                let profile = self.get_selected_profile();
                if let Some(bin) = profile.monado_gui_binary() {
                    let mut jobs = VecDeque::new();
                    jobs.push_back(WorkerJob::new_cmd(
                        Some(profile.environment.clone()),
                        bin.to_string_lossy().to_string(),
                        None,
                    ));
                    let worker = JobWorker::new(jobs, sender.input_sender(), |msg| match msg {
                        JobWorkerOut::Log(rows) => Msg::OnServiceLog(rows),
                        JobWorkerOut::Exit(_) => Msg::OnMonadoGuiExit,
                    });
                    worker.start();
                    self.monado_gui_worker = Some(worker);
                }
            }
            Msg::OnMonadoGuiExit => {
                self.monado_gui_worker = None;
            }
            Msg::ProfilesEnabledChanged(enabled) => {
                self.build_profile_action.set_enabled(enabled);
                self.clean_build_profile_action.set_enabled(enabled);
                self.config.profiles_enabled = enabled;
                self.config.save();
                if enabled {
                    self.debug_view
                        .sender()
                        .emit(DebugViewMsg::SetSelectedProfile(
                            self.config
                                .profiles()
                                .iter()
                                .position(|p| p.uuid == self.config.selected_profile_uuid)
                                .unwrap_or_default() as u32,
                        ));
                }
                sender.input(Msg::ProfileSelected(self.get_selected_profile()));
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
            PreferencesAction,
            clone!(
                #[strong]
                sender,
                move |_| {
                    sender.input(Msg::OpenPreferences);
                }
            )
        );
        stateless_action!(
            actions,
            ThemeManagerAction,
            clone!(
                #[strong]
                sender,
                move |_| {
                    sender.input(Msg::ShowThemeManager);
                }
            )
        );
        // this bypasses the macro because I need the underlying gio action
        // to enable/disable it in update()
        let build_profile_action = {
            let action = RelmAction::<BuildProfileAction>::new_stateless(clone!(
                #[strong]
                sender,
                move |_| sender.input(Msg::BuildProfile(false))
            ));
            let ret = action.gio_action().clone();
            actions.add_action(action);
            ret.set_enabled(config.profiles_enabled);
            ret
        };
        let clean_build_profile_action = {
            let action = RelmAction::<BuildProfileCleanAction>::new_stateless(clone!(
                #[strong]
                sender,
                move |_| sender.input(Msg::BuildProfile(true))
            ));
            let ret = action.gio_action().clone();
            actions.add_action(action);
            ret.set_enabled(config.profiles_enabled);
            ret
        };

        let selected_profile = if config.profiles_enabled {
            config.get_selected_profile(&profiles)
        } else {
            system_profile(&config)
        };

        let vkinfo = {
            match VulkanInfo::get() {
                Ok(info) => Some(info),
                Err(e) => {
                    error!("failed to get Vulkan info: {e:#?}");
                    None
                }
            }
        };

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
                    MainViewOutMsg::OpenLibsurviveSetup => Msg::OpenLibsurviveSetup,
                    MainViewOutMsg::LaunchMonadoGui => Msg::LaunchMonadoGui,
                }),
            vkinfo,
            debug_view: DebugView::builder()
                .launch(DebugViewInit {
                    profile: selected_profile,
                    root_win: root.clone().into(),
                    profiles_enabled: config.profiles_enabled,
                })
                .forward(sender.input_sender(), |message| match message {
                    DebugViewOutMsg::StartWithDebug => Msg::StartWithDebug,
                    DebugViewOutMsg::ProfileSelected(prof) => Msg::ProfileSelected(prof),
                    DebugViewOutMsg::DeleteProfile(delete_files) => {
                        Msg::DeleteProfile(delete_files)
                    }
                    DebugViewOutMsg::SaveProfile(p) => Msg::SaveProfile(p),
                    DebugViewOutMsg::BuildProfile(clean) => Msg::BuildProfile(clean),
                    DebugViewOutMsg::ProfilesEnabledChanged(enabled) => {
                        Msg::ProfilesEnabledChanged(enabled)
                    }
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
            theme_engine: ThemeEngine::new_with_theme(&{
                if config.theme_name == "Follow system" {
                    Theme::default()
                } else {
                    Theme::default_themes()
                        .into_iter()
                        .find(|t| t.name == config.theme_name)
                        .unwrap_or_default()
                }
            })
            .unwrap(),
            config,
            profiles,
            xrservice_worker: None,
            plugins_worker: None,
            build_worker: None,
            monado_gui_worker: None,
            restart_xrservice: false,
            libmonado: None,
            skip_depcheck: false,
            build_profile_action,
            clean_build_profile_action,
            openxr_prober_worker: None,
            xrservice_ready: false,
            inhibit_fail_notif: None,
            preferences_win: None,
            build_alerts: VecDeque::new(),
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
                #[strong(rename_to = vkinfo)]
                model.vkinfo,
                move |_| {
                    populate_debug_info(&about_dialog, vkinfo.as_ref());
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
            app.set_accelerators_for_action::<PreferencesAction>(&["<Control>comma"]);
            app.set_accelerators_for_action::<BuildProfileCleanAction>(&["<Control>F5"]);
            app.set_accelerators_for_action::<BuildProfileAction>(&["F5"]);
        }

        model.debug_view.sender().emit(DebugViewMsg::UpdateProfiles(
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
new_stateless_action!(pub PreferencesAction, AppActionGroup, "preferences");
new_stateless_action!(pub ThemeManagerAction, AppActionGroup, "thememanager");

new_stateless_action!(pub DebugOpenDataAction, AppActionGroup, "debugopendata");
new_stateless_action!(pub DebugOpenPrefixAction, AppActionGroup, "debugopenprefix");
