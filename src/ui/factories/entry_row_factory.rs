use crate::ui::profile_editor::ProfileEditorMsg;
use adw::prelude::*;
use gtk::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub struct EntryModel {
    key: String,
    name: String,
    value: String,
}

pub struct EntryModelInit {
    pub key: String,
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
            add_suffix: clear_btn = &gtk::Button {
                set_icon_name: "edit-clear-symbolic",
                set_tooltip_text: Some("Clear"),
                set_valign: gtk::Align::Center,
                add_css_class: "flat",
                add_css_class: "circular",
                connect_clicked[root] => move |_| {
                    root.set_text("");
                }
            },
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
                    .emit(Self::Output::Changed(self.key.clone(), val));
            }
        }
    }

    fn forward_to_parent(output: Self::Output) -> Option<Self::ParentInput> {
        Some(match output {
            Self::Output::Changed(key, value) => ProfileEditorMsg::EntryChanged(key, value),
        })
    }

    fn init_model(init: Self::Init, index: &Self::Index, sender: FactorySender<Self>) -> Self {
        Self {
            key: init.key,
            name: init.name,
            value: init.value,
        }
    }
}
