use std::path::PathBuf;

use crate::{
    constants::APP_ID,
    ui::{
        SENDER_IO_ERR_MSG,
        preference_rows::{entry_row, file_row},
    },
};

use super::Plugin;
use adw::prelude::*;
use gtk::glib::clone;
use relm4::prelude::*;

#[tracker::track]
pub struct AddCustomPluginWin {
    #[tracker::do_not_track]
    parent: gtk::Window,
    #[tracker::do_not_track]
    win: Option<adw::Dialog>,
    /// this is true when enough fields are populated, allowing the creation
    /// of the plugin object to add
    can_add: bool,
    #[tracker::do_not_track]
    plugin: Plugin,
}

#[derive(Debug)]
pub enum AddCustomPluginWinMsg {
    Present,
    Close,
    OnNameChange(String),
    OnArgsChange(String),
    OnExecPathChange(Option<String>),
    Add,
}

#[derive(Debug)]
pub enum AddCustomPluginWinOutMsg {
    Add(Plugin),
}

#[derive(Debug)]
pub struct AddCustomPluginWinInit {
    pub parent: gtk::Window,
}

#[relm4::component(pub)]
impl SimpleComponent for AddCustomPluginWin {
    type Init = AddCustomPluginWinInit;
    type Input = AddCustomPluginWinMsg;
    type Output = AddCustomPluginWinOutMsg;

    view! {
        #[name(win)]
        adw::Dialog {
            set_can_close: true,
            #[wrap(Some)]
            set_child: inner = &adw::ToolbarView {
                set_top_bar_style: adw::ToolbarStyle::Flat,
                set_bottom_bar_style: adw::ToolbarStyle::Flat,
                set_vexpand: true,
                set_hexpand: true,
                add_top_bar: top_bar = &adw::HeaderBar {
                    set_show_end_title_buttons: false,
                    set_show_start_title_buttons: false,
                    pack_start: cancel_btn = &gtk::Button {
                        set_label: "Cancel",
                        add_css_class: "destructive-action",
                        connect_clicked[sender] => move |_| {
                            sender.input(Self::Input::Close)
                        },
                    },
                    pack_end: add_btn = &gtk::Button {
                        set_label: "Add",
                        add_css_class: "suggested-action",
                        #[track = "model.changed(AddCustomPluginWin::can_add())"]
                        set_sensitive: model.can_add,
                        connect_clicked[sender] => move |_| {
                            sender.input(Self::Input::Add)
                        },
                    },
                    #[wrap(Some)]
                    set_title_widget: title_label = &adw::WindowTitle {
                        set_title: "Add Custom Plugin",
                    },
                },
                #[wrap(Some)]
                set_content: content = &adw::PreferencesPage {
                    set_hexpand: true,
                    set_vexpand: true,
                    add: grp = &adw::PreferencesGroup {
                        add: &entry_row(
                            "Plugin Name",
                            "",
                            clone!(
                                #[strong] sender,
                                move |row| sender.input(
                                    Self::Input::OnNameChange(
                                        row.text().to_string()
                                    )
                                )
                            )
                        ),
                        add: &file_row(
                            "Plugin Executable",
                            None,
                            None,
                            Some(model.parent.clone()),
                            clone!(
                                #[strong] sender,
                                move |path_s| sender.input(
                                    Self::Input::OnExecPathChange(path_s)
                                )
                            )
                        ),
                        add: &entry_row(
                            "Plugin Arguments",
                            "",
                            clone!(
                                #[strong] sender,
                                move |row| sender.input(
                                    Self::Input::OnArgsChange(
                                        row.text().to_string()
                                    )
                                )
                            )
                        ),
                    },
                },
            },
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        self.reset();

        match message {
            Self::Input::Present => self.win.as_ref().unwrap().present(Some(&self.parent)),
            Self::Input::Close => {
                self.win.as_ref().unwrap().close();
            }
            Self::Input::Add => {
                if self.plugin.validate() {
                    sender
                        .output(Self::Output::Add(self.plugin.clone()))
                        .expect(SENDER_IO_ERR_MSG);
                    self.win.as_ref().unwrap().close();
                }
            }
            Self::Input::OnNameChange(name) => {
                self.plugin.appid = if !name.is_empty() {
                    format!("{APP_ID}.customPlugin.{name}")
                } else {
                    String::default()
                };
                self.plugin.name = name;
                self.set_can_add(self.plugin.validate());
            }
            Self::Input::OnArgsChange(args) => {
                let args = args.trim().to_string();
                self.plugin.args = if args.is_empty() {
                    None
                } else {
                    // it's fine to have them joined
                    // since they will ultimately be
                    // passed as a joined string
                    Some(vec![args])
                }
            }
            Self::Input::OnExecPathChange(ep) => {
                self.plugin.exec_path = ep.map(PathBuf::from);
                self.set_can_add(self.plugin.validate());
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = Self {
            tracker: 0,
            win: None,
            parent: init.parent,
            can_add: false,
            plugin: Plugin {
                short_description: Some("Custom Plugin".into()),
                ..Default::default()
            },
        };
        let widgets = view_output!();
        model.win = Some(widgets.win.clone());

        ComponentParts { model, widgets }
    }
}
