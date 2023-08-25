use crate::{
    file_builders::monado_config_v0::{TrackerRole, XrtTrackerRole},
    ui::fbt_config_editor::FbtConfigEditorMsg,
    ui::preference_rows::{combo_row, entry_row},
    withclones,
};
use adw::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub struct TrackerRoleModel {
    index: usize,
    tracker_role: TrackerRole,
}

pub struct TrackerRoleModelInit {
    pub index: usize,
    pub tracker_role: Option<TrackerRole>,
}

#[derive(Debug)]
pub enum TrackerRoleModelMsg {
    Changed(TrackerRole),
    Delete,
}

#[derive(Debug)]
pub enum TrackerRoleModelOutMsg {
    Changed(usize, TrackerRole),
    Delete(usize),
}

#[relm4::factory(pub)]
impl FactoryComponent for TrackerRoleModel {
    type Init = TrackerRoleModelInit;
    type Input = TrackerRoleModelMsg;
    type Output = TrackerRoleModelOutMsg;
    type CommandOutput = ();
    type Widgets = TrackerRoleModelWidgets;
    type ParentInput = FbtConfigEditorMsg;
    type ParentWidget = adw::PreferencesPage;

    view! {
        root = adw::PreferencesGroup {
            set_title: "Tracker",
            #[wrap(Some)]
            set_header_suffix: del_btn = &gtk::Button {
                set_icon_name: "edit-delete-symbolic",
                set_tooltip_text: Some("Delete Tracker"),
                set_valign: gtk::Align::Center,
                add_css_class: "flat",
                add_css_class: "circular",
                connect_clicked[sender] => move |_| {
                    sender.input(Self::Input::Delete);
                }
            },
            add: {
                withclones![sender];
                let tr = self.tracker_role.clone();
                &entry_row("Device serial", self.tracker_role.device_serial.as_str(), move |row| {
                    let mut ntr = tr.clone();
                    ntr.device_serial = row.text().to_string();
                    sender.input(Self::Input::Changed(ntr));
                })
            },
            add: {
                withclones![sender];
                let tr = self.tracker_role.clone();
                &combo_row("Tracker role", None, &tr.role.clone().to_picker_string(),
                    XrtTrackerRole::iter()
                        .map(XrtTrackerRole::to_picker_string)
                        .map(String::from)
                        .collect::<Vec<String>>(),
                    move |row| {
                        let mut ntr = tr.clone();
                        ntr.role = XrtTrackerRole::from_number(&row.selected());
                        sender.input(Self::Input::Changed(ntr));
                    }
                )
            },
        }
    }

    fn forward_to_parent(output: Self::Output) -> Option<Self::ParentInput> {
        Some(match output {
            Self::Output::Changed(index, tracker_role) => {
                Self::ParentInput::TrackerRoleChanged(index, tracker_role.clone())
            }
            Self::Output::Delete(index) => Self::ParentInput::TrackerRoleDeleted(index),
        })
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            Self::Input::Changed(r) => {
                self.tracker_role = r;
                sender.output(Self::Output::Changed(self.index, self.tracker_role.clone()));
            }
            Self::Input::Delete => sender.output(Self::Output::Delete(self.index)),
        }
    }

    fn init_model(init: Self::Init, _index: &Self::Index, _sender: FactorySender<Self>) -> Self {
        Self {
            index: init.index,
            tracker_role: init.tracker_role.unwrap_or_else(|| TrackerRole::default()),
        }
    }
}
