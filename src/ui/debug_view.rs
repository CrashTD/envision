use crate::log_level::LogLevel;
use crate::log_parser::MonadoLog;
use gtk::prelude::*;
use relm4::prelude::*;
use relm4::{ComponentSender, SimpleComponent};
use sourceview5::prelude::*;

#[derive(Debug)]
pub enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Debug)]
pub enum DebugViewMsg {
    LogUpdated(Vec<String>),
    ClearLog,
    EnableDebugViewChanged(bool),
    FilterLog(SearchDirection),
    LogLevelChanged(LogLevel),
}

#[tracker::track]
pub struct DebugView {
    #[tracker::do_not_track]
    log: Vec<String>,
    #[tracker::do_not_track]
    textbuf: sourceview5::Buffer,
    #[tracker::do_not_track]
    textview: Option<sourceview5::View>,
    #[tracker::do_not_track]
    searchbar: Option<gtk::SearchBar>,
    #[tracker::do_not_track]
    search_entry: Option<gtk::SearchEntry>,
    #[tracker::do_not_track]
    dropdown: Option<gtk::DropDown>,
    #[tracker::do_not_track]
    scrolledwin: Option<gtk::ScrolledWindow>,
    #[tracker::do_not_track]
    search_ctx: sourceview5::SearchContext,
    #[tracker::do_not_track]
    search_settings: sourceview5::SearchSettings,
    #[tracker::do_not_track]
    search_mark: Option<gtk::TextMark>,
    #[tracker::do_not_track]
    log_level: LogLevel,

    enable_debug_view: bool,
}

pub struct DebugViewInit {
    pub enable_debug_view: bool,
}

#[relm4::component(pub)]
impl SimpleComponent for DebugView {
    type Init = DebugViewInit;
    type Input = DebugViewMsg;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            set_vexpand: true,
            #[track = "model.changed(DebugView::enable_debug_view())"]
            set_visible: model.enable_debug_view,
            gtk::WindowHandle {
                set_hexpand: true,
                set_vexpand: false,
                adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget: title_label = &gtk::Label {
                        set_label: "Debug View",
                        add_css_class: "title",
                    },
                    pack_end: search_toggle = &gtk::ToggleButton {
                        set_icon_name: "edit-find-symbolic",
                        set_tooltip_text: Some("Filter Log"),
                    },
                    pack_start: log_level_dropdown = &gtk::DropDown::from_strings(
                        LogLevel::iter()
                            .map(|lvl| lvl.to_string())
                            .collect::<Vec<String>>()
                            .iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<&str>>()
                            .as_slice()
                    ),
                },
            },
            #[name(searchbar)]
            gtk::SearchBar {
                set_hexpand: true,
                #[chain(flags(gtk::glib::BindingFlags::BIDIRECTIONAL).build())]
                bind_property: ("search-mode-enabled", &search_toggle, "active"),
                #[wrap(Some)]
                set_child: searchbox = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    add_css_class: "linked",
                    set_hexpand: true,
                    #[name(search_entry)]
                    gtk::SearchEntry {
                        set_hexpand: true,
                        connect_changed[sender] => move |_| {
                            sender.input(Self::Input::FilterLog(SearchDirection::Forward));
                        },
                        connect_activate[sender] => move |_| {
                            sender.input(Self::Input::FilterLog(SearchDirection::Forward));
                        },
                    },
                    gtk::Button {
                        set_icon_name: "go-up-symbolic",
                        set_tooltip_text: Some("Previous Match"),
                        connect_clicked[sender] => move |_| {
                            sender.input(Self::Input::FilterLog(SearchDirection::Backward))
                        },
                    },
                    gtk::Button {
                        set_icon_name: "go-down-symbolic",
                        set_tooltip_text: Some("Next Match"),
                        connect_clicked[sender] => move |_| {
                            sender.input(Self::Input::FilterLog(SearchDirection::Forward))
                        },
                    },
                },
                connect_entry: &search_entry,
            },
            #[name(scrolledwin)]
            gtk::ScrolledWindow {
                set_hexpand: true,
                set_vexpand: true,
                #[name(textview)]
                sourceview5::View {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_editable: false,
                    set_monospace: true,
                    set_buffer: Some(&model.textbuf),
                },
            }
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        self.reset();

        match message {
            Self::Input::LogLevelChanged(lvl) => {
                self.log_level = lvl;
                let log = self.log.clone();
                self.log = vec![];
                self.textbuf.set_text("");
                sender.input(Self::Input::LogUpdated(log));
            }
            Self::Input::FilterLog(direction) => {
                let searchbar = self.searchbar.as_ref().unwrap().clone();
                let search_entry = self.search_entry.as_ref().unwrap().clone();
                let search_text = search_entry.text().to_string();
                if searchbar.is_search_mode() && !search_text.is_empty() {
                    self.search_settings
                        .set_search_text(Some(search_text.as_str()));
                    self.search_mark = Some(self.textbuf.get_insert());
                    let mut iter = self
                        .textbuf
                        .iter_at_mark(self.search_mark.as_ref().unwrap());
                    iter.forward_char();
                    let search_res = match direction {
                        SearchDirection::Forward => self.search_ctx.forward(&iter),
                        SearchDirection::Backward => self.search_ctx.backward(&iter),
                    };
                    match search_res {
                        None => {
                            // TODO: mark search entry red
                        }
                        Some((start, end, _)) => {
                            self.textbuf.move_mark(
                                self.search_mark.as_ref().unwrap(),
                                match direction {
                                    SearchDirection::Forward => &end,
                                    SearchDirection::Backward => &start,
                                },
                            );
                            self.textbuf.select_range(&start, &end);
                            self.textview
                                .as_ref()
                                .unwrap()
                                .scroll_mark_onscreen(&self.textbuf.create_mark(None, &end, false));
                        }
                    }
                } else {
                    self.search_settings.set_search_text(None);
                }
            }
            Self::Input::LogUpdated(n_log) => {
                let is_at_bottom = {
                    let adj = self.scrolledwin.as_ref().unwrap().vadjustment();
                    (adj.upper() - adj.page_size() - adj.value()) <= 15.0
                };
                self.log.extend(n_log.clone());
                for row in n_log {
                    let txt = match MonadoLog::from_str(row.as_str()) {
                        Some(o) => match o.level >= self.log_level {
                            false => None,
                            true => Some(format!(
                                "{lvl}\t[{file}:{func}]\n\t{msg}\n",
                                lvl = o.level.to_string(),
                                file = o.file,
                                func = o.func,
                                msg = o.message
                            )),
                        },
                        None => Some(row),
                    };
                    if txt.is_some() {
                        self.textbuf
                            .insert(&mut self.textbuf.end_iter(), txt.unwrap().as_str());
                    }
                }
                let textbuf = self.textbuf.clone();
                let textview = self.textview.as_ref().unwrap().clone();
                if is_at_bottom && !self.searchbar.as_ref().unwrap().is_search_mode() {
                    gtk::glib::idle_add_local_once(move || {
                        let end_mark = textbuf.create_mark(None, &textbuf.end_iter(), false);
                        textview.scroll_mark_onscreen(&end_mark);
                    });
                }
            }
            Self::Input::ClearLog => {
                self.log = vec![];
                self.textbuf.set_text("");
            }
            Self::Input::EnableDebugViewChanged(val) => self.set_enable_debug_view(val),
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let textbuf = sourceview5::Buffer::builder()
            .highlight_syntax(false)
            .style_scheme(
                &sourceview5::StyleSchemeManager::new()
                    .scheme("Adwaita-dark")
                    .expect("Couldn't find Adwaita-dark style scheme for gtksourceview5"),
            )
            .build();
        let search_settings = sourceview5::SearchSettings::builder()
            .wrap_around(true)
            .case_sensitive(false)
            .build();
        let search_ctx = sourceview5::SearchContext::builder()
            .buffer(&textbuf)
            .settings(&search_settings)
            .build();
        let mut model = Self {
            tracker: 0,
            log: vec![],
            textbuf,
            textview: None,
            enable_debug_view: init.enable_debug_view,
            searchbar: None,
            search_entry: None,
            dropdown: None,
            scrolledwin: None,
            search_settings,
            search_ctx,
            search_mark: None,
            log_level: LogLevel::Trace,
        };

        let widgets = view_output!();
        model.searchbar = Some(widgets.searchbar.clone());
        model.search_entry = Some(widgets.search_entry.clone());
        model.textview = Some(widgets.textview.clone());
        model.dropdown = Some(widgets.log_level_dropdown.clone());
        model.scrolledwin = Some(widgets.scrolledwin.clone());

        {
            let dd_sender = sender.clone();
            widgets
                .log_level_dropdown
                .connect_selected_notify(move |dd| {
                    dd_sender.input(Self::Input::LogLevelChanged(
                        LogLevel::iter()
                            .as_slice()
                            .get(dd.selected() as usize)
                            .unwrap()
                            .clone(),
                    ));
                });
        }

        ComponentParts { model, widgets }
    }
}
