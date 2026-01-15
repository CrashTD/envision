use libmonado::ClientLogic;
use std::rc::Rc;

#[derive(Clone)]
pub struct XRClient {
    pub id: u32,
    pub name: String,
    // client reference not necessary right now
    // client: libmonado::ClientRc,
}

impl std::fmt::Debug for XRClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("XRClient")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

impl PartialEq for XRClient {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.name == other.name
    }
}
impl Eq for XRClient {}

impl XRClient {
    pub fn from_libmonado(monado: Rc<libmonado::Monado>) -> Vec<Self> {
        if let Ok(monado_clients) = libmonado::Monado::clients_rc(&monado) {
            monado_clients
                .into_iter()
                .map(|mut cl| {
                    Self {
                        id: cl.id(),
                        name: cl.name().unwrap_or("<No name>".into()),
                        // client: cl,
                    }
                })
                .collect()
        } else {
            Vec::default()
        }
    }
}
