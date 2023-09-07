use super::factories::tracker_role_group_factory::TrackerRoleModel;
use crate::{
    file_builders::monado_config_v0::{
        dump_monado_config_v0, get_monado_config_v0, MonadoConfigV0, TrackerRole,
    },
    ui::factories::tracker_role_group_factory::TrackerRoleModelInit,
    withclones,
};
use adw::prelude::*;
use relm4::{factory::FactoryVecDeque, prelude::*};

#[tracker::track]
pub struct FbtConfigEditor {
    monado_config_v0: MonadoConfigV0,

    #[tracker::do_not_track]
    win: Option<adw::PreferencesWindow>,

    #[tracker::do_not_track]
    tracker_role_groups: FactoryVecDeque<TrackerRoleModel>,
}

#[derive(Debug)]
pub enum FbtConfigEditorMsg {
    Present,
    Save,
    TrackerRoleNew,
    TrackerRoleChanged(usize, TrackerRole),
    TrackerRoleDeleted(usize),
    Repopulate,
}

pub struct FbtConfigEditorInit {
    pub root_win: gtk::Window,
}
#[relm4::component(pub)]
impl SimpleComponent for FbtConfigEditor {
    type Init = FbtConfigEditorInit;
    type Input = FbtConfigEditorMsg;
    type Output = ();

    view! {
        #[name(win)]
        adw::PreferencesWindow {
            set_title: Some("Full Body Trackers"),
            set_modal: true,
            set_transient_for: Some(&init.root_win),
            set_search_enabled: false,
            add: model.tracker_role_groups.widget(),
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        self.reset();

        match message {
            Self::Input::Present => {
                self.win.as_ref().unwrap().present();
            }
            Self::Input::Save => {
                dump_monado_config_v0(&self.monado_config_v0);
                self.win.as_ref().unwrap().close();
            }
            Self::Input::TrackerRoleChanged(index, n_role) => {
                let role = self.monado_config_v0.tracker_roles.get_mut(index).unwrap();
                role.device_serial = n_role.device_serial;
                role.role = n_role.role;
                role.xrt_input_name = n_role.xrt_input_name;
            }
            Self::Input::TrackerRoleDeleted(index) => {
                self.monado_config_v0.tracker_roles.remove(index);
                sender.input(Self::Input::Repopulate);
            }
            Self::Input::TrackerRoleNew => {
                self.monado_config_v0
                    .tracker_roles
                    .push(TrackerRole::default());
                self.tracker_role_groups
                    .guard()
                    .push_back(TrackerRoleModelInit {
                        index: self.monado_config_v0.tracker_roles.len() - 1,
                        tracker_role: None,
                    });
            }
            Self::Input::Repopulate => {
                self.populate_tracker_roles();
            }
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let page = adw::PreferencesPage::builder().build();
        let grp = adw::PreferencesGroup::builder().build();
        let add_btn = gtk::Button::builder()
            .label("Add Tracker")
            .margin_bottom(12)
            .css_classes(["pill"])
            .halign(gtk::Align::Center)
            .build();
        let save_btn = gtk::Button::builder()
            .label("Save")
            .margin_bottom(12)
            .halign(gtk::Align::Center)
            .css_classes(["suggested-action", "pill"])
            .build();
        {
            withclones![sender];
            add_btn.connect_clicked(move |_| {
                sender.input(Self::Input::TrackerRoleNew);
            });
        };
        {
            withclones![sender];
            save_btn.connect_clicked(move |_| {
                sender.input(Self::Input::Save);
            });
        };
        grp.add(&save_btn);
        grp.add(&add_btn);
        page.add(&grp);

        let mut model = Self {
            win: None,
            tracker: 0,
            monado_config_v0: get_monado_config_v0(),
            tracker_role_groups: FactoryVecDeque::new(page, sender.input_sender()),
        };

        model.populate_tracker_roles();

        let widgets = view_output!();
        model.win = Some(widgets.win.clone());

        ComponentParts { model, widgets }
    }
}

impl FbtConfigEditor {
    fn populate_tracker_roles(&mut self) {
        let mut guard = self.tracker_role_groups.guard();
        guard.clear();
        for (i, v) in self.monado_config_v0.tracker_roles.iter().enumerate() {
            guard.push_back(TrackerRoleModelInit {
                index: i,
                tracker_role: Some(v.clone()),
            });
        }
    }
}
