use super::{
    factories::wivrn_encoder_group_factory::{
        WivrnEncoderModel, WivrnEncoderModelInit, WivrnEncoderModelOutMsg,
    },
    util::bits_from_mbits,
    wivrn_encoder_presets_win::{
        WivrnEncoderPresetsWin, WivrnEncoderPresetsWinMsg, WivrnEncoderPresetsWinOutMsg,
    },
};
use crate::{
    file_builders::wivrn_config::{
        dump_wivrn_config, get_wivrn_config, WivrnConfEncoder, WivrnConfig,
    },
    ui::{
        preference_rows::{number_entry_row, spin_row, switch_row},
        util::bits_to_mbits,
        wivrn_encoder_presets_win::WivrnEncoderPresetsWinInit,
        ADW_DIALOG_WIDTH,
    },
};
use adw::prelude::*;
use gtk::glib::clone;
use relm4::{factory::AsyncFactoryVecDeque, prelude::*};

#[tracker::track]
pub struct WivrnConfEditor {
    conf: WivrnConfig,
    #[tracker::do_not_track]
    win: Option<adw::Dialog>,
    #[tracker::do_not_track]
    pub encoder_models: Option<AsyncFactoryVecDeque<WivrnEncoderModel>>,
    #[tracker::do_not_track]
    pub scalex_row: Option<adw::SpinRow>,
    #[tracker::do_not_track]
    pub scaley_row: Option<adw::SpinRow>,
    #[tracker::do_not_track]
    bitrate_row: Option<adw::EntryRow>,
    #[tracker::do_not_track]
    wivrn_encoder_presets_win: Option<Controller<WivrnEncoderPresetsWin>>,
    #[tracker::do_not_track]
    parent: gtk::Window,
}

#[derive(Debug)]
pub enum WivrnConfEditorMsg {
    Present,
    Save,
    AddEncoder,
    DeleteEncoder(String),
    TcpOnlyChanged(bool),
    OpenEncoderPresetsWin,
    SetEncoderPreset(Vec<WivrnConfEncoder>),
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
        adw::Dialog {
            set_title: "WiVRn Configuration",
            set_content_width: ADW_DIALOG_WIDTH,
            #[wrap(Some)]
            set_child: tbview = &adw::ToolbarView {
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
                    set_description: "<b>Warning</b>: you likely don't need to change the default configuration, this is for advanced users only. Proceed at your own risk!\n\n<a href=\"https://github.com/WiVRn/WiVRn/blob/master/docs/configuration.md\">WiVRn Configuration Documentation</a>",
                    add: scalegrp = &adw::PreferencesGroup {
                        set_title: "Scale",
                        set_description: Some(concat!("Foveation scale, 1.0 means no foveation. ",
                            "A value too high may overload your headset's decoder.\n",
                            "<b>This does not affect game render resolution. ",
                            "Render resolution is set from within the Android app.</b>",
                        )),
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
                                .and_then(|n| bits_to_mbits(n).map(|mbits| mbits.to_string()))
                                .unwrap_or_default(),
                            false,
                            move |_| {}
                        ) -> adw::EntryRow,
                    },
                    // TODO: add application? conflicts with envision autostart
                    add: networkgrp = &adw::PreferencesGroup {
                        set_title: "Network",
                        add: &switch_row(
                            "TCP Only Mode",
                            Some(concat!(
                                "Only use TCP for communications with the client, ",
                                "this may have increased latency. If off, WiVRn ",
                                "will use both TCP and UDP."
                            )),
                            model.conf.tcp_only,
                            clone!(#[strong] sender, move |_, val| {
                                sender.input(Self::Input::TcpOnlyChanged(val));
                                gtk::glib::Propagation::Proceed
                            }),
                        ),
                    },
                    add: encodersrgp = &adw::PreferencesGroup {
                        set_title: "Encoders",
                        adw::ActionRow {
                            set_title: "Add encoder",
                            add_suffix: presets_btn = &gtk::Button {
                                set_valign: gtk::Align::Center,
                                set_label: "Use Preset",
                                connect_clicked[sender] => move |_| {
                                    sender.input(Self::Input::OpenEncoderPresetsWin)
                                }
                            },
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
                self.win.as_ref().unwrap().present(Some(&self.parent));
            }
            Self::Input::OpenEncoderPresetsWin => {
                self.wivrn_encoder_presets_win
                    .as_ref()
                    .unwrap()
                    .sender()
                    .emit(WivrnEncoderPresetsWinMsg::Present);
            }
            Self::Input::SetEncoderPreset(preset) => {
                self.encoder_models.as_mut().unwrap().guard().clear();
                preset.iter().for_each(|enc| {
                    self.encoder_models.as_mut().unwrap().guard().push_back(
                        WivrnEncoderModelInit {
                            encoder_conf: Some(enc.clone()),
                        },
                    );
                });
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
            Self::Input::TcpOnlyChanged(val) => {
                self.conf.tcp_only = val;
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
            wivrn_encoder_presets_win: None,
            parent: init.root_win,
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

        model.wivrn_encoder_presets_win = Some(
            WivrnEncoderPresetsWin::builder()
                .launch(WivrnEncoderPresetsWinInit {
                    parent_win: widgets.win.clone().upcast(),
                })
                .forward(sender.input_sender(), move |msg| match msg {
                    WivrnEncoderPresetsWinOutMsg::Selected(preset) => {
                        Self::Input::SetEncoderPreset(preset)
                    }
                }),
        );

        ComponentParts { model, widgets }
    }
}
