use adw::prelude::*;
use gtk::prelude::*;
use relm4::prelude::*;

use crate::ui::profile_editor::ProfileEditorMsg;

#[derive(Debug)]
pub struct PathModel {
    name: String,
    key: String,
    value: String,
    path_label: gtk::Label,
}

pub struct PathModelInit {
    pub name: String,
    pub key: String,
    pub value: Option<String>,
}

#[derive(Debug)]
pub enum PathModelMsg {
    Changed(String),
    OpenFileChooser,
}

#[derive(Debug)]
pub enum PathModelOutMsg {
    /** key, value */
    Changed(String, String),
}

#[relm4::factory(pub)]
impl FactoryComponent for PathModel {
    type Init = PathModelInit;
    type Input = PathModelMsg;
    type Output = PathModelOutMsg;
    type CommandOutput = ();
    type Widgets = PathModelWidgets;
    type ParentInput = ProfileEditorMsg;
    type ParentWidget = adw::PreferencesGroup;

    view! {
        root = adw::ActionRow {
            set_title: &self.name,
            set_subtitle_lines: 0,
            set_icon_name: Some("folder-open-symbolic"),
            add_suffix: &self.path_label,
            set_activatable: true,
            connect_activated[sender] => move |_| {
                sender.input(Self::Input::OpenFileChooser)
            }
        }
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            Self::Input::Changed(val) => {
                self.value = val.clone();
                self.path_label.set_label(val.as_str());
                sender
                    .output_sender()
                    .emit(Self::Output::Changed(self.key.clone(), val))
            }
            Self::Input::OpenFileChooser => {
                println!("file chooser");
            }
        }
    }

    fn forward_to_parent(_output: Self::Output) -> Option<Self::ParentInput> {
        None
    }

    fn init_model(init: Self::Init, index: &Self::Index, sender: FactorySender<Self>) -> Self {
        Self {
            name: init.name,
            key: init.key,
            value: match init.value.as_ref() {
                Some(val) => val.clone(), None => "".into()
            },
            path_label: gtk::Label::builder().label(match init.value {
                Some(val) => val, None => "(None)".into()
            }).ellipsize(gtk::pango::EllipsizeMode::Start).build(),
        }
    }
}
