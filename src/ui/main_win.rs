use crate::{
    config::{get_config, save_config, Config},
    constants::APP_NAME,
    profile::{load_profile, Profile},
};
use iced::{
    executor,
    widget::{button, checkbox, column, pick_list, row, scrollable, text, text_input},
    Alignment, Application, Command, Element, Length, Padding, Theme,
};

pub struct MainWin {
    profiles: Vec<Profile>,
    monado_active: bool,
    config: Config,
    debug_search_term: String,
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
}

impl Application for MainWin {
    type Message = Message;
    type Flags = ();
    type Executor = executor::Default;
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                // TODO: load profiles from disk somehow
                profiles: vec![load_profile(&"./test/files/profile.json".to_string()).unwrap()],
                monado_active: false,
                config: get_config(),
                debug_search_term: "".into(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        return APP_NAME.into();
    }

    fn update(&mut self, msg: Message) -> Command<Message> {
        match msg {
            Message::Start => self.monado_active = true,
            Message::Stop => self.monado_active = false,
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
            _ => println!("unhandled"),
        };
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        row![
            column![
                column![match self.monado_active {
                    true => button("Stop").on_press(Message::Stop),
                    false => button("Start").on_press(Message::Start),
                }
                .padding(Padding::from([6, 24])),]
                .height(Length::Fill),
                row![
                    pick_list(
                        self.profiles.to_vec(),
                        self.get_selected_profile().cloned(),
                        Message::ProfileChanged
                    ),
                    button("Edit").on_press(Message::EditProfile),
                    row![].width(Length::Fill),
                    checkbox(
                        "Debug View",
                        self.config.debug_view_enabled,
                        Message::DebugViewChanged
                    ),
                ]
                .width(Length::Fill),
            ]
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(12)
            .align_items(Alignment::Center),
            match self.config.debug_view_enabled {
                true => column![
                    // TODO: tab bar
                    scrollable(text("Foo bar baz"))
                        .width(Length::Fill)
                        .height(Length::Fill),
                    row![
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
                        row![].width(Length::Fill),
                        text_input("Search...", self.debug_search_term.as_str())
                            .on_input(Message::DebugSearchChanged),
                    ]
                    .padding(12)
                ]
                .width(Length::Fill),
                false => column![],
            }
        ]
        .into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }
}
