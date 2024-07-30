use super::wivrn_config::{Codec, Encoder, WivrnConfEncoder};
use lazy_static::lazy_static;

fn wivrn_encoder_presets() -> Vec<(&'static str, &'static str, Vec<WivrnConfEncoder>)> {
    vec![
        (
            "3x VAAPI",
            "Split encoding in 3 slices using VAAPI h265 hardware acceleration.",
            vec![
                WivrnConfEncoder {
                    encoder: Encoder::Vaapi,
                    width: Some(0.5),
                    height: Some(0.25),
                    offset_x: Some(0.0),
                    offset_y: Some(0.0),
                    group: Some(0),
                    codec: Codec::H265,
                    ..Default::default()
                },
                WivrnConfEncoder {
                    encoder: Encoder::Vaapi,
                    width: Some(0.5),
                    height: Some(0.75),
                    offset_x: Some(0.0),
                    offset_y: Some(0.25),
                    group: Some(0),
                    codec: Codec::H265,
                    ..Default::default()
                },
                WivrnConfEncoder {
                    encoder: Encoder::Vaapi,
                    width: Some(0.5),
                    height: Some(1.0),
                    offset_x: Some(0.5),
                    offset_y: Some(0.0),
                    group: Some(0),
                    codec: Codec::H265,
                    ..Default::default()
                },
            ],
        ),
        (
            "2x VAAPI + 1 Software",
            "Split encoding in 3 slices, half the image using VAAPI h265 hardware acceleration, the other half using h264 software encoding. Lowest possible latency at the cost of CPU usage and some quality degradation.",
            vec![
                WivrnConfEncoder {
                    encoder: Encoder::Vaapi,
                    width: Some(0.5),
                    height: Some(0.25),
                    offset_x: Some(0.0),
                    offset_y: Some(0.0),
                    group: Some(0),
                    codec: Codec::H265,
                    ..Default::default()
                },
                WivrnConfEncoder {
                    encoder: Encoder::Vaapi,
                    width: Some(0.5),
                    height: Some(0.75),
                    offset_x: Some(0.0),
                    offset_y: Some(0.25),
                    group: Some(0),
                    codec: Codec::H265,
                    ..Default::default()
                },
                WivrnConfEncoder {
                    encoder: Encoder::X264,
                    width: Some(0.5),
                    height: Some(1.0),
                    offset_x: Some(0.5),
                    offset_y: Some(0.0),
                    group: Some(1),
                    codec: Codec::H264,
                    ..Default::default()
                },
            ],
        ),
    ]
}

lazy_static! {
    pub static ref WIVRN_ENCODER_PRESETS: Vec<(&'static str, &'static str, Vec<WivrnConfEncoder>)> =
        wivrn_encoder_presets();
}
