use adw::prelude::*;
use relm4::prelude::*;

use crate::file_builders::{
    wivrn_config::WivrnConfEncoder, wivrn_encoder_presets::WIVRN_ENCODER_PRESETS,
};

use super::SENDER_IO_ERR_MSG;

#[tracker::track]
pub struct WivrnEncoderPresetsWin {
    win: Option<adw::Window>,
}

#[derive(Debug)]
pub enum WivrnEncoderPresetsWinMsg {
    Present,
    Selected(Vec<WivrnConfEncoder>),
}

#[derive(Debug)]
pub enum WivrnEncoderPresetsWinOutMsg {
    Selected(Vec<WivrnConfEncoder>),
}

pub struct WivrnEncoderPresetsWinInit {
    pub parent_win: gtk::Window,
}

#[relm4::component(pub)]
impl SimpleComponent for WivrnEncoderPresetsWin {
    type Init = WivrnEncoderPresetsWinInit;
    type Input = WivrnEncoderPresetsWinMsg;
    type Output = WivrnEncoderPresetsWinOutMsg;

    view! {
        #[name(win)]
        adw::Window {
            set_modal: true,
            set_hide_on_close: true,
            set_transient_for: Some(&init.parent_win),
            set_title: Some("WiVRn Encoder Presets"),
            set_default_height: 400,
            set_default_width: 300,
            adw::ToolbarView {
                set_top_bar_style: adw::ToolbarStyle::Flat,
                set_hexpand: true,
                set_vexpand: true,
                add_top_bar: top_bar = &adw::HeaderBar {
                    set_vexpand: false,
                },
                #[wrap(Some)]
                set_content: prefpage = &adw::PreferencesPage {
                    set_title: "Encoder Presets",
                    add: prefgrp = &adw::PreferencesGroup {

                    }
                }
            },
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        self.reset();

        match message {
            Self::Input::Present => {
                self.win.as_ref().unwrap().present();
            }
            Self::Input::Selected(preset) => {
                sender
                    .output(Self::Output::Selected(preset))
                    .expect(SENDER_IO_ERR_MSG);
                self.win.as_ref().unwrap().close();
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = Self {
            win: None,
            tracker: 0,
        };

        let widgets = view_output!();

        model.win = Some(widgets.win.clone());

        WIVRN_ENCODER_PRESETS
            .iter()
            .for_each(|(name, desc, enc_preset)| {
                let row = adw::ActionRow::builder()
                    .title(*name)
                    .subtitle_lines(0)
                    .subtitle(*desc)
                    .activatable(true)
                    .build();
                let sndr = sender.clone();
                row.connect_activated(move |_| {
                    sndr.input(Self::Input::Selected(enc_preset.clone()))
                });
                widgets.prefgrp.add(&row);
            });

        ComponentParts { model, widgets }
    }
}
