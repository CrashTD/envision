use super::{
    add_custom_plugin_win::{
        AddCustomPluginWin, AddCustomPluginWinInit, AddCustomPluginWinMsg, AddCustomPluginWinOutMsg,
    },
    refresh_plugins,
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
    /// sets state and calls DoRefresh
    Refresh,
    /// called by Refresh
    DoRefresh,
    Install(Plugin),
    InstallDownload(Vec<Plugin>),
    Remove(Plugin),
    SetEnabled(PluginStoreSignalSource, Plugin, bool),
    ShowDetails(usize),
    ShowPluginList,
    PresentAddCustomPluginWin,
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

    fn add_plugin_to_config(&mut self, sender: &relm4::AsyncComponentSender<Self>, plugin: Plugin) {
        self.config_plugins
            .insert(plugin.appid.clone(), PluginConfig::from(&plugin));
        sender
            .output(PluginStoreOutMsg::UpdateConfigPlugins(
                self.config_plugins.clone(),
            ))
            .expect(SENDER_IO_ERR_MSG);
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
                                set_width_request: 32,
                                set_height_request: 32,
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
                self.add_plugin_to_config(&sender, plugin);
                sender.input(Self::Input::Refresh);
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
                sender.input(Self::Input::DoRefresh);
            }
            Self::Input::DoRefresh => {
                let mut plugins = match refresh_plugins().await {
                    Err(e) => {
                        error!("failed to refresh plugins: {e}");
                        Vec::new()
                    }
                    Ok(results) => results
                        .into_iter()
                        .filter_map(|res| match res {
                            Ok(plugin) => Some(plugin),
                            Err(e) => {
                                error!("failed to refresh single plugin manifest: {e}");
                                None
                            }
                        })
                        .collect(),
                };
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
            Self::Input::Install(plugin) => {
                self.set_locked(true);
                let mut plugins = vec![plugin];
                for dep in plugins[0].dependencies.clone().unwrap_or_default() {
                    if let Some(dep_plugin) = self
                        .plugins
                        .iter()
                        .find(|plugin| plugin.appid == dep)
                        .cloned()
                    {
                        plugins.push(dep_plugin);
                    } else {
                        error!(
                            "unable to find dependency for {}: {}",
                            plugins[0].appid, dep
                        );
                        alert(
                            "Missing dependencies",
                            Some(&format!(
                                "{} depends on unknown plugin {}",
                                plugins[0].name, dep
                            )),
                            Some(&self.win.clone().unwrap().upcast::<gtk::Window>()),
                        );
                        return;
                    }
                }
                sender.input(Self::Input::InstallDownload(plugins))
            }
            Self::Input::InstallDownload(plugins) => {
                for plugin in plugins {
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
                                    Some(
                                        &self.win.as_ref().unwrap().clone().upcast::<gtk::Window>(),
                                    ),
                                );
                            } else {
                                plugin.exec_path = Some(exec_path);
                                if let Err(e) = plugin.enable() {
                                    error!("failed to enable plugin {}: {e}", plugin.appid);
                                    alert(
                                        "Failed to enable plugin",
                                        Some(&e.to_string()),
                                        Some(&self.win.clone().unwrap().upcast::<gtk::Window>()),
                                    );
                                    return;
                                }
                                self.add_plugin_to_config(&sender, plugin.clone());
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
                    self.refresh_plugin_rows();
                    self.details
                        .emit(StoreDetailMsg::Refresh(plugin.appid, true, false));
                }
                self.set_locked(false);
            }
            Self::Input::Remove(plugin) => {
                self.set_locked(true);
                if let Err(e) = plugin.disable() {
                    error!("failed to disable plugin {}: {e}", plugin.appid);
                    alert(
                        "Failed to disable plugin",
                        Some(&e.to_string()),
                        Some(&self.win.clone().unwrap().upcast::<gtk::Window>()),
                    );
                    return;
                }
                if let Some(exec) = plugin.executable() {
                    // delete executable only if it's not a custom plugin
                    if exec.is_file()
                        && plugin.exec_url.is_some()
                        && let Err(e) = remove_file(&exec)
                    {
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
                    if let Err(e) = plugin.set_enabled(enabled) {
                        error!(
                            "failed to {} plugin {}: {e}",
                            if enabled { "enable" } else { "disable" },
                            plugin.appid
                        );
                        alert(
                            &format!(
                                "Failed to {} plugin",
                                if enabled { "enable" } else { "disable" }
                            ),
                            Some(&e.to_string()),
                            Some(&self.win.clone().unwrap().upcast::<gtk::Window>()),
                        );
                        return;
                    }
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
                    StoreDetailOutMsg::Install(plugin) => Self::Input::Install(plugin),
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
                    StoreRowModelOutMsg::Install(appid) => Self::Input::Install(appid),
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
