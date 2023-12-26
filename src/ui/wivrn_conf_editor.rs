use crate::file_builders::wivrn_config::{
    dump_wivrn_config, get_wivrn_config, Encoder, WivrnConfig,
};
use adw::prelude::*;
use relm4::prelude::*;

#[tracker::track]
pub struct WivrnConfEditor {
    conf: WivrnConfig,
    #[tracker::do_not_track]
    win: Option<adw::Window>,
    #[tracker::do_not_track]
    scalex_entry: Option<adw::EntryRow>,
    #[tracker::do_not_track]
    scaley_entry: Option<adw::EntryRow>,
    #[tracker::do_not_track]
    encoder_combo: Option<adw::ComboRow>,
    #[tracker::do_not_track]
    bitrate_entry: Option<adw::EntryRow>,
}

#[derive(Debug)]
pub enum WivrnConfEditorMsg {
    Present,
    Save,
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
                    pack_end: save_btn = &gtk::Button {
                        set_label: "Save",
                        add_css_class: "suggested-action",
                        connect_clicked[sender] => move |_| {
                            sender.input(Self::Input::Save);
                        },
                    },
                },
                #[wrap(Some)]
                set_content: pref_page = &adw::PreferencesPage {
                    set_hexpand: true,
                    set_vexpand: true,
                    add: scalegrp = &adw::PreferencesGroup {
                        set_title: "Scale",
                        set_description: Some("Render resolution scale. 1.0 is 100%."),
                        #[name(scalex_entry)]
                        adw::EntryRow {
                            set_title: "Scale X",
                            #[track = "model.changed(Self::conf())"]
                            set_text: match model.conf.scale {
                                Some([x, _]) => x.to_string(),
                                None => "".to_string(),
                            }.as_str(),
                            set_input_purpose: gtk::InputPurpose::Number,
                        },
                        #[name(scaley_entry)]
                        adw::EntryRow {
                            set_title: "Scale Y",
                            #[track = "model.changed(Self::conf())"]
                            set_text: match model.conf.scale {
                                Some([_, y]) => y.to_string(),
                                None => "".to_string(),
                            }.as_str(),
                            set_input_purpose: gtk::InputPurpose::Number,
                        },
                    },
                    add: encgrp = &adw::PreferencesGroup {
                        set_title: "Encoder",
                        #[name(encoder_combo)]
                        adw::ComboRow {
                            set_title: "Encoder",
                            set_subtitle: "x264: CPU based h264 encoding\n\nNVEnc: Nvidia GPU encoding\n\nVAAPI: Intel or AMD GPU encoding",
                            set_model: Some(&gtk::StringList::new(
                                Encoder::iter()
                                    .map(Encoder::to_string)
                                    .collect::<Vec<String>>()
                                    .iter()
                                    .map(String::as_str)
                                    .collect::<Vec<&str>>()
                                    .as_slice()
                            )),
                            #[track = "model.changed(Self::conf())"]
                            set_selected: model.conf.encoders.get(0).unwrap().encoder.as_number(),
                        },
                        #[name(bitrate_entry)]
                        adw::EntryRow {
                            set_title: "Bitrate",
                            #[track = "model.changed(Self::conf())"]
                            set_text: match model.conf.encoders.get(0).unwrap().bitrate {
                                Some(br) => br.to_string(),
                                None => "".to_string()
                            }.as_str(),
                            set_input_purpose: gtk::InputPurpose::Number,
                        },
                    },
                    add: save_grp = &adw::PreferencesGroup {
                        add: save_box = &gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_hexpand: true,
                            gtk::Button {
                                set_halign: gtk::Align::Center,
                                set_label: "Save",
                                add_css_class: "pill",
                                add_css_class: "suggested-action",
                                connect_clicked[sender] => move |_| {
                                    sender.input(Self::Input::Save);
                                },
                            },
                        }
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
                let scalex = self.scalex_entry.as_ref().unwrap().text().parse::<f32>();
                let scaley = self.scaley_entry.as_ref().unwrap().text().parse::<f32>();
                if scalex.is_ok() && scaley.is_ok() {
                    self.conf.scale = Some([*scalex.as_ref().unwrap(), *scaley.as_ref().unwrap()]);
                }
                if scalex.is_ok() || scaley.is_ok() {
                    let scale = scalex.unwrap_or(scaley.unwrap());
                    self.conf.scale = Some([scale, scale]);
                } else {
                    self.conf.scale = None
                }

                let mut enc = self.conf.encoders.remove(0);
                let bitrate = self.bitrate_entry.as_ref().unwrap().text().parse::<u32>();
                if let Ok(br) = bitrate {
                    enc.bitrate = Some(br);
                }
                let encoders = Encoder::as_vec();
                let encoder =
                    encoders.get(self.encoder_combo.as_ref().unwrap().selected() as usize);
                if let Some(e) = encoder {
                    enc.encoder = e.clone();
                }
                self.conf.encoders.insert(0, enc);

                dump_wivrn_config(&self.conf);
                self.win.as_ref().unwrap().close();
            }
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = Self {
            conf: get_wivrn_config(),
            win: None,
            scalex_entry: None,
            scaley_entry: None,
            bitrate_entry: None,
            encoder_combo: None,
            tracker: 0,
        };

        let widgets = view_output!();

        model.win = Some(widgets.win.clone());
        model.scalex_entry = Some(widgets.scalex_entry.clone());
        model.scaley_entry = Some(widgets.scaley_entry.clone());
        model.bitrate_entry = Some(widgets.bitrate_entry.clone());
        model.encoder_combo = Some(widgets.encoder_combo.clone());

        ComponentParts { model, widgets }
    }
}
