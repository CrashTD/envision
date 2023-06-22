use crate::ui::profile_editor::ProfileEditorMsg;
use adw::prelude::*;
use gtk::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub struct EntryModel {
    name: String,
    value: String,
}

pub struct EntryModelInit {
    pub name: String,
    pub value: String,
}

#[derive(Debug)]
pub enum EntryModelMsg {
    Changed(String),
}

#[derive(Debug)]
pub enum EntryModelOutMsg {
    Changed(String, String),
}

#[relm4::factory(pub)]
impl FactoryComponent for EntryModel {
    type Init = EntryModelInit;
    type Input = EntryModelMsg;
    type Output = EntryModelOutMsg;
    type CommandOutput = ();
    type Widgets = EntryModelWidgets;
    type ParentInput = ProfileEditorMsg;
    type ParentWidget = adw::PreferencesGroup;

    view! {
        root = adw::EntryRow {
            set_title:  &self.name,
            set_text: &self.value,
            connect_changed[sender] => move |entry| {
                sender.input_sender().emit(Self::Input::Changed(entry.text().to_string()));
            },
        }
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            Self::Input::Changed(val) => {
                self.value = val.clone();
                sender
                    .output_sender()
                    .emit(Self::Output::Changed(self.name.clone(), val));
            }
        }
    }

    fn forward_to_parent(output: Self::Output) -> Option<Self::ParentInput> {
        Some(match output {
            Self::Output::Changed(name, value) => ProfileEditorMsg::EntryChanged(name, value),
        })
    }

    fn init_model(init: Self::Init, index: &Self::Index, sender: FactorySender<Self>) -> Self {
        Self {
            name: init.name,
            value: init.value,
        }
    }
}
