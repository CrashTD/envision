use crate::ui::profile_editor::ProfileEditorMsg;
use adw::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub struct EnvVarModel {
    pub name: String,
    value: String,
}

pub struct EnvVarModelInit {
    pub name: String,
    pub value: String,
}

#[derive(Debug)]
pub enum EnvVarModelMsg {
    Changed(String),
    Delete,
}

#[derive(Debug)]
pub enum EnvVarModelOutMsg {
    Changed(String, String),
    Delete(String),
}

#[relm4::factory(pub)]
impl FactoryComponent for EnvVarModel {
    type Init = EnvVarModelInit;
    type Input = EnvVarModelMsg;
    type Output = EnvVarModelOutMsg;
    type CommandOutput = ();
    type Widgets = EnvVarModelWidgets;
    type ParentInput = ProfileEditorMsg;
    type ParentWidget = adw::PreferencesGroup;

    view! {
        root = adw::EntryRow {
            set_title:  &self.name,
            set_text: &self.value,
            add_suffix: del_btn = &gtk::Button {
                set_icon_name: "edit-delete-symbolic",
                set_tooltip_text: Some("Delete Environment Variable"),
                set_valign: gtk::Align::Center,
                add_css_class: "flat",
                add_css_class: "circular",
                connect_clicked[sender] => move |_| {
                    sender.input(Self::Input::Delete);
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
                    .emit(Self::Output::Changed(self.name.clone(), val));
            }
            Self::Input::Delete => {
                sender.output(Self::Output::Delete(self.name.clone()));
            }
        }
    }

    fn forward_to_parent(output: Self::Output) -> Option<Self::ParentInput> {
        Some(match output {
            Self::Output::Changed(name, value) => ProfileEditorMsg::EnvVarChanged(name, value),
            Self::Output::Delete(name) => ProfileEditorMsg::EnvVarDelete(name),
        })
    }

    fn init_model(init: Self::Init, _index: &Self::Index, _sender: FactorySender<Self>) -> Self {
        Self {
            name: init.name,
            value: init.value,
        }
    }
}
