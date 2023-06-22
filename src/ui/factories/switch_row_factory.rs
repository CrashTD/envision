use adw::prelude::*;
use gtk::prelude::*;
use relm4::prelude::*;

use crate::ui::profile_editor::ProfileEditorMsg;

#[derive(Debug)]
pub struct SwitchModel {
    name: String,
    description: Option<String>,
    key: String,
    value: bool,
}

pub struct SwitchModelInit {
    pub name: String,
    pub description: Option<String>,
    pub key: String,
    pub value: bool,
}

#[derive(Debug)]
pub enum SwitchModelMsg {
    Changed(bool),
}

#[derive(Debug)]
pub enum SwitchModelOutMsg {
    /** key, value */
    Changed(String, bool),
}

#[relm4::factory(pub)]
impl FactoryComponent for SwitchModel {
    type Init = SwitchModelInit;
    type Input = SwitchModelMsg;
    type Output = SwitchModelOutMsg;
    type CommandOutput = ();
    type Widgets = SwitchModelWidgets;
    type ParentInput = ProfileEditorMsg;
    type ParentWidget = adw::PreferencesGroup;

    view! {
        root = adw::ActionRow {
            set_title: &self.name,
            set_subtitle_lines: 0,
            set_subtitle: match &self.description {
                Some(s) => s,
                None => "".into(),
            },
            add_suffix: switch = &gtk::Switch {
                set_valign: gtk::Align::Center,
                set_active: self.value,
                connect_state_set[sender] => move |_, state| {
                    sender.input(Self::Input::Changed(state));
                    gtk::Inhibit(false)
                }
            },
            set_activatable_widget: Some(&switch),
        }
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            Self::Input::Changed(val) => {
                self.value = val.clone();
                sender
                    .output_sender()
                    .emit(Self::Output::Changed(self.key.clone(), val))
            }
        }
    }

    fn forward_to_parent(_output: Self::Output) -> Option<Self::ParentInput> {
        None
    }

    fn init_model(init: Self::Init, index: &Self::Index, sender: FactorySender<Self>) -> Self {
        Self {
            name: init.name,
            description: init.description,
            key: init.key,
            value: init.value,
        }
    }
}
