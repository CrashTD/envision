use super::{
    add_custom_plugin_win::{
        AddCustomPluginWin, AddCustomPluginWinInit, AddCustomPluginWinMsg, AddCustomPluginWinOutMsg,
    },
    store_detail::{StoreDetail, StoreDetailMsg, StoreDetailOutMsg},
    store_row_factory::{StoreRowModel, StoreRowModelInit, StoreRowModelMsg, StoreRowModelOutMsg},
    Plugin,
};
use crate::{
    config::PluginConfig,
    downloader::download_file_async,
    ui::{alert::alert, SENDER_IO_ERR_MSG},
};
use adw::prelude::*;
use relm4::{factory::AsyncFactoryVecDeque, prelude::*};
use std::{collections::HashMap, fs::remove_file};
use tracing::{debug, error};

#[tracker::track]
pub struct PluginStore {
    #[tracker::do_not_track]
    win: Option<adw::Window>,
    #[tracker::do_not_track]
    plugin_rows: Option<AsyncFactoryVecDeque<StoreRowModel>>,
    #[tracker::do_not_track]
    details: AsyncController<StoreDetail>,
    #[tracker::do_not_track]
    main_stack: Option<gtk::Stack>,
    #[tracker::do_not_track]
    config_plugins: HashMap<String, PluginConfig>,
    refreshing: bool,
    locked: bool,
    plugins: Vec<Plugin>,
    #[tracker::do_not_track]
    add_custom_plugin_win: Option<Controller<AddCustomPluginWin>>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PluginStoreSignalSource {
    Row,
    Detail,
}

#[derive(Debug)]
pub enum PluginStoreMsg {
    Present,
    Refresh,
    Install(Plugin, relm4::Sender<StoreRowModelMsg>),
    InstallFromDetails(Plugin),
    InstallDownload(Plugin, relm4::Sender<StoreRowModelMsg>),
    Remove(Plugin),
    SetEnabled(PluginStoreSignalSource, Plugin, bool),
    ShowDetails(usize),
    ShowPluginList,
    PresentAddCustomPluginWin,
    AddPluginToConfig(Plugin),
    AddCustomPlugin(Plugin),
}

#[derive(Debug)]
pub struct PluginStoreInit {
    pub config_plugins: HashMap<String, PluginConfig>,
}

#[derive(Debug)]
pub enum PluginStoreOutMsg {
    UpdateConfigPlugins(HashMap<String, PluginConfig>),
}

impl PluginStore {
    fn refresh_plugin_rows(&mut self) {
        let mut guard = self.plugin_rows.as_mut().unwrap().guard();
        guard.clear();
        self.plugins.iter().for_each(|plugin| {
            guard.push_back(StoreRowModelInit {
                plugin: plugin.clone(),
                enabled: self
                    .config_plugins
                    .get(&plugin.appid)
                    .is_some_and(|cp| cp.enabled),
                needs_update: self
                    .config_plugins
                    .get(&plugin.appid)
                    .is_some_and(|cp| cp.plugin.version != plugin.version),
            });
        });
    }
}

#[relm4::component(pub async)]
impl AsyncComponent for PluginStore {
    type Init = PluginStoreInit;
    type Input = PluginStoreMsg;
    type Output = PluginStoreOutMsg;
    type CommandOutput = ();

    view! {
        #[name(win)]
        adw::Window {
            set_title: Some("Plugins"),
            #[name(main_stack)]
            gtk::Stack {
                add_child = &adw::ToolbarView {
                    set_top_bar_style: adw::ToolbarStyle::Flat,
                    add_top_bar: headerbar = &adw::HeaderBar {
                        pack_start: add_custom_plugin_btn = &gtk::Button {
                            set_icon_name: "list-add-symbolic",
                            set_tooltip_text: Some("Add custom plugin"),
                            #[track = "model.changed(PluginStore::refreshing()) || model.changed(PluginStore::locked())"]
                            set_sensitive: !(model.refreshing || model.locked),
                            connect_clicked[sender] => move |_| {
                                sender.input(Self::Input::PresentAddCustomPluginWin)
                            },
                        },
                        pack_end: refreshbtn = &gtk::Button {
                            set_icon_name: "view-refresh-symbolic",
                            set_tooltip_text: Some("Refresh"),
                            #[track = "model.changed(PluginStore::refreshing()) || model.changed(PluginStore::locked())"]
                            set_sensitive: !(model.refreshing || model.locked),
                            connect_clicked[sender] => move |_| {
                                sender.input(Self::Input::Refresh);
                            },
                        },
                    },
                    #[wrap(Some)]
                    set_content: inner = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_hexpand: true,
                        set_vexpand: true,
                        gtk::Stack {
                            set_hexpand: true,
                            set_vexpand: true,
                            add_child = &gtk::ScrolledWindow {
                                set_hscrollbar_policy: gtk::PolicyType::Never,
                                set_hexpand: true,
                                set_vexpand: true,
                                adw::Clamp {
                                    #[name(listbox)]
                                    gtk::ListBox {
                                        #[track = "model.changed(PluginStore::refreshing()) || model.changed(PluginStore::locked())"]
                                        set_sensitive: !(model.refreshing || model.locked),
                                        add_css_class: "boxed-list",
                                        set_valign: gtk::Align::Start,
                                        set_margin_all: 12,
                                        set_selection_mode: gtk::SelectionMode::None,
                                        connect_row_activated[sender] => move |_, row| {
                                            sender.input(
                                                Self::Input::ShowDetails(
                                                    row.index() as usize
                                                )
                                            );
                                        },
                                    }
                                }
                            } -> {
                                set_name: "pluginlist"
                            },
                            add_child = &gtk::Spinner {
                                set_hexpand: true,
                                set_vexpand: true,
                                set_valign: gtk::Align::Center,
                                set_halign: gtk::Align::Center,
                                #[track = "model.changed(PluginStore::refreshing())"]
                                set_spinning: model.refreshing,
                            } -> {
                                set_name: "spinner"
                            },
                            add_child = &adw::StatusPage {
                                set_hexpand: true,
                                set_vexpand: true,
                                set_title: "No Plugins Found",
                                set_description: Some("Make sure you're connected to the internet and refresh"),
                                set_icon_name: Some("application-x-addon-symbolic"),
                            } -> {
                                set_name: "emptystate"
                            },
                            #[track = "model.changed(PluginStore::refreshing()) || model.changed(PluginStore::plugins())"]
                            set_visible_child_name: if model.refreshing {
                                "spinner"
                            } else if model.plugins.is_empty() {
                                "emptystate"
                            } else {
                                "pluginlist"
                            },
                        },
                    }
                } -> {
                    set_name: "listview"
                },
                add_child = &adw::Bin {
                    #[track = "model.changed(PluginStore::refreshing()) || model.changed(PluginStore::locked())"]
                    set_sensitive: !(model.refreshing || model.locked),
                    set_child: Some(details_view),
                } -> {
                    set_name: "detailsview",
                },
                set_visible_child_name: "listview",
            }
        }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        self.reset();

        match message {
            Self::Input::Present => {
                self.win.as_ref().unwrap().present();
                sender.input(Self::Input::Refresh);
            }
            Self::Input::AddCustomPlugin(plugin) => {
                if self.config_plugins.contains_key(&plugin.appid) {
                    alert(
                        "Failed to add custom plugin",
                        Some("A plugin with the same name already exists"),
                        Some(&self.win.as_ref().unwrap().clone().upcast::<gtk::Window>()),
                    );
                    return;
                }
                sender.input(Self::Input::AddPluginToConfig(plugin));
                sender.input(Self::Input::Refresh);
            }
            Self::Input::AddPluginToConfig(plugin) => {
                self.config_plugins
                    .insert(plugin.appid.clone(), PluginConfig::from(&plugin));
                sender
                    .output(Self::Output::UpdateConfigPlugins(
                        self.config_plugins.clone(),
                    ))
                    .expect(SENDER_IO_ERR_MSG);
            }
            Self::Input::PresentAddCustomPluginWin => {
                let add_win = AddCustomPluginWin::builder()
                    .launch(AddCustomPluginWinInit {
                        parent: self.win.as_ref().unwrap().clone().upcast(),
                    })
                    .forward(sender.input_sender(), |msg| match msg {
                        AddCustomPluginWinOutMsg::Add(plugin) => {
                            Self::Input::AddCustomPlugin(plugin)
                        }
                    });
                add_win.sender().emit(AddCustomPluginWinMsg::Present);
                self.add_custom_plugin_win = Some(add_win);
            }
            Self::Input::Refresh => {
                self.set_refreshing(true);
                // TODO: populate from web
                let mut plugins = vec![
	                Plugin {
	                    appid: "com.github.galiser.wlx-overlay-s".into(),
	                    name: "WLX Overlay S".into(),
	                    version: Some("0.6.0".into()),
	                    hompage_url: Some("https://github.com/galister/wlx-overlay-s".into()),
	                    icon_url: Some("https://github.com/galister/wlx-overlay-s/raw/main/wlx-overlay-s.svg".into()),
	                    screenshots: vec![
	                        "https://github.com/galister/wlx-overlay-s/raw/guide/wlx-s.png".into(),
	                    ],
	                    description: Some("A lightweight OpenXR/OpenVR overlay for Wayland and X11 desktops, inspired by XSOverlay.\n\nWlxOverlay-S allows you to access your desktop screens while in VR.\n\nIn comparison to similar overlays, WlxOverlay-S aims to run alongside VR games and experiences while having as little performance impact as possible. The UI appearance and rendering techniques are kept as simple and efficient as possible, while still allowing a high degree of customizability.".into()),
	                    short_description: Some("Access your Wayland/X11 desktop".into()),
	                    exec_url: Some("https://github.com/galister/wlx-overlay-s/releases/download/v0.6/WlxOverlay-S-v0.6-x86_64.AppImage".into()),
	                    exec_path: None,
	                },
                    Plugin {
	                    appid: "org.stardustxr.telescope".into(),
	                    name: "Telescope (Stardust XR)".into(),
	                    version: Some("0.45.0".into()),
	                    hompage_url: Some("https://stardustxr.org/".into()),
	                    icon_url: Some("https://stardustxr.org/img/icon.png".into()),
	                    screenshots: vec![
	                        "https://stardustxr.org/img/carousel/workflow.png".into(),
	                    ],
	                    description: Some("A more intuitive overlay designed for spatial computing from the ground up. Part of the Stardust XR project.\n\nTelescope lets you use your favorite apps directly with your controllers or hands at any time.".into()),
	                    short_description: Some("Use your existing apps spatially and intuitively.".into()),
	                    exec_url: Some("https://github.com/StardustXR/telescope/releases/download/0.45.0/Telescope-x86_64.AppImage".into()),
	                    exec_path: None,
	                },
                ];
                {
                    let appids_from_web = plugins
                        .iter()
                        .map(|p| p.appid.clone())
                        .collect::<Vec<String>>();
                    // add all plugins that are in config but not retrieved
                    plugins.extend(self.config_plugins.values().filter_map(|cp| {
                        if appids_from_web.contains(&cp.plugin.appid) {
                            None
                        } else {
                            Some(Plugin::from(cp))
                        }
                    }));
                }
                self.set_plugins(plugins);
                self.refresh_plugin_rows();
                self.set_refreshing(false);
            }
            Self::Input::InstallFromDetails(plugin) => {
                if let Some(row) = self
                    .plugin_rows
                    .as_mut()
                    .unwrap()
                    .guard()
                    .iter()
                    .find(|row| row.is_some_and(|row| row.plugin.appid == plugin.appid))
                    .flatten()
                {
                    sender.input(Self::Input::Install(plugin, row.input_sender.clone()))
                } else {
                    error!("could not find corresponding listbox row")
                }
            }
            Self::Input::Install(plugin, row_sender) => {
                self.set_locked(true);
                sender.input(Self::Input::InstallDownload(plugin, row_sender))
            }
            Self::Input::InstallDownload(plugin, row_sender) => {
                let mut plugin = plugin.clone();
                match plugin.exec_url.as_ref() {
                    Some(url) => {
                        let exec_path = plugin.canonical_exec_path();
                        if let Err(e) = download_file_async(url, &exec_path).await {
                            alert(
                                "Download failed",
                                Some(&format!(
                                    "Downloading {} {} failed:\n\n{e}",
                                    plugin.name,
                                    plugin
                                        .version
                                        .as_ref()
                                        .unwrap_or(&"(no version)".to_string())
                                )),
                                Some(&self.win.as_ref().unwrap().clone().upcast::<gtk::Window>()),
                            );
                        } else {
                            plugin.exec_path = Some(exec_path);
                            sender.input(Self::Input::AddPluginToConfig(plugin.clone()));
                        }
                    }
                    None => {
                        alert(
                            "Download failed",
                            Some(&format!(
                                "Downloading {} {} failed:\n\nNo executable url provided for this plugin, this is likely a bug!",
                                plugin.name,
                                plugin.version.as_ref().unwrap_or(&"(no version)".to_string()))
                            ),
                            Some(&self.win.as_ref().unwrap().clone().upcast::<gtk::Window>())
                        );
                    }
                };
                row_sender.emit(StoreRowModelMsg::Refresh(true, false));
                self.details
                    .emit(StoreDetailMsg::Refresh(plugin.appid, true, false));
                self.set_locked(false);
            }
            Self::Input::Remove(plugin) => {
                self.set_locked(true);
                if let Some(exec) = plugin.executable() {
                    // delete executable only if it's not a custom plugin
                    if exec.is_file() && plugin.exec_url.is_some() {
                        if let Err(e) = remove_file(&exec) {
                            alert(
                                "Failed removing plugin",
                                Some(&format!(
                                    "Could not remove plugin executable {}:\n\n{e}",
                                    exec.to_string_lossy()
                                )),
                                Some(&self.win.as_ref().unwrap().clone().upcast::<gtk::Window>()),
                            );
                        }
                    }
                }
                self.config_plugins.remove(&plugin.appid);
                self.set_plugins(
                    self.plugins
                        .clone()
                        .into_iter()
                        .filter(|p| !(p.appid == plugin.appid && p.exec_url.is_none()))
                        .collect(),
                );
                sender
                    .output(Self::Output::UpdateConfigPlugins(
                        self.config_plugins.clone(),
                    ))
                    .expect(SENDER_IO_ERR_MSG);
                self.refresh_plugin_rows();
                self.details
                    .emit(StoreDetailMsg::Refresh(plugin.appid, false, false));
                self.set_locked(false);
            }
            Self::Input::SetEnabled(signal_sender, plugin, enabled) => {
                if let Some(cp) = self.config_plugins.get_mut(&plugin.appid) {
                    cp.enabled = enabled;
                    if signal_sender == PluginStoreSignalSource::Detail {
                        if let Some(row) = self
                            .plugin_rows
                            .as_mut()
                            .unwrap()
                            .guard()
                            .iter()
                            .find(|row| row.is_some_and(|row| row.plugin.appid == plugin.appid))
                            .flatten()
                        {
                            row.input_sender.emit(StoreRowModelMsg::Refresh(
                                enabled,
                                cp.plugin.version != plugin.version,
                            ));
                        } else {
                            error!("could not find corresponding listbox row")
                        }
                    }
                    if signal_sender == PluginStoreSignalSource::Row {
                        self.details.emit(StoreDetailMsg::Refresh(
                            plugin.appid,
                            enabled,
                            cp.plugin.version != plugin.version,
                        ));
                    }
                } else {
                    debug!(
                        "failed to set plugin {} enabled: could not find in hashmap",
                        plugin.appid
                    )
                }
                sender
                    .output(Self::Output::UpdateConfigPlugins(
                        self.config_plugins.clone(),
                    ))
                    .expect(SENDER_IO_ERR_MSG);
            }
            // we use index here because it's the listbox not the row that can
            // send this signal, so I don't directly have the plugin object
            Self::Input::ShowDetails(index) => {
                if let Some(plugin) = self.plugins.get(index) {
                    self.details.sender().emit(StoreDetailMsg::SetPlugin(
                        plugin.clone(),
                        self.config_plugins
                            .get(&plugin.appid)
                            .is_some_and(|cp| cp.enabled),
                        self.config_plugins
                            .get(&plugin.appid)
                            .is_some_and(|cp| cp.plugin.version != plugin.version),
                    ));
                    self.main_stack
                        .as_ref()
                        .unwrap()
                        .set_visible_child_name("detailsview");
                } else {
                    error!("plugins list index out of range!")
                }
            }
            Self::Input::ShowPluginList => {
                self.main_stack
                    .as_ref()
                    .unwrap()
                    .set_visible_child_name("listview");
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let mut model = Self {
            tracker: 0,
            refreshing: false,
            locked: false,
            win: None,
            plugins: Vec::default(),
            plugin_rows: None,
            details: StoreDetail::builder()
                .launch(())
                .forward(sender.input_sender(), move |msg| match msg {
                    StoreDetailOutMsg::GoBack => Self::Input::ShowPluginList,
                    StoreDetailOutMsg::Install(plugin) => Self::Input::InstallFromDetails(plugin),
                    StoreDetailOutMsg::Remove(plugin) => Self::Input::Remove(plugin),
                    StoreDetailOutMsg::SetEnabled(plugin, enabled) => {
                        Self::Input::SetEnabled(PluginStoreSignalSource::Detail, plugin, enabled)
                    }
                }),
            config_plugins: init.config_plugins,
            main_stack: None,
            add_custom_plugin_win: None,
        };

        let details_view = model.details.widget();

        let widgets = view_output!();

        model.win = Some(widgets.win.clone());
        model.plugin_rows = Some(
            AsyncFactoryVecDeque::builder()
                .launch(widgets.listbox.clone())
                .forward(sender.input_sender(), move |msg| match msg {
                    StoreRowModelOutMsg::Install(appid, row_sender) => {
                        Self::Input::Install(appid, row_sender)
                    }
                    StoreRowModelOutMsg::Remove(appid) => Self::Input::Remove(appid),
                    StoreRowModelOutMsg::SetEnabled(plugin, enabled) => {
                        Self::Input::SetEnabled(PluginStoreSignalSource::Row, plugin, enabled)
                    }
                }),
        );
        model.main_stack = Some(widgets.main_stack.clone());

        AsyncComponentParts { model, widgets }
    }
}
