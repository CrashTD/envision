use super::styles::bordered_container::BorderedContainer;
use crate::{
    builders::build_monado::get_build_monado_runner,
    config::{get_config, save_config, Config},
    constants::APP_NAME,
    profile::Profile,
    profiles::valve_index::valve_index_profile,
    runner::{Runner, RunnerStatus},
    ui::widgets::widgets::vseparator,
};
use expect_dialog::ExpectDialog;
use iced::{
    executor,
    theme::{Button, Container},
    time,
    widget::{
        button, checkbox, column, container, pick_list, row, scrollable, text, text_input, Space,
    },
    Alignment, Application, Command, Element, Length, Padding, Subscription, Theme,
};
use iced_aw::{Card, Modal};
use std::{
    cell::Cell,
    time::{Duration, Instant},
};

pub struct MainWin {
    profiles: Vec<Profile>,
    config: Config,
    debug_search_term: String,
    monado_runner: Cell<Runner>,
    monado_active: bool,
    monado_log: String,
    build_monado_runner: Option<Runner>,
    show_build_monado_modal: bool,
    build_monado_log: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Start,
    Stop,
    Tick,
    ProfileChanged(Profile),
    EditProfile,
    DebugViewChanged(bool),
    LogLevelChanged(String),
    DebugSearchChanged(String),
    LogUpdate(Instant),
    InstallMonado,
    CloseBuildMonadoModal,
}

impl MainWin {
    fn get_profile_by_name(&self, name: &String) -> Option<&Profile> {
        for profile in &self.profiles {
            if &profile.name == name {
                return Some(profile);
            }
        }
        return None;
    }

    fn get_selected_profile(&self) -> Option<&Profile> {
        self.get_profile_by_name(&self.config.selected_profile_name)
    }

    fn start_monado(&mut self) {
        self.monado_runner.get_mut().terminate();

        let profile = self.get_selected_profile().unwrap();

        self.monado_runner
            .set(Runner::monado_runner_from_profile(profile.clone()));

        self.monado_runner.get_mut().start();
    }

    fn get_monado_log(&mut self) {
        self.monado_log = match self.monado_runner.get_mut().status() {
            RunnerStatus::Running => self.monado_runner.get_mut().get_output(),
            RunnerStatus::Stopped => "Monado service inactive".into(),
        }
    }

    fn install_monado(&mut self) {
        self.build_monado_log = String::new();
        let profile = match self.get_selected_profile() {
            None => {
                return;
            }
            Some(p) => p,
        };
        self.build_monado_runner = Some(get_build_monado_runner(profile.clone()));
        self.build_monado_runner.as_mut().unwrap().start();
        self.show_build_monado_modal = true;
        self.build_monado_log = self.build_monado_runner.as_mut().unwrap().get_output();
    }
}

impl Application for MainWin {
    type Message = Message;
    type Flags = ();
    type Executor = executor::Default;
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let profiles = vec![valve_index_profile()];
        let monado_runner = Cell::new(Runner::monado_runner_from_profile(
            profiles.get(0).cloned().unwrap(),
        ));
        (
            Self {
                // TODO: load profiles from disk somehow
                profiles,
                config: get_config(),
                debug_search_term: "".into(),
                monado_runner,
                monado_active: false,
                monado_log: "".into(),
                build_monado_runner: None,
                show_build_monado_modal: false,
                build_monado_log: String::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        return APP_NAME.into();
    }

    fn update(&mut self, msg: Message) -> Command<Message> {
        match msg {
            Message::Start => {
                self.start_monado();
            }
            Message::Stop => {
                self.monado_runner.get_mut().terminate();
            }
            Message::ProfileChanged(profile) => {
                if self.config.selected_profile_name != profile.name {
                    self.config.selected_profile_name = profile.name;
                    save_config(&self.config).expect_dialog("Failed to save config");
                }
            }
            Message::EditProfile => {
                println!("Edit {}", self.get_selected_profile().unwrap())
            }
            Message::DebugViewChanged(nval) => {
                self.config.debug_view_enabled = nval;
                save_config(&self.config).expect_dialog("Failed to save config");
            }
            Message::DebugSearchChanged(term) => self.debug_search_term = term,
            Message::InstallMonado => {
                self.install_monado();
            }
            Message::CloseBuildMonadoModal => match &mut self.build_monado_runner {
                None => {
                    self.show_build_monado_modal = false;
                }
                Some(runner) => match runner.status() {
                    RunnerStatus::Running => {}
                    RunnerStatus::Stopped => {
                        self.show_build_monado_modal = false;
                    }
                },
            },
            Message::LogUpdate(_) => {
                self.monado_active = match self.monado_runner.get_mut().status() {
                    RunnerStatus::Running => true,
                    RunnerStatus::Stopped => false,
                };
                self.get_monado_log();
                if self.show_build_monado_modal {
                    match &mut self.build_monado_runner {
                        None => {}
                        Some(runner) => match runner.status() {
                            RunnerStatus::Running => {
                                self.build_monado_log = runner.get_output();
                            }
                            RunnerStatus::Stopped => {}
                        },
                    }
                }
            }
            _ => println!("unhandled"),
        };
        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        time::every(Duration::from_millis(500)).map(Message::LogUpdate)
    }

    fn view(&self) -> Element<Message> {
        let debug_view: Element<Message> = match self.config.debug_view_enabled {
            false => Space::new(Length::Shrink, Length::Shrink).into(),
            true => {
                let debug_toolbar = container(row![
                    pick_list(
                        vec![
                            "Debug".to_string(),
                            "Info".into(),
                            "Warning".into(),
                            "Error".into(),
                        ],
                        Some("Debug".into()),
                        Message::LogLevelChanged
                    ),
                    Space::new(Length::Fill, Length::Shrink),
                    text_input("Search...", self.debug_search_term.as_str())
                        .on_input(Message::DebugSearchChanged),
                ])
                .style(Container::Custom(Box::new(BorderedContainer)))
                .padding(12);

                let view = column![
                    // TODO: tab bar
                    container(
                        scrollable(text(self.monado_log.clone()))
                            .height(Length::Fill)
                            .width(Length::Fill)
                    )
                    // .style(Container::Custom(Box::new(RoundedBorderedContainer)))
                    .padding(6)
                    .height(Length::Fill),
                    debug_toolbar,
                ];

                row![vseparator(), view]
                    .width(Length::FillPortion(3))
                    .into()
            }
        };

        let toolbar = container(
            row![
                pick_list(
                    self.profiles.to_vec(),
                    self.get_selected_profile().cloned(),
                    Message::ProfileChanged
                ),
                button("Edit").on_press(Message::EditProfile),
                Space::new(Length::Fill, Length::Shrink),
                checkbox(
                    "Debug View",
                    self.config.debug_view_enabled,
                    Message::DebugViewChanged
                ),
            ]
            .align_items(Alignment::Center)
            .spacing(3),
        )
        .padding(12)
        .style(Container::Custom(Box::new(BorderedContainer)));

        let monado_view = column![
            match self.monado_active {
                true => button("Stop")
                    .on_press(Message::Stop)
                    .style(Button::Destructive),
                false => button("Start").on_press(Message::Start),
            }
            .padding(Padding::from([6, 24])),
            scrollable(
                column![button("DEBUG: install monado").on_press(Message::InstallMonado),]
                    .spacing(6)
            ) // device info goes here
        ]
        .spacing(12)
        .height(Length::Fill)
        .padding(12)
        .align_items(Alignment::Center);

        let main_view = column![monado_view, toolbar,]
            .width(Length::FillPortion(2))
            .height(Length::Fill);

        let win_content = row![main_view, debug_view,];

        Modal::new(self.show_build_monado_modal, win_content, || {
            Card::new(
                text("Installing Monado..."),
                column![scrollable(text(self.build_monado_log.clone())),].spacing(12),
            )
            .foot(row![
                button("Close").on_press(Message::CloseBuildMonadoModal),
            ])
            .max_width(600.0)
            .max_height(400.0)
            .into()
        })
        .into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }
}
