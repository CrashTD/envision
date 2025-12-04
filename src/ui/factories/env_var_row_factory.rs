use adw::prelude::*;
use relm4::{AsyncFactorySender, factory::AsyncFactoryComponent, prelude::*};

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

#[relm4::factory(async pub)]
impl AsyncFactoryComponent for EnvVarModel {
    type Init = EnvVarModelInit;
    type Input = EnvVarModelMsg;
    type Output = EnvVarModelOutMsg;
    type CommandOutput = ();
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
                self.value.clone_from(&val);
                sender
                    .output_sender()
                    .emit(Self::Output::Changed(self.name.clone(), val));
            }
            Self::Input::Delete => {
                let _ = sender.output(Self::Output::Delete(self.name.clone()));
            }
        }
    }

    async fn init_model(
        init: Self::Init,
        _index: &DynamicIndex,
        _sender: AsyncFactorySender<Self>,
    ) -> Self {
        Self {
            name: init.name,
            value: init.value,
        }
    }
}
