use crate::{
    file_builders::wivrn_config::{Codec, Encoder, WivrnConfEncoder},
    ui::{
        SENDER_IO_ERR_MSG,
        preference_rows::{combo_row, spin_row},
    },
};
use relm4::{AsyncFactorySender, adw::prelude::*, factory::AsyncFactoryComponent, prelude::*};
use uuid::Uuid;

#[derive(Debug)]
pub struct WivrnEncoderModel {
    pub encoder_conf: WivrnConfEncoder,
    pub uid: String,
}

#[derive(Debug)]
pub enum WivrnEncoderModelMsg {
    EncoderChanged(u32),
    CodecChanged(u32),
    WidthChanged(Option<f32>),
    HeightChanged(Option<f32>),
    OffsetXChanged(Option<f32>),
    OffsetYChanged(Option<f32>),
    GroupChanged(Option<i32>),
    Delete,
}

#[derive(Debug)]
pub enum WivrnEncoderModelOutMsg {
    Delete(String),
}

#[derive(Debug, Default)]
pub struct WivrnEncoderModelInit {
    pub encoder_conf: Option<WivrnConfEncoder>,
}

#[relm4::factory(async pub)]
impl AsyncFactoryComponent for WivrnEncoderModel {
    type Init = WivrnEncoderModelInit;
    type Input = WivrnEncoderModelMsg;
    type Output = WivrnEncoderModelOutMsg;
    type CommandOutput = ();
    type ParentWidget = adw::PreferencesPage;

    view! {
        root = adw::PreferencesGroup {
            set_title: "Encoder",
            #[wrap(Some)]
            set_header_suffix: delete_btn = &gtk::Button {
                set_label: "Delete",
                add_css_class: "destructive-action",
                connect_clicked[sender] => move |_| {
                    sender.input(Self::Input::Delete)
                },
            },
            add: encoder_combo = &combo_row(
                "Encoder",
                Some("x264: CPU based h264 encoding\nNVEnc: Nvidia GPU encoding\nVAAPI: Intel or AMD GPU encoding\nVulkan: Vulkan Video encoding (All GPUs, H264/H265 only)"),
                &self.encoder_conf.encoder.to_string(),
                Encoder::iter().map(Encoder::to_string).collect::<Vec<String>>(),
                {
                    let sender = sender.clone();
                    move |row| {
                        sender.input(Self::Input::EncoderChanged(row.selected()));
                    }
                }
            ) -> adw::ComboRow,
            add: codec_combo = &combo_row(
                "Codec",
                None,
                &self.encoder_conf.codec.to_string(),
                Codec::iter().map(Codec::to_string).collect::<Vec<String>>(),
                {
                    let sender = sender.clone();
                    move |row| {
                        sender.input(Self::Input::CodecChanged(row.selected()));
                    }
                }
            ) -> adw::ComboRow,
            add: width_row = &spin_row(
                "Width",
                None,
                self.encoder_conf.width.unwrap_or(1.0).into(),
                0.0,
                1.0,
                0.01,
                {
                    let sender = sender.clone();
                    move |adj| {
                        sender.input(Self::Input::WidthChanged(
                            Some(adj.value() as f32)
                        ));
                    }
                }
            ) -> adw::SpinRow,
            add: height_row = &spin_row(
                "Height",
                None,
                self.encoder_conf.height.unwrap_or(1.0).into(),
                0.0,
                1.0,
                0.01,
                {
                    let sender = sender.clone();
                    move |adj| {
                        sender.input(Self::Input::HeightChanged(
                            Some(adj.value() as f32)
                        ));
                    }
                }
            ) -> adw::SpinRow,
            add: offset_x_row = &spin_row(
                "Offset X",
                None,
                self.encoder_conf.offset_x.unwrap_or(0.0).into(),
                0.0,
                1.0,
                0.01,
                {
                    let sender = sender.clone();
                    move |adj| {
                        sender.input(Self::Input::OffsetXChanged(
                            Some(adj.value() as f32)
                        ));
                    }
                }
            ) -> adw::SpinRow,
            add: offset_y_row = &spin_row(
                "Offset Y",
                None,
                self.encoder_conf.offset_y.unwrap_or(0.0).into(),
                0.0,
                1.0,
                0.01,
                {
                    let sender = sender.clone();
                    move |adj| {
                        sender.input(Self::Input::OffsetYChanged(
                            Some(adj.value() as f32)
                        ));
                    }
                }
            ) -> adw::SpinRow,
            add: group_row = &spin_row(
                "Group",
                None,
                self.encoder_conf.group.unwrap_or(0).into(),
                0.0,
                99999.0,
                1.0,
                {
                    let sender = sender.clone();
                    move |adj| {
                        sender.input(Self::Input::GroupChanged(
                            Some(adj.value().trunc() as i32)
                        ));
                    }
                }
            ) -> adw::SpinRow,
        }
    }

    async fn update(&mut self, message: Self::Input, sender: AsyncFactorySender<Self>) {
        match message {
            Self::Input::EncoderChanged(idx) => {
                self.encoder_conf.encoder = Encoder::as_vec().get(idx as usize).unwrap().clone();
            }
            Self::Input::CodecChanged(idx) => {
                self.encoder_conf.codec = Codec::as_vec().get(idx as usize).unwrap().clone();
            }
            Self::Input::WidthChanged(val) => {
                self.encoder_conf.width = val;
            }
            Self::Input::HeightChanged(val) => {
                self.encoder_conf.height = val;
            }
            Self::Input::OffsetXChanged(val) => {
                self.encoder_conf.offset_x = val;
            }
            Self::Input::OffsetYChanged(val) => {
                self.encoder_conf.offset_y = val;
            }
            Self::Input::GroupChanged(val) => {
                self.encoder_conf.group = val;
            }
            Self::Input::Delete => {
                sender
                    .output(Self::Output::Delete(self.uid.clone()))
                    .expect(SENDER_IO_ERR_MSG);
            }
        }
    }

    async fn init_model(
        init: Self::Init,
        _index: &DynamicIndex,
        _sender: AsyncFactorySender<Self>,
    ) -> Self {
        Self {
            encoder_conf: init.encoder_conf.unwrap_or_default(),
            uid: Uuid::new_v4().to_string(),
        }
    }
}
