use super::{styles::bordered_container::BorderedContainer, widgets::widgets::hspacer};
use crate::{
    config::{get_config, save_config, Config},
    constants::APP_NAME,
    profile::{load_profile, Profile},
    runner::{Runner, RunnerStatus},
    ui::widgets::widgets::vseparator,
};
use iced::{
    executor,
    theme::{Button, Container},
    time,
    widget::{button, checkbox, column, container, pick_list, row, scrollable, text, text_input},
    Alignment, Application, Command, Element, Length, Padding, Subscription, Theme,
};
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
}

impl Application for MainWin {
    type Message = Message;
    type Flags = ();
    type Executor = executor::Default;
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let profiles = vec![load_profile(&"./test/files/profile.json".to_string()).unwrap()];
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
                    save_config(&self.config);
                }
            }
            Message::EditProfile => {
                println!("Edit {}", self.get_selected_profile().unwrap())
            }
            Message::DebugViewChanged(nval) => {
                self.config.debug_view_enabled = nval;
                save_config(&self.config);
            }
            Message::DebugSearchChanged(term) => self.debug_search_term = term,
            Message::LogUpdate(_) => {
                self.monado_active = match self.monado_runner.get_mut().status() {
                    RunnerStatus::Running => true,
                    RunnerStatus::Stopped => false,
                }; 
                self.get_monado_log();
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
            false => row![].into(),
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
                    hspacer!(),
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
                    .width(Length::FillPortion(2))
                    .into()
            }
        };

        let toolbar = container(row![
            pick_list(
                self.profiles.to_vec(),
                self.get_selected_profile().cloned(),
                Message::ProfileChanged
            ),
            button("Edit").on_press(Message::EditProfile),
            hspacer!(),
            checkbox(
                "Debug View",
                self.config.debug_view_enabled,
                Message::DebugViewChanged
            ),
        ])
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
            scrollable(column![]) // device info goes here
        ]
        .height(Length::Fill)
        .padding(12)
        .align_items(Alignment::Center);

        let main_view = column![monado_view, toolbar,]
            .width(Length::FillPortion(1))
            .height(Length::Fill);

        row![main_view, debug_view,].into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }
}
