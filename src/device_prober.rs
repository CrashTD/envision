use std::fmt::Display;

use expect_dialog::ExpectDialog;

#[derive(Debug, PartialEq, Eq)]
pub enum PhysicalXRDevice {
    ValveIndex,
    HTCVive,
    HTCVivePro,
    HTCViveCosmos,
    HTCVivePro2,
    OculusRift,
    OculusRiftS,
    PlayStationVR,
    PlayStationVR2,
    Pimax4K,
    Pimax5KPlus,
    Pimax8K,
    DellVisor,
    AcerAH101,
    HPWMR,
    HPReverbG1,
    HPReverbG2,
    LenovoExplorer,
    SamsungOdyssey,
    SamsungOdysseyPlus,
    VarjoVR1,
    VarjoAero,
    VarjoVR2,
    VarjoVR3,
    RazerHydra,
}

impl Display for PhysicalXRDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::ValveIndex => "Valve Index",
            Self::HTCVive => "HTC Vive",
            Self::HTCVivePro => "HTC Vive Pro",
            Self::HTCViveCosmos => "HTC Vive Cosmos",
            Self::HTCVivePro2 => "HTC Vive Pro2",
            Self::OculusRift => "Oculus Rift",
            Self::OculusRiftS => "Oculus Rift S",
            Self::PlayStationVR => "PlayStation Vr",
            Self::PlayStationVR2 => "PlayStation Vr2",
            Self::Pimax4K => "Pimax 4K",
            Self::Pimax5KPlus => "Pimax 5K Plus",
            Self::Pimax8K => "Pimax 8K",
            Self::DellVisor => "Dell Visor",
            Self::AcerAH101 => "Acer AH101",
            Self::HPWMR => "HP WMR",
            Self::HPReverbG1 => "HP Reverb G1",
            Self::HPReverbG2 => "HP Reverb G2",
            Self::LenovoExplorer => "Lenovo Explorer",
            Self::SamsungOdyssey => "Samsung Odyssey",
            Self::SamsungOdysseyPlus => "Samsung Odyssey Plus",
            Self::VarjoVR1 => "Varjo VR1",
            Self::VarjoAero => "Varjo Aero",
            Self::VarjoVR2 => "Varjo VR2",
            Self::VarjoVR3 => "Varjo VR3",
            Self::RazerHydra => "Razer Hydra",
        })
    }
}

/**
* Returns a Vec of tuples, each representing (vendor_id, product_id)
*/
fn list_usb_devs() -> Vec<(u16, u16)> {
    let ctx = libusb::Context::new().expect_dialog("Failed to create libusb context");
    let devs = ctx.devices().expect_dialog("Failed to list devices from libusb context");

    devs.iter().filter_map(|dev| 
        match dev.device_descriptor() {
            Err(_) => None,
            Ok(desc) => Some((desc.vendor_id(), desc.product_id())),
        }
    ).collect()
}

pub fn get_xr_usb_devices() -> Vec<PhysicalXRDevice> {
    list_usb_devs().into_iter().filter_map(|t| match t {
        (0x28de, 0x2102) => Some(PhysicalXRDevice::ValveIndex),
        (0x0bb4, 0x2c87) => Some(PhysicalXRDevice::HTCVive),
        (0x0bb4, 0x0309) => Some(PhysicalXRDevice::HTCVivePro),
        (0x0bb4, 0x0313) => Some(PhysicalXRDevice::HTCViveCosmos),
        (0x0bb4, 0x0342) => Some(PhysicalXRDevice::HTCVivePro2),
        (0x2833, 0x0031) => Some(PhysicalXRDevice::OculusRift),
        (0x2833, 0x0051) => Some(PhysicalXRDevice::OculusRiftS),
        (0x1532, 0x0300) => Some(PhysicalXRDevice::RazerHydra),
        (0x0483, 0x0021) => Some(PhysicalXRDevice::Pimax4K),
        (0x054c, 0x09af) => Some(PhysicalXRDevice::PlayStationVR),
        (0x03f0, 0x0c6a) => Some(PhysicalXRDevice::HPReverbG1),
        (0x03f0, 0x0580) => Some(PhysicalXRDevice::HPReverbG2),
        _ => None,
    }).collect()
}
