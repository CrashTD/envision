use crate::ui::profile_editor::ProfileEditorMsg;
use adw::prelude::*;
use relm4::{factory::AsyncFactoryComponent, prelude::*, AsyncFactorySender};

#[derive(Debug, Clone, Copy)]
pub enum VarType {
    EnvVar,
    XrServiceCmakeFlags,
}

#[derive(Debug)]
pub struct EnvVarModel {
    pub name: String,
    value: String,
    var_type: VarType,
}

pub struct EnvVarModelInit {
    pub name: String,
    pub value: String,
    pub var_type: VarType,
}

#[derive(Debug)]
pub enum EnvVarModelMsg {
    Changed(String),
    Delete,
}

#[derive(Debug)]
pub enum EnvVarModelOutMsg {
    Changed(VarType, String, String),
    Delete(VarType, String),
}

#[relm4::factory(async pub)]
impl AsyncFactoryComponent for EnvVarModel {
    type Init = EnvVarModelInit;
    type Input = EnvVarModelMsg;
    type Output = EnvVarModelOutMsg;
    type CommandOutput = ();
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

    async fn update(&mut self, message: Self::Input, sender: AsyncFactorySender<Self>) {
        match message {
            Self::Input::Changed(val) => {
                self.value = val.clone();
                sender.output_sender().emit(Self::Output::Changed(
                    self.var_type,
                    self.name.clone(),
                    val,
                ));
            }
            Self::Input::Delete => {
                sender.output(Self::Output::Delete(self.var_type, self.name.clone()));
            }
        }
    }

    fn forward_to_parent(output: Self::Output) -> Option<Self::ParentInput> {
        Some(match output {
            Self::Output::Changed(var_type, name, value) => match var_type {
                VarType::EnvVar => ProfileEditorMsg::EnvVarChanged(name, value),
                VarType::XrServiceCmakeFlags => {
                    ProfileEditorMsg::XrServiceCmakeFlagsChanged(name, value)
                }
            },
            Self::Output::Delete(var_type, name) => match var_type {
                VarType::EnvVar => ProfileEditorMsg::EnvVarDelete(name),
                VarType::XrServiceCmakeFlags => ProfileEditorMsg::XrServiceCmakeFlagsDelete(name),
            },
        })
    }

    async fn init_model(
        init: Self::Init,
        _index: &DynamicIndex,
        _sender: AsyncFactorySender<Self>,
    ) -> Self {
        Self {
            name: init.name,
            value: init.value,
            var_type: init.var_type,
        }
    }
}
