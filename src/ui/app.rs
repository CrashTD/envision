use super::about_dialog::AboutDialog;
use super::alert::alert;
use super::build_window::{BuildStatus, BuildWindow};
use super::debug_view::{DebugView, DebugViewMsg};
use super::fbt_config_editor::{FbtConfigEditor, FbtConfigEditorInit, FbtConfigEditorMsg};
use super::job_worker::internal_worker::JobWorkerOut;
use super::job_worker::job::WorkerJob;
use super::job_worker::JobWorker;
use super::libsurvive_setup_window::LibsurviveSetupWindow;
use super::main_view::MainViewMsg;
use super::util::open_with_default_handler;
use crate::builders::build_basalt::get_build_basalt_jobs;
use crate::builders::build_libsurvive::get_build_libsurvive_jobs;
use crate::builders::build_mercury::get_build_mercury_job;
use crate::builders::build_monado::get_build_monado_jobs;
use crate::builders::build_opencomposite::get_build_opencomposite_jobs;
use crate::builders::build_openhmd::get_build_openhmd_jobs;
use crate::builders::build_wivrn::get_build_wivrn_jobs;
use crate::config::Config;
use crate::constants::APP_NAME;
use crate::depcheck::check_dependency;
use crate::dependencies::basalt_deps::get_missing_basalt_deps;
use crate::dependencies::libsurvive_deps::get_missing_libsurvive_deps;
use crate::dependencies::mercury_deps::get_missing_mercury_deps;
use crate::dependencies::monado_deps::get_missing_monado_deps;
use crate::dependencies::openhmd_deps::get_missing_openhmd_deps;
use crate::dependencies::pkexec_dep::pkexec_dep;
use crate::dependencies::wivrn_deps::get_missing_wivrn_deps;
use crate::file_builders::active_runtime_json::{
    set_current_active_runtime_to_profile, set_current_active_runtime_to_steam,
};
use crate::file_builders::openvrpaths_vrpath::{
    set_current_openvrpaths_to_profile, set_current_openvrpaths_to_steam,
};
use crate::file_utils::setcap_cap_sys_nice_eip;
use crate::linux_distro::get_distro;
use crate::log_parser::MonadoLog;
use crate::paths::{get_data_dir, get_ipc_file_path};
use crate::profile::{Profile, XRServiceType};
use crate::profiles::lighthouse::lighthouse_profile;
use crate::profiles::openhmd::openhmd_profile;
use crate::profiles::survive::survive_profile;
use crate::profiles::wivrn::wivrn_profile;
use crate::profiles::wmr::wmr_profile;
use crate::stateless_action;
use crate::ui::build_window::{BuildWindowMsg, BuildWindowOutMsg};
use crate::ui::debug_view::DebugViewInit;
use crate::ui::libsurvive_setup_window::LibsurviveSetupMsg;
use crate::ui::main_view::{MainView, MainViewInit, MainViewOutMsg};
use crate::xr_devices::XRDevice;
use gtk::glib::clone;
use gtk::prelude::*;
use relm4::actions::{AccelsPlus, ActionGroupName, RelmAction, RelmActionGroup};
use relm4::adw::traits::MessageDialogExt;
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
    restart_xrservice: bool,
    #[tracker::do_not_track]
    build_worker: Option<JobWorker>,
    #[tracker::do_not_track]
    profiles: Vec<Profile>,
    #[tracker::do_not_track]
    xr_devices: Vec<XRDevice>,
    #[tracker::do_not_track]
    fbt_config_editor: Option<Controller<FbtConfigEditor>>,
    #[tracker::do_not_track]
    libmonado: Option<libmonado_rs::Monado>,
}

#[derive(Debug)]
pub enum Msg {
    OnServiceLog(Vec<String>),
    OnServiceExit(i32),
    OnBuildLog(Vec<String>),
    OnBuildExit(i32),
    ClockTicking,
    BuildProfile(bool, bool),
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
    ParseLog(Vec<String>),
    ConfigFbt,
    DebugOpenPrefix,
    DebugOpenData,
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
        if set_current_active_runtime_to_profile(&prof).is_err() {
            alert(
                "Failed to start XR Service",
                Some("Error setting current active runtime to profile"),
                Some(&self.app_win.clone().upcast::<gtk::Window>()),
            );
            return;
        }
        if set_current_openvrpaths_to_profile(&prof).is_err() {
            alert(
                "Failed to start XR Service",
                Some("Error setting current openvrpaths file to profile"),
                Some(&self.app_win.clone().upcast::<gtk::Window>()),
            );
            return;
        };
        self.debug_view.sender().emit(DebugViewMsg::ClearLog);
        self.xr_devices = vec![];
        if prof.can_start() {
            remove_file(&get_ipc_file_path(&prof.xrservice_type))
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
                ));
            self.debug_view
                .sender()
                .emit(DebugViewMsg::XRServiceActiveChanged(true));
            self.set_inhibit_session(true);
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

    pub fn restore_openxr_openvr_files(&self) {
        if set_current_active_runtime_to_steam().is_err() {
            alert(
                "Could not restore Steam active runtime",
                None,
                Some(&self.app_win.clone().upcast::<gtk::Window>()),
            );
        }
        if set_current_openvrpaths_to_steam().is_err() {
            alert(
                "Could not restore Steam openvrpaths",
                None,
                Some(&self.app_win.clone().upcast::<gtk::Window>()),
            );
        };
    }

    pub fn shutdown_xrservice(&mut self) {
        self.set_inhibit_session(false);
        if let Some(worker) = self.xrservice_worker.as_ref() {
            worker.stop();
        }
        self.libmonado = None;
        self.restore_openxr_openvr_files();
        self.main_view
            .sender()
            .emit(MainViewMsg::XRServiceActiveChanged(false, None));
        self.debug_view
            .sender()
            .emit(DebugViewMsg::XRServiceActiveChanged(false));
        self.xr_devices = vec![];
    }

    pub fn profiles_list(config: &Config) -> Vec<Profile> {
        let mut profiles = vec![
            lighthouse_profile(),
            survive_profile(),
            wivrn_profile(),
            wmr_profile(),
            openhmd_profile(),
        ];
        profiles.extend(config.user_profiles.clone());
        profiles.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        profiles
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
            Msg::OnServiceLog(rows) => {
                if !rows.is_empty() {
                    sender.input(Msg::ParseLog(rows.clone()));
                    self.debug_view
                        .sender()
                        .emit(DebugViewMsg::LogUpdated(rows));
                }
            }
            Msg::OnServiceExit(_) => {
                self.main_view
                    .sender()
                    .emit(MainViewMsg::XRServiceActiveChanged(false, None));
                self.debug_view
                    .sender()
                    .emit(DebugViewMsg::XRServiceActiveChanged(false));
                if let Some(worker) = self.xrservice_worker.as_ref() {
                    if let Some(code) = worker.exit_code() {
                        if code != 0 && code != 15 {
                            // 15 is SIGTERM
                            sender.input(Msg::OnServiceLog(vec![format!(
                                "{} exited with code {}",
                                self.get_selected_profile().xrservice_type,
                                code
                            )]));
                        }
                    }
                }
                self.xrservice_worker = None;
                if self.restart_xrservice {
                    self.restart_xrservice = false;
                    self.start_xrservice(sender, false);
                }
            }
            Msg::ClockTicking => {
                self.main_view.sender().emit(MainViewMsg::ClockTicking);
                if let Some(w) = self.xrservice_worker.as_ref() {
                    if {
                        let state = w.state.lock().unwrap();
                        state.exit_status.is_none() && !state.stop_requested
                    } {
                        if let Some(monado) = self.libmonado.as_ref() {
                            self.xr_devices = XRDevice::merge(
                                &self.xr_devices,
                                &XRDevice::from_libmonado(monado),
                            );
                            self.main_view
                                .sender()
                                .emit(MainViewMsg::UpdateDevices(self.xr_devices.clone()));
                        } else {
                            if let Some(so) = self.get_selected_profile().libmonado_so() {
                                self.libmonado = libmonado_rs::Monado::create(so).ok();
                                if self.libmonado.is_some() {
                                    sender.input(Msg::ClockTicking);
                                }
                            }
                        }
                    }
                }
            }
            Msg::ParseLog(rows) => {
                for row in rows {
                    match MonadoLog::new_from_str(row.as_str()) {
                        None => {}
                        Some(parsed) => {
                            if let Some(tracker) =
                                XRDevice::generic_tracker_from_log_row(parsed.message.as_str())
                            {
                                self.xr_devices.push(tracker);
                            }
                        }
                    };
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
                    let status = worker.state.lock().unwrap().exit_status.clone();
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
            Msg::BuildProfile(clean_build, debug_build) => {
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
                        jobs.push_back(get_build_mercury_job(&profile));
                    }
                    jobs.extend(match profile.xrservice_type {
                        XRServiceType::Monado => {
                            get_build_monado_jobs(&profile, clean_build, debug_build)
                        }
                        XRServiceType::Wivrn => {
                            get_build_wivrn_jobs(&profile, clean_build, debug_build)
                        }
                    });
                    // no listed deps for opencomp
                }
                jobs.extend(get_build_opencomposite_jobs(
                    &profile,
                    clean_build,
                    debug_build,
                ));
                if !missing_deps.is_empty() {
                    missing_deps.sort_unstable();
                    missing_deps.dedup(); // dedup only works if sorted, hence the above
                    let distro = get_distro();
                    alert(
                        "Missing dependencies:",
                        Some(
                            missing_deps
                                .iter()
                                .map(|dep| {
                                    if let Some(d) = distro {
                                        return dep
                                            .packages
                                            .get(&d)
                                            .unwrap_or_else(|| &dep.name)
                                            .clone();
                                    }
                                    dep.name.clone()
                                })
                                .collect::<Vec<String>>()
                                .join(", ")
                                .as_str(),
                        ),
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
                    self.profiles = Self::profiles_list(&self.config);
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
                if !check_dependency(pkexec_dep()) {
                    println!("pkexec not found, skipping setcap");
                } else {
                    let profile = self.get_selected_profile();
                    setcap_cap_sys_nice_eip(format!(
                        "{pfx}/bin/monado-service",
                        pfx = profile.prefix
                    ));
                }
            }
            Msg::ProfileSelected(prof) => {
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
            Msg::ConfigFbt => {
                self.fbt_config_editor = Some(
                    FbtConfigEditor::builder()
                        .launch(FbtConfigEditorInit {
                            root_win: self.app_win.clone().upcast::<gtk::Window>(),
                        })
                        .detach(),
                );
                self.fbt_config_editor
                    .as_ref()
                    .unwrap()
                    .emit(FbtConfigEditorMsg::Present);
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
                open_with_default_handler(&format!("file://{}", get_data_dir()));
            }
            Msg::DebugOpenPrefix => {
                open_with_default_handler(&format!(
                    "file://{}",
                    self.get_selected_profile().prefix
                ));
            }
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let config = Config::get_config();
        let win_size = config.win_size;
        let profiles = Self::profiles_list(&config);
        let setcap_confirm_dialog = adw::MessageDialog::builder()
            .modal(true)
            .transient_for(root)
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

        let mut model = App {
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
                    MainViewOutMsg::EnableDebugViewChanged(val) => Msg::EnableDebugViewChanged(val),
                    MainViewOutMsg::DoStartStopXRService => Msg::DoStartStopXRService,
                    MainViewOutMsg::StartWithDebug => Msg::StartWithDebug,
                    MainViewOutMsg::RestartXRService => Msg::RestartXRService,
                    MainViewOutMsg::ProfileSelected(uuid) => Msg::ProfileSelected(uuid),
                    MainViewOutMsg::DeleteProfile => Msg::DeleteProfile,
                    MainViewOutMsg::SaveProfile(p) => Msg::SaveProfile(p),
                    MainViewOutMsg::OpenLibsurviveSetup => Msg::OpenLibsurviveSetup,
                }),
            debug_view: DebugView::builder().launch(DebugViewInit {}).forward(
                sender.input_sender(),
                |message| match message {
                    _ => Msg::ClockTicking,
                },
            ),
            about_dialog: AboutDialog::builder()
                .transient_for(root)
                .launch(())
                .detach(),
            build_window: BuildWindow::builder()
                .transient_for(root)
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    BuildWindowOutMsg::CancelBuild => Msg::CancelBuild,
                }),
            libsurvive_setup_window: LibsurviveSetupWindow::builder()
                .transient_for(root)
                .launch(())
                .detach(),
            split_view: None,
            setcap_confirm_dialog,
            enable_debug_view: config.debug_view_enabled,
            config,
            tracker: 0,
            profiles,
            xrservice_worker: None,
            build_worker: None,
            xr_devices: vec![],
            fbt_config_editor: None,
            restart_xrservice: false,
            libmonado: None,
        };
        let widgets = view_output!();

        let mut actions = RelmActionGroup::<AppActionGroup>::new();

        stateless_action!(
            actions,
            BuildProfileAction,
            clone!(@strong sender => move |_| {
                sender.input_sender().emit(Msg::BuildProfile(false, false));
            })
        );
        stateless_action!(
            actions,
            BuildProfileCleanAction,
            clone!(@strong sender => move |_| {
                sender.input_sender().emit(Msg::BuildProfile(true, false));
            })
        );
        stateless_action!(
            actions,
            BuildProfileDebugAction,
            clone!(@strong sender => move |_| {
                sender.input_sender().emit(Msg::BuildProfile(false, true));
            })
        );
        stateless_action!(
            actions,
            BuildProfileCleanDebugAction,
            clone!(@strong sender => move |_| {
                sender.input_sender().emit(Msg::BuildProfile(true, true));
            })
        );
        stateless_action!(
            actions,
            ConfigFbtAction,
            clone!(@strong sender => move |_| {
                sender.input_sender().emit(Msg::ConfigFbt);
            })
        );
        {
            let abd_sender = model.about_dialog.sender().clone();
            stateless_action!(actions, AboutAction, move |_| {
                abd_sender.send(()).unwrap();
            });
        }
        stateless_action!(
            actions,
            QuitAction,
            clone!(@strong sender => move |_| {
                sender.input(Msg::Quit);
            })
        );
        stateless_action!(
            actions,
            DebugOpenDataAction,
            clone!(@strong sender => move |_| {
                sender.input(Msg::DebugOpenData);
            })
        );
        stateless_action!(
            actions,
            DebugOpenPrefixAction,
            clone!(@strong sender => move |_| {
                sender.input(Msg::DebugOpenPrefix);
            })
        );
        actions.add_action(RelmAction::<DebugViewToggleAction>::new_stateful(
            &model.enable_debug_view,
            clone!(@strong sender => move |_, state| {
                let s = *state;
                *state = !s;
                sender.input(Msg::EnableDebugViewChanged(*state));
            }),
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
            clone!(@strong sender => move || {
                sender.input(Msg::ClockTicking);
                glib::ControlFlow::Continue
            }),
        );

        model.split_view = Some(widgets.split_view.clone());

        ComponentParts { model, widgets }
    }
}

new_action_group!(pub AppActionGroup, "win");
new_stateless_action!(pub AboutAction, AppActionGroup, "about");
new_stateless_action!(pub BuildProfileAction, AppActionGroup, "buildprofile");
new_stateless_action!(pub BuildProfileCleanAction, AppActionGroup, "buildprofileclean");
new_stateless_action!(pub BuildProfileDebugAction, AppActionGroup, "buildprofiledebug");
new_stateless_action!(pub BuildProfileCleanDebugAction, AppActionGroup, "buildprofilecleandebug");
new_stateless_action!(pub ConfigFbtAction, AppActionGroup, "configfbt");
new_stateless_action!(pub QuitAction, AppActionGroup, "quit");
new_stateful_action!(pub DebugViewToggleAction, AppActionGroup, "debugviewtoggle", (), bool);

new_stateless_action!(pub DebugOpenDataAction, AppActionGroup, "debugopendata");
new_stateless_action!(pub DebugOpenPrefixAction, AppActionGroup, "debugopenprefix");
