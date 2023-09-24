use crate::profile::Profile;

pub fn get_devs(prof: &Profile) {
    if let Ok(monado) = libmonado_rs::Monado::create(prof.libmonado_so().unwrap()) {
        if let Ok(devs) = monado.devices() {
            for dev in devs {
                println!(">>> {}", dev.name);
            }
        }
    }
}
