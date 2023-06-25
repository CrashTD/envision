use adw::prelude::*;
use gtk::prelude::*;
use relm4::prelude::*;

use crate::ui::profile_editor::ProfileEditorMsg;

#[derive(Debug)]
pub struct PathModel {
    name: String,
    key: String,
    value: Option<String>,
    path_label: gtk::Label,
    filedialog: gtk::FileDialog,
}

pub struct PathModelInit {
    pub name: String,
    pub key: String,
    pub value: Option<String>,
}

#[derive(Debug)]
pub enum PathModelMsg {
    Changed(Option<String>),
    OpenFileChooser,
}

#[derive(Debug)]
pub enum PathModelOutMsg {
    /** key, value */
    Changed(String, Option<String>),
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
            add_suffix: unset_btn = &gtk::Button {
                set_valign: gtk::Align::Center,
                add_css_class: "flat",
                add_css_class: "circular",
                set_icon_name: "edit-clear-symbolic",
                set_tooltip_text: Some("Clear Path"),
                connect_clicked[sender] => move |_| {
                    sender.input(Self::Input::Changed(None));
                }
            },
            connect_activated[sender] => move |_| {
                sender.input(Self::Input::OpenFileChooser)
            },
        }
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            Self::Input::Changed(val) => {
                self.value = val;
                self.path_label.set_label(match self.value.as_ref() {
                    Some(val) => val.as_str(),
                    None => "(None)".into(),
                });
                sender
                    .output_sender()
                    .emit(Self::Output::Changed(self.key.clone(), self.value.clone()))
            }
            Self::Input::OpenFileChooser => {
                let fd_sender = sender.clone();
                self.filedialog.select_folder(
                    self.init_root().root().and_downcast_ref::<gtk::Window>(),
                    gtk::gio::Cancellable::NONE,
                    move |res| match res {
                        Ok(file) => {
                            let path = file.path();
                            if path.is_some() {
                                fd_sender.input(Self::Input::Changed(
                                    Some(path.unwrap().to_str().unwrap().to_string())
                                ));
                            }
                        }
                        _ => {}
                    },
                );
            }
        }
    }

    fn forward_to_parent(output: Self::Output) -> Option<Self::ParentInput> {
        Some(match output {
            Self::Output::Changed(key, value) => ProfileEditorMsg::PathChanged(key, value)
        })
    }

    fn init_model(init: Self::Init, index: &Self::Index, sender: FactorySender<Self>) -> Self {
        Self {
            name: init.name.clone(),
            key: init.key,
            value: init.value.clone(),
            path_label: gtk::Label::builder()
                .label(match init.value {
                    Some(val) => val,
                    None => "(None)".into(),
                })
                .ellipsize(gtk::pango::EllipsizeMode::Start)
                .build(),
            filedialog: gtk::FileDialog::builder()
                .modal(true)
                .title(format!("Select Path for {}", init.name.clone()))
                .build(),
        }
    }
}
