use super::{
    factories::wivrn_encoder_group_factory::{
        WivrnEncoderModel, WivrnEncoderModelInit, WivrnEncoderModelOutMsg,
    },
    util::bits_from_mbits,
};
use crate::{
    file_builders::wivrn_config::{dump_wivrn_config, get_wivrn_config, WivrnConfig},
    ui::{
        preference_rows::{number_entry_row, spin_row},
        util::bits_to_mbits,
    },
};
use adw::prelude::*;
use relm4::{factory::AsyncFactoryVecDeque, prelude::*};

#[tracker::track]
pub struct WivrnConfEditor {
    conf: WivrnConfig,
    #[tracker::do_not_track]
    win: Option<adw::Window>,
    #[tracker::do_not_track]
    pub encoder_models: Option<AsyncFactoryVecDeque<WivrnEncoderModel>>,
    #[tracker::do_not_track]
    pub scalex_row: Option<adw::SpinRow>,
    #[tracker::do_not_track]
    pub scaley_row: Option<adw::SpinRow>,
    #[tracker::do_not_track]
    bitrate_row: Option<adw::EntryRow>,
}

#[derive(Debug)]
pub enum WivrnConfEditorMsg {
    Present,
    Save,
    AddEncoder,
    DeleteEncoder(String),
}

pub struct WivrnConfEditorInit {
    pub root_win: gtk::Window,
}

#[relm4::component(pub)]
impl SimpleComponent for WivrnConfEditor {
    type Init = WivrnConfEditorInit;
    type Input = WivrnConfEditorMsg;
    type Output = ();

    view! {
        #[name(win)]
        adw::Window {
            set_modal: true,
            set_transient_for: Some(&init.root_win),
            set_title: Some("WiVRn Configuration"),
            set_default_height: 500,
            set_default_width: 600,
            adw::ToolbarView {
                set_top_bar_style: adw::ToolbarStyle::Flat,
                set_hexpand: true,
                set_vexpand: true,
                add_top_bar: top_bar = &adw::HeaderBar {
                    set_vexpand: false,
                    set_show_end_title_buttons: false,
                    set_show_start_title_buttons: false,
                    pack_end: save_btn = &gtk::Button {
                        set_label: "Save",
                        add_css_class: "suggested-action",
                        connect_clicked[sender] => move |_| {
                            sender.input(Self::Input::Save);
                        },
                    },
                    pack_start: cancel_btn = &gtk::Button {
                        set_label: "Cancel",
                        add_css_class: "destructive-action",
                        connect_clicked[win] => move |_| {
                            win.close();
                        }
                    },
                },
                #[wrap(Some)]
                set_content: pref_page = &adw::PreferencesPage {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_description: "<b>Warning</b>: you likely don't need to change the default configuration, this is for advanced users only. Proceed at your own risk!\n\n<a href=\"https://github.com/Meumeu/WiVRn/blob/master/docs/configuration.md\">WiVRn Configuration Documentation</a>",
                    add: scalegrp = &adw::PreferencesGroup {
                        set_title: "Scale",
                        set_description: Some("Render resolution scale. 1.0 is 100%."),
                        add: scalex_row = &spin_row(
                            "Scale X",
                            None,
                            match model.conf.scale {
                                Some([x, _]) => x.into(),
                                None => 1.0,
                            },
                            0.0,
                            1.0,
                            0.01,
                            move |_| {}
                        ) -> adw::SpinRow,
                        add: scaley_row = &spin_row(
                            "Scale Y",
                            None,
                            match model.conf.scale {
                                Some([_, y]) => y.into(),
                                None => 1.0,
                            },
                            0.0,
                            1.0,
                            0.01,
                            move |_| {}
                        ) -> adw::SpinRow,
                    },
                    add: bitrategrp = &adw::PreferencesGroup {
                        set_title: "Bitrate",
                        add: bitrate_row = &number_entry_row(
                            "Bitrate (Mbps)",
                            &model.conf.bitrate
                                .and_then(|n| if let Some(mbits) = bits_to_mbits(n) {
                                    Some(mbits.to_string())
                                } else {
                                    None
                                })
                                .unwrap_or_default(),
                            false,
                            move |_| {}
                        ) -> adw::EntryRow,
                    },
                    add: encodersrgp = &adw::PreferencesGroup {
                        set_title: "Encoders",
                        adw::ActionRow {
                            set_title: "Add encoder",
                            add_suffix: add_encoder_btn = &gtk::Button {
                                set_halign: gtk::Align::Center,
                                set_valign: gtk::Align::Center,
                                add_css_class: "suggested-action",
                                set_icon_name: "list-add-symbolic",
                                set_tooltip_text: Some("Add encoder"),
                                connect_clicked[sender] => move |_| {
                                    sender.input(Self::Input::AddEncoder)
                                }
                            },
                        },
                    },
                }
            }
        }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        self.reset();

        match message {
            Self::Input::Present => {
                self.set_conf(get_wivrn_config());
                self.win.as_ref().unwrap().present();
            }
            Self::Input::Save => {
                let x = self.scalex_row.as_ref().unwrap().adjustment().value();
                let y = self.scaley_row.as_ref().unwrap().adjustment().value();
                self.conf.scale = Some([x as f32, y as f32]);
                self.conf.bitrate = {
                    let txt = self.bitrate_row.as_ref().unwrap().text();
                    if txt.is_empty() {
                        None
                    } else {
                        match txt.parse::<u32>() {
                            Ok(mbits) => bits_from_mbits(mbits),
                            Err(_) => None,
                        }
                    }
                };
                self.conf.encoders = self
                    .encoder_models
                    .as_ref()
                    .unwrap()
                    .iter()
                    .filter(Option::is_some)
                    .map(|m| m.as_ref().unwrap().encoder_conf.clone())
                    .collect();
                dump_wivrn_config(&self.conf);
                self.win.as_ref().unwrap().close();
            }
            Self::Input::AddEncoder => {
                self.encoder_models
                    .as_mut()
                    .unwrap()
                    .guard()
                    .push_back(WivrnEncoderModelInit::default());
            }
            Self::Input::DeleteEncoder(id) => {
                let idx_opt = self
                    .encoder_models
                    .as_ref()
                    .unwrap()
                    .iter()
                    .position(|m_opt| m_opt.is_some_and(|m| m.uid == id));
                if let Some(idx) = idx_opt {
                    self.encoder_models.as_mut().unwrap().guard().remove(idx);
                } else {
                    eprintln!("Couldn't find encoder model with id {id}");
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = Self {
            conf: get_wivrn_config(),
            encoder_models: None,
            win: None,
            scalex_row: None,
            scaley_row: None,
            bitrate_row: None,
            tracker: 0,
        };

        let widgets = view_output!();

        model.scalex_row = Some(widgets.scalex_row.clone());
        model.scaley_row = Some(widgets.scaley_row.clone());
        model.bitrate_row = Some(widgets.bitrate_row.clone());

        let mut encoder_models: AsyncFactoryVecDeque<WivrnEncoderModel> =
            AsyncFactoryVecDeque::builder()
                .launch(widgets.pref_page.clone())
                .forward(sender.input_sender(), |msg| match msg {
                    WivrnEncoderModelOutMsg::Delete(id) => WivrnConfEditorMsg::DeleteEncoder(id),
                });
        for encoder_conf in model.conf.encoders.clone() {
            encoder_models.guard().push_back(WivrnEncoderModelInit {
                encoder_conf: Some(encoder_conf),
            });
        }
        model.encoder_models = Some(encoder_models);

        model.win = Some(widgets.win.clone());

        ComponentParts { model, widgets }
    }
}
