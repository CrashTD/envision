use crate::{
    ui::factories::client_row_factory::{ClientRowModel, ClientRowModelInit},
    xr_clients::XRClient,
};
use adw::prelude::*;
use relm4::{factory::AsyncFactoryVecDeque, prelude::*};

#[tracker::track]
pub struct ClientsBox {
    has_clients: bool,

    #[tracker::do_not_track]
    client_rows: AsyncFactoryVecDeque<ClientRowModel>,
}

#[derive(Debug)]
pub enum ClientsBoxMsg {
    UpdateClients(Vec<XRClient>),
}

#[relm4::component(pub)]
impl SimpleComponent for ClientsBox {
    type Init = ();
    type Input = ClientsBoxMsg;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            set_vexpand: false,
            set_spacing: 12,
            #[track = "model.changed(Self::has_clients())"]
            set_visible: model.has_clients,
            append: &clients_listbox,
        }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        self.reset();

        match message {
            Self::Input::UpdateClients(clients) => {
                self.set_has_clients(!clients.is_empty());
                let mut guard = self.client_rows.guard();
                let existing_clients: Vec<XRClient> = guard
                    .iter()
                    .filter_map(|optrow| optrow.map(|row| row.client.clone()))
                    .collect();
                let mut new_clients = clients.clone();
                new_clients.retain(|cl| !existing_clients.iter().any(|ecl| ecl == cl));
                // remove stale rows
                {
                    let mut indexes_to_rm: Vec<usize> = Vec::new();
                    for (i, row) in guard.iter().enumerate().rev() {
                        if let Some(row) = row
                            && clients.iter().any(|cl| cl == &row.client)
                        {
                            // row can be retained
                        } else {
                            indexes_to_rm.push(i);
                        }
                    }
                    for i in indexes_to_rm {
                        guard.remove(i);
                    }
                }
                if !self.has_clients {
                    return;
                }
                for cl in new_clients {
                    guard.push_back(ClientRowModelInit { client: cl.clone() });
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let clients_listbox = gtk::ListBox::builder()
            .css_classes(["boxed-list"])
            .selection_mode(gtk::SelectionMode::None)
            .build();

        let model = Self {
            tracker: 0,
            has_clients: false,
            client_rows: AsyncFactoryVecDeque::builder()
                .launch(clients_listbox.clone())
                .detach(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
