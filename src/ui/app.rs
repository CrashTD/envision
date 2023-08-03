use super::about_dialog::AboutDialog;
use super::alert::alert;
use super::build_window::{BuildStatus, BuildWindow};
use super::debug_view::{DebugView, DebugViewMsg};
use super::libsurvive_setup_window::LibsurviveSetupWindow;
use super::main_view::MainViewMsg;
use crate::builders::build_basalt::get_build_basalt_runners;
use crate::builders::build_libsurvive::get_build_libsurvive_runners;
use crate::builders::build_mercury::get_build_mercury_runner;
use crate::builders::build_monado::get_build_monado_runners;
use crate::builders::build_opencomposite::get_build_opencomposite_runners;
use crate::builders::build_wivrn::get_build_wivrn_runners;
use crate::config::Config;
use crate::constants::APP_NAME;
use crate::depcheck::check_dependency;
use crate::dependencies::basalt_deps::get_missing_basalt_deps;
use crate::dependencies::libsurvive_deps::get_missing_libsurvive_deps;
use crate::dependencies::mercury_deps::get_missing_mercury_deps;
use crate::dependencies::monado_deps::get_missing_monado_deps;
use crate::dependencies::pkexec_dep::pkexec_dep;
use crate::dependencies::wivrn_deps::get_missing_wivrn_deps;
use crate::file_builders::active_runtime_json::{
    set_current_active_runtime_to_profile, set_current_active_runtime_to_steam,
};
use crate::file_builders::openvrpaths_vrpath::{
    set_current_openvrpaths_to_profile, set_current_openvrpaths_to_steam,
};
use crate::file_utils::setcap_cap_sys_nice_eip;
use crate::log_parser::MonadoLog;
use crate::profile::{Profile, XRServiceType};
use crate::profiles::system_valve_index::system_valve_index_profile;
use crate::profiles::valve_index::valve_index_profile;
use crate::profiles::wivrn::wivrn_profile;
use crate::runner::{Runner, RunnerStatus};
use crate::runner_pipeline::RunnerPipeline;
use crate::ui::build_window::BuildWindowMsg;
use crate::ui::debug_view::DebugViewInit;
use crate::ui::libsurvive_setup_window::LibsurviveSetupMsg;
use crate::ui::main_view::{MainView, MainViewInit, MainViewOutMsg};
use crate::xr_devices::XRDevices;
use crate::{stateless_action, withclones};
use gtk::prelude::*;
use relm4::actions::{AccelsPlus, ActionGroupName, RelmAction, RelmActionGroup};
use relm4::adw::traits::MessageDialogExt;
use relm4::adw::ResponseAppearance;
use relm4::gtk::glib;
use relm4::{new_action_group, new_stateful_action, new_stateless_action, prelude::*};
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
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
    xrservice_runner: Option<Runner>,
    #[tracker::do_not_track]
    build_pipeline: Option<RunnerPipeline>,
    #[tracker::do_not_track]
    profiles: Vec<Profile>,
    #[tracker::do_not_track]
    devices_processed: bool,
}

#[derive(Debug)]
pub enum Msg {
    ClockTicking,
    BuildProfile(bool),
    EnableDebugViewChanged(bool),
    DoStartStopXRService,
    RestartXRService,
    ProfileSelected(Profile),
    DeleteProfile,
    SaveProfile(Profile),
    RunSetCap,
    OpenLibsurviveSetup,
    SaveWinSize(i32, i32),
    Quit,
    ProcessDevicesLog(Vec<String>),
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

    pub fn start_xrservice(&mut self) {
        self.set_inhibit_session(true);
        let prof = self.get_selected_profile();
        set_current_active_runtime_to_profile(&prof);
        set_current_openvrpaths_to_profile(&prof);
        self.devices_processed = match prof.xrservice_type {
            XRServiceType::Monado => false,
            XRServiceType::Wivrn => true, // no device from log in wivrn
        };
        self.debug_view.sender().emit(DebugViewMsg::ClearLog);
        let mut runner = Runner::xrservice_runner_from_profile(&prof);
        match runner.try_start() {
            Ok(_) => {
                self.xrservice_runner = Some(runner);
                self.main_view
                    .sender()
                    .emit(MainViewMsg::XRServiceActiveChanged(
                        true,
                        Some(self.get_selected_profile()),
                    ));
            }
            Err(_) => {
                alert(
                    "Failed to start profile",
                    Some(concat!(
                        "You need to build the current profile before starting it.",
                        "\n\nYou can do this from the menu."
                    )),
                    Some(&self.app_win.clone().upcast::<gtk::Window>()),
                );
            }
        };
    }

    pub fn shutdown_xrservice(&mut self) {
        if self.xrservice_runner.is_some() {
            if self.xrservice_runner.as_mut().unwrap().status() == RunnerStatus::Running {
                self.xrservice_runner.as_mut().unwrap().terminate();
            }
        }
        set_current_active_runtime_to_steam();
        set_current_openvrpaths_to_steam();
        self.set_inhibit_session(false);
        self.main_view
            .sender()
            .emit(MainViewMsg::XRServiceActiveChanged(false, None));
    }

    pub fn profiles_list(config: &Config) -> Vec<Profile> {
        let mut profiles = vec![
            valve_index_profile(),
            system_valve_index_profile(),
            wivrn_profile(),
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
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: true,
                set_vexpand: true,
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,
                    set_vexpand: true,
                    model.main_view.widget(),
                    gtk::Separator {
                        set_orientation: gtk::Orientation::Vertical,
                        #[track = "model.changed(App::enable_debug_view())"]
                        set_visible: model.enable_debug_view,
                    },
                    model.debug_view.widget(),
                }
            },
            connect_close_request[sender] => move |win| {
                sender.input(Msg::SaveWinSize(win.width(), win.height()));
                gtk::Inhibit(false)
            }
        }
    }

    fn shutdown(&mut self, widgets: &mut Self::Widgets, output: relm4::Sender<Self::Output>) {
        if self.xrservice_runner.is_some() {
            if self.xrservice_runner.as_mut().unwrap().status() == RunnerStatus::Running {
                self.xrservice_runner.as_mut().unwrap().terminate();
            }
        }
        set_current_active_runtime_to_steam();
        set_current_openvrpaths_to_steam();
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        self.reset();

        match message {
            Msg::ClockTicking => {
                match &mut self.xrservice_runner {
                    None => {}
                    Some(runner) => {
                        let n_rows = runner.consume_rows();
                        if !n_rows.is_empty() {
                            if !self.devices_processed {
                                sender.input(Msg::ProcessDevicesLog(n_rows.clone()))
                            }
                            self.debug_view
                                .sender()
                                .emit(DebugViewMsg::LogUpdated(n_rows));
                        }
                        match runner.status() {
                            RunnerStatus::Running => {}
                            RunnerStatus::Stopped(_) => {
                                self.main_view
                                    .sender()
                                    .emit(MainViewMsg::XRServiceActiveChanged(false, None));
                            }
                        };
                    }
                };
                match &mut self.build_pipeline {
                    None => {}
                    Some(pipeline) => {
                        pipeline.update();
                        self.build_window
                            .sender()
                            .emit(BuildWindowMsg::UpdateContent(pipeline.get_log()));
                        match pipeline.status() {
                            RunnerStatus::Running | RunnerStatus::Stopped(None) => {}
                            RunnerStatus::Stopped(Some(code)) => {
                                self.build_window
                                    .sender()
                                    .emit(BuildWindowMsg::UpdateCanClose(true));
                                self.build_pipeline.take();
                                match code {
                                    0 => {
                                        self.build_window.sender().emit(
                                            BuildWindowMsg::UpdateBuildStatus(BuildStatus::Done),
                                        );
                                        // apparently setcap on wivrn causes issues, so in case
                                        // it's not monado, we're just skipping this
                                        if self.get_selected_profile().xrservice_type
                                            == XRServiceType::Monado
                                        {
                                            self.setcap_confirm_dialog.present();
                                        }
                                        self.build_window
                                            .sender()
                                            .emit(BuildWindowMsg::UpdateCanClose(true));
                                    }
                                    errcode => self.build_window.sender().emit(
                                        BuildWindowMsg::UpdateBuildStatus(BuildStatus::Error(
                                            format!("Exit status {}", errcode),
                                        )),
                                    ),
                                }
                            }
                        }
                    }
                };
                self.main_view.sender().emit(MainViewMsg::ClockTicking);
            }
            Msg::ProcessDevicesLog(rows) => {
                for row in rows {
                    match MonadoLog::from_str(row.as_str()) {
                        None => {}
                        Some(parsed) => {
                            if parsed.func == "p_create_system" {
                                match XRDevices::from_log_message(parsed.message) {
                                    None => {}
                                    Some(devices) => {
                                        self.devices_processed = true;
                                        self.main_view
                                            .sender()
                                            .emit(MainViewMsg::UpdateDevices(Some(devices)));
                                        break;
                                    }
                                };
                            }
                        }
                    };
                }
            }
            Msg::EnableDebugViewChanged(val) => {
                self.set_enable_debug_view(val);
                self.config.debug_view_enabled = val;
                self.config.save();
                self.debug_view
                    .sender()
                    .emit(DebugViewMsg::EnableDebugViewChanged(val));
                self.main_view
                    .sender()
                    .emit(MainViewMsg::EnableDebugViewChanged(val));
            }
            Msg::DoStartStopXRService => match &mut self.xrservice_runner {
                None => {
                    self.start_xrservice();
                }
                Some(runner) => match runner.status() {
                    RunnerStatus::Running => {
                        self.shutdown_xrservice();
                    }
                    RunnerStatus::Stopped(_) => {
                        self.start_xrservice();
                    }
                },
            },
            Msg::RestartXRService => {
                match &mut self.xrservice_runner {
                    None => {}
                    Some(runner) => match runner.status() {
                        RunnerStatus::Stopped(_) => {}
                        RunnerStatus::Running => {
                            if self.xrservice_runner.is_some() {
                                if self.xrservice_runner.as_mut().unwrap().status()
                                    == RunnerStatus::Running
                                {
                                    self.xrservice_runner.as_mut().unwrap().terminate();
                                }
                            }
                        }
                    },
                }
                self.start_xrservice();
            }
            Msg::BuildProfile(clean_build) => {
                let profile = self.get_selected_profile();
                let mut missing_deps = vec![];
                let mut runners: Vec<Runner> = vec![];
                // profile per se can't be built, but we still need opencomp
                if profile.can_be_built {
                    missing_deps.extend(match profile.xrservice_type {
                        XRServiceType::Monado => get_missing_monado_deps(),
                        XRServiceType::Wivrn => get_missing_wivrn_deps(),
                    });
                    if profile.features.libsurvive.enabled {
                        missing_deps.extend(get_missing_libsurvive_deps());
                        runners.extend(get_build_libsurvive_runners(&profile, clean_build));
                    }
                    if profile.features.basalt.enabled {
                        missing_deps.extend(get_missing_basalt_deps());
                        runners.extend(get_build_basalt_runners(&profile, clean_build));
                    }
                    if profile.features.mercury_enabled {
                        missing_deps.extend(get_missing_mercury_deps());
                        runners.push(get_build_mercury_runner(&profile));
                    }
                    runners.extend(match profile.xrservice_type {
                        XRServiceType::Monado => get_build_monado_runners(&profile, clean_build),
                        XRServiceType::Wivrn => get_build_wivrn_runners(&profile, clean_build),
                    });
                    // no listed deps for opencomp
                }
                runners.extend(get_build_opencomposite_runners(&profile, clean_build));
                if !missing_deps.is_empty() {
                    missing_deps.sort_unstable();
                    missing_deps.dedup(); // dedup only works if sorted, hence the above
                    alert(
                        "Missing dependencies:",
                        Some(
                            missing_deps
                                .iter()
                                .map(|dep| dep.name.clone())
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
                let mut pipeline = RunnerPipeline::new(runners);
                pipeline.start();
                self.build_window
                    .sender()
                    .emit(BuildWindowMsg::UpdateTitle(format!(
                        "Building Profile {}",
                        profile.name
                    )));
                self.build_window
                    .sender()
                    .emit(BuildWindowMsg::UpdateCanClose(false));
                self.build_pipeline = Some(pipeline);
            }
            Msg::DeleteProfile => {
                let todel = self.get_selected_profile();
                if todel.editable {
                    self.config.user_profiles = self
                        .config
                        .user_profiles
                        .iter()
                        .filter(|p| p.uuid != todel.uuid)
                        .map(|p| p.clone())
                        .collect();
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
                self.profiles.push(prof);
                self.profiles.sort_unstable_by(|a, b| a.name.cmp(&b.name));
                self.config.set_profiles(&self.profiles);
                self.config.save();
                self.main_view.sender().emit(MainViewMsg::UpdateProfiles(
                    self.profiles.clone(),
                    self.config.clone(),
                ))
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
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let config = Config::get_config();
        let win_size = config.win_size.clone();
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

        let model = App {
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
                    MainViewOutMsg::RestartXRService => Msg::RestartXRService,
                    MainViewOutMsg::ProfileSelected(uuid) => Msg::ProfileSelected(uuid),
                    MainViewOutMsg::DeleteProfile => Msg::DeleteProfile,
                    MainViewOutMsg::SaveProfile(p) => Msg::SaveProfile(p),
                }),
            debug_view: DebugView::builder()
                .launch(DebugViewInit {
                    enable_debug_view: config.debug_view_enabled,
                })
                .forward(sender.input_sender(), |message| match message {
                    _ => Msg::ClockTicking,
                }),
            about_dialog: AboutDialog::builder()
                .transient_for(root)
                .launch(())
                .detach(),
            build_window: BuildWindow::builder()
                .transient_for(root)
                .launch(())
                .detach(),
            libsurvive_setup_window: LibsurviveSetupWindow::builder()
                .transient_for(root)
                .launch(())
                .detach(),
            setcap_confirm_dialog,
            enable_debug_view: config.debug_view_enabled,
            config,
            tracker: 0,
            profiles,
            xrservice_runner: None,
            build_pipeline: None,
            devices_processed: false,
        };
        let widgets = view_output!();

        let mut actions = RelmActionGroup::<AppActionGroup>::new();

        {
            withclones![sender];
            stateless_action!(actions, BuildProfileAction, {
                sender.input_sender().emit(Msg::BuildProfile(false));
            });
        }
        {
            withclones![sender];
            stateless_action!(actions, BuildProfileCleanAction, {
                sender.input_sender().emit(Msg::BuildProfile(true));
            });
        }
        {
            let abd_sender = model.about_dialog.sender().clone();
            stateless_action!(actions, AboutAction, {
                abd_sender.send(()).unwrap();
            });
        }
        {
            withclones![sender];
            stateless_action!(actions, LibsurviveSetupAction, {
                sender.input(Msg::OpenLibsurviveSetup);
            });
        }
        {
            withclones![sender];
            stateless_action!(actions, QuitAction, {
                sender.input(Msg::Quit);
            });
        }
        {
            withclones![sender];
            actions.add_action(RelmAction::<DebugViewToggleAction>::new_stateful(
                &model.enable_debug_view,
                move |_, state| {
                    let s = *state;
                    *state = !s;
                    sender.input(Msg::EnableDebugViewChanged(*state));
                },
            ))
        }

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

        {
            withclones![sender];
            glib::timeout_add_local(Duration::from_millis(1000), move || {
                sender.input(Msg::ClockTicking);
                return glib::Continue(true);
            });
        }

        ComponentParts { model, widgets }
    }
}

new_action_group!(pub AppActionGroup, "win");
new_stateless_action!(pub AboutAction, AppActionGroup, "about");
new_stateless_action!(pub BuildProfileAction, AppActionGroup, "buildprofile");
new_stateless_action!(pub BuildProfileCleanAction, AppActionGroup, "buildprofileclean");
new_stateless_action!(pub LibsurviveSetupAction, AppActionGroup, "libsurvivesetup");
new_stateless_action!(pub QuitAction, AppActionGroup, "quit");
new_stateful_action!(pub DebugViewToggleAction, AppActionGroup, "debugviewtoggle", (), bool);
