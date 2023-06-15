use super::about_dialog::AboutDialog;
use super::build_window::BuildWindow;
use super::debug_view::{DebugView, DebugViewMsg};
use super::main_view::MainViewMsg;
use crate::builders::build_libsurvive::get_build_libsurvive_runner;
use crate::builders::build_monado::get_build_monado_runner;
use crate::builders::build_opencomposite::get_build_opencomposite_runner;
use crate::config::{get_config, save_config, Config};
use crate::constants::APP_NAME;
use crate::dependencies::libsurvive_deps::get_missing_libsurvive_deps;
use crate::dependencies::monado_deps::get_missing_monado_deps;
use crate::file_builders::active_runtime_json::{
    set_current_active_runtime_to_profile, set_current_active_runtime_to_steam,
};
use crate::file_builders::openvrpaths_vrpath::{
    set_current_openvrpaths_to_profile, set_current_openvrpaths_to_steam,
};
use crate::profile::Profile;
use crate::profiles::valve_index::valve_index_profile;
use crate::runner::{Runner, RunnerStatus};
use crate::runner_pipeline::RunnerPipeline;
use crate::ui::build_window::BuildWindowMsg;
use crate::ui::debug_view::DebugViewInit;
use crate::ui::main_view::{MainView, MainViewInit, MainViewOutMsg};
use expect_dialog::ExpectDialog;
use gtk::prelude::*;
use relm4::actions::{ActionGroupName, RelmAction, RelmActionGroup};
use relm4::adw::traits::MessageDialogExt;
use relm4::gtk::glib;
use relm4::{new_action_group, new_stateful_action, new_stateless_action, prelude::*};
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use std::time::Duration;

#[tracker::track]
pub struct App {
    enable_debug_view: bool,

    #[tracker::do_not_track]
    main_view: Controller<MainView>,
    #[tracker::do_not_track]
    debug_view: Controller<DebugView>,
    #[tracker::do_not_track]
    about_dialog: Controller<AboutDialog>,
    #[tracker::do_not_track]
    build_window: Controller<BuildWindow>,
    #[tracker::do_not_track]
    dependencies_dialog: adw::MessageDialog,

    #[tracker::do_not_track]
    config: Config,
    #[tracker::do_not_track]
    monado_runner: Option<Runner>,
    #[tracker::do_not_track]
    monado_log: Vec<String>,
    #[tracker::do_not_track]
    build_pipeline: Option<RunnerPipeline>,
    #[tracker::do_not_track]
    profiles: Vec<Profile>,
}

#[derive(Debug)]
pub enum Msg {
    UpdateView,
    BuildProfile,
    EnableDebugViewChanged(bool),
    DoStartStopMonado,
    ProfileSelected(String),
    SetMonadoRuntime(bool),
}

impl App {
    fn get_profile_by_name(&self, name: &String) -> Option<&Profile> {
        for profile in &self.profiles {
            if &profile.name == name {
                return Some(profile);
            }
        }
        return None;
    }

    pub fn get_selected_profile(&self) -> &Profile {
        match self.get_profile_by_name(&self.config.selected_profile_name) {
            Some(profile) => profile,
            None => self.profiles.get(0).expect_dialog("No profiles found"),
        }
    }

    pub fn start_monado(&mut self) {
        self.monado_log.clear();
        let mut runner = Runner::monado_runner_from_profile(self.get_selected_profile().clone());
        runner.start();
        self.monado_runner = Some(runner);
    }
}

#[relm4::component(pub)]
impl SimpleComponent for App {
    type Init = ();
    type Input = Msg;
    type Output = ();

    view! {
        #[root]
        adw::ApplicationWindow {
            set_title: Some(APP_NAME),
            set_default_size: (400, 300),
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: true,
                set_vexpand: true,
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,
                    set_vexpand: true,
                    append: model.main_view.widget(),
                    append: sep = &gtk::Separator {
                        set_orientation: gtk::Orientation::Vertical,
                        #[track = "model.changed(App::enable_debug_view())"]
                        set_visible: model.enable_debug_view,
                    },
                    append: model.debug_view.widget(),
                    // append: model.debug_view.widget(),
                }
            }
        }
    }

    fn shutdown(&mut self, widgets: &mut Self::Widgets, output: relm4::Sender<Self::Output>) {
        match &mut self.monado_runner {
            None => {}
            Some(runner) => {
                runner.terminate();
            }
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        self.reset();

        match message {
            Msg::UpdateView => {
                match &mut self.monado_runner {
                    None => {}
                    Some(runner) => {
                        let n_rows = runner.consume_rows();
                        if !n_rows.is_empty() {
                            self.monado_log.extend(n_rows);
                            self.debug_view
                                .sender()
                                .emit(DebugViewMsg::LogUpdated(self.monado_log.clone()));
                        }
                        match runner.status() {
                            RunnerStatus::Running => {},
                            RunnerStatus::Stopped => {
                                self.main_view.sender().emit(MainViewMsg::MonadoActiveChanged(false));
                            }
                        };
                    }
                };
                match &mut self.build_pipeline {
                    None => {}
                    Some(pipeline) => match pipeline.is_running() {
                        true => {
                            pipeline.update();
                            self.build_window
                                .sender()
                                .emit(BuildWindowMsg::UpdateContent(pipeline.get_log()));
                        }
                        false => {
                            self.build_window
                                .sender()
                                .emit(BuildWindowMsg::UpdateCanClose(true));
                        }
                    },
                };
            }
            Msg::EnableDebugViewChanged(val) => {
                self.set_enable_debug_view(val);
                self.config.debug_view_enabled = val;
                save_config(&self.config);
                self.debug_view
                    .sender()
                    .emit(DebugViewMsg::EnableDebugViewChanged(val));
                self.main_view
                    .sender()
                    .emit(MainViewMsg::EnableDebugViewChanged(val));
            }
            Msg::DoStartStopMonado => match &mut self.monado_runner {
                None => {
                    self.debug_view
                        .sender()
                        .emit(DebugViewMsg::LogUpdated(vec![]));
                    self.start_monado();
                    self.main_view
                        .sender()
                        .emit(MainViewMsg::MonadoActiveChanged(true));
                    self.main_view
                        .sender()
                        .emit(MainViewMsg::SteamLaunchOptionsChanged(
                            self.get_selected_profile().get_steam_launch_options(),
                        ));
                }
                Some(runner) => match runner.status() {
                    RunnerStatus::Running => {
                        runner.terminate();
                        self.main_view
                            .sender()
                            .emit(MainViewMsg::MonadoActiveChanged(false));
                    }
                    RunnerStatus::Stopped => {
                        self.debug_view
                            .sender()
                            .emit(DebugViewMsg::LogUpdated(vec![]));
                        self.start_monado();
                        self.main_view
                            .sender()
                            .emit(MainViewMsg::MonadoActiveChanged(true));
                    }
                },
            },
            Msg::BuildProfile => {
                let profile = self.get_selected_profile();
                let mut missing_deps = get_missing_monado_deps();
                let mut runners: Vec<Runner> = vec![];
                if profile.libsurvive_enabled {
                    missing_deps.extend(get_missing_libsurvive_deps());
                    runners.push(get_build_libsurvive_runner(profile.clone()));
                }
                // if profile.basalt_enabled {
                //     missing_deps.extend(get_missing_basalt_deps());
                //     runners.push(get_build_basalt_runner(profile.clone()));
                // }
                // if profile.mercury_enabled {
                //     missing_deps.extend(get_missing_mercury_deps());
                //     runners.push(get_build_mercury_runner(profile.clone()));
                // }
                runners.push(get_build_monado_runner(profile.clone()));
                // no listed deps for opencomp
                runners.push(get_build_opencomposite_runner(profile.clone()));
                if !missing_deps.is_empty() {
                    self.dependencies_dialog.set_body(
                        missing_deps
                            .iter()
                            .map(|dep| dep.name.clone())
                            .collect::<Vec<String>>()
                            .join(", ")
                            .as_str(),
                    );
                    self.dependencies_dialog.present();
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
                    .emit(BuildWindowMsg::UpdateTitle("Building Monado".into()));
                self.build_window
                    .sender()
                    .emit(BuildWindowMsg::UpdateCanClose(false));
                self.build_pipeline = Some(pipeline);
            }
            Msg::ProfileSelected(prof_name) => {
                self.config.selected_profile_name = prof_name;
                save_config(&self.config);
            }
            Msg::SetMonadoRuntime(monado) => {
                if monado {
                    let profile = self.get_selected_profile();
                    set_current_active_runtime_to_profile(profile.clone());
                    set_current_openvrpaths_to_profile(profile.clone());
                } else {
                    set_current_active_runtime_to_steam();
                    set_current_openvrpaths_to_steam();
                }
            }
        }
    }

    fn init(
        _params: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let config = get_config();
        let profiles = vec![valve_index_profile()];
        let dependencies_dialog = adw::MessageDialog::builder()
            .modal(true)
            .transient_for(root)
            .heading("Missing dependencies:")
            .hide_on_close(true)
            .build();
        dependencies_dialog.add_response("ok", "_Ok");
        let model = App {
            main_view: MainView::builder()
                .launch(MainViewInit {
                    config: config.clone(),
                })
                .forward(sender.input_sender(), |message| match message {
                    MainViewOutMsg::UpdateView => Msg::UpdateView,
                    MainViewOutMsg::EnableDebugViewChanged(val) => Msg::EnableDebugViewChanged(val),
                    MainViewOutMsg::DoStartStopMonado => Msg::DoStartStopMonado,
                    MainViewOutMsg::ProfileSelected(name) => Msg::ProfileSelected(name),
                    MainViewOutMsg::SetMonadoRuntime(monado) => Msg::SetMonadoRuntime(monado),
                }),
            debug_view: DebugView::builder()
                .launch(DebugViewInit {
                    enable_debug_view: config.debug_view_enabled,
                })
                .forward(sender.input_sender(), |message| match message {
                    _ => Msg::UpdateView,
                }),
            about_dialog: AboutDialog::builder()
                .transient_for(root)
                .launch(())
                .detach(),
            build_window: BuildWindow::builder()
                .transient_for(root)
                .launch(())
                .detach(),
            dependencies_dialog,
            enable_debug_view: config.debug_view_enabled,
            config,
            tracker: 0,
            profiles,
            monado_runner: None,
            monado_log: vec![],
            build_pipeline: None,
        };
        let widgets = view_output!();

        let mut actions = RelmActionGroup::<AppActionGroup>::new();

        let buildprofile_action = {
            let this_sender = sender.clone();
            RelmAction::<BuildProfileAction>::new_stateless(move |_| {
                this_sender.input_sender().emit(Msg::BuildProfile);
            })
        };

        let about_action = {
            let abd_sender = model.about_dialog.sender().clone();
            RelmAction::<AboutAction>::new_stateless(move |_| {
                abd_sender.send(()).unwrap();
            })
        };

        let debug_view_toggle_action: RelmAction<DebugViewToggleAction> = {
            let debugtoggle_sender = sender.clone();
            RelmAction::<DebugViewToggleAction>::new_stateful(
                &model.enable_debug_view,
                move |_, state| {
                    let s = *state;
                    *state = !s;
                    debugtoggle_sender.input(Msg::EnableDebugViewChanged(*state));
                },
            )
        };

        actions.add_action(about_action);
        actions.add_action(buildprofile_action);
        actions.add_action(debug_view_toggle_action);

        root.insert_action_group(AppActionGroup::NAME, Some(&actions.into_action_group()));

        model
            .main_view
            .sender()
            .emit(MainViewMsg::UpdateProfileNames(
                model.profiles.iter().map(|p| p.clone().name).collect(),
            ));

        let timer_sender = sender.clone();
        glib::timeout_add_local(Duration::from_millis(1000), move || {
            timer_sender.input(Msg::UpdateView);
            return glib::Continue(true);
        });

        ComponentParts { model, widgets }
    }
}

new_action_group!(pub AppActionGroup, "win");
new_stateless_action!(pub AboutAction, AppActionGroup, "about");
new_stateless_action!(pub BuildProfileAction, AppActionGroup, "buildprofile");
new_stateful_action!(pub DebugViewToggleAction, AppActionGroup, "debugviewtoggle", (), bool);
