use expect_dialog::ExpectDialog;
use gtk::prelude::*;
use relm4::prelude::*;
use relm4::{ComponentSender, SimpleComponent};
use std::fmt::Display;
use std::slice::Iter;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warning,
    Error,
}

impl LogLevel {
    pub fn from_string(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "trace" => Self::Trace,
            "debug" => Self::Debug,
            "info" => Self::Info,
            "warning" => Self::Warning,
            "error" => Self::Error,
            _ => Self::Debug,
        }
    }

    pub fn iter() -> Iter<'static, LogLevel> {
        [
            Self::Trace,
            Self::Debug,
            Self::Info,
            Self::Warning,
            Self::Error,
        ]
        .iter()
    }
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            LogLevel::Trace => "Trace",
            LogLevel::Debug => "Debug",
            LogLevel::Info => "Info",
            LogLevel::Warning => "Warning",
            LogLevel::Error => "Error",
        })
    }
}

#[derive(Debug)]
pub enum DebugViewMsg {
    LogUpdated(Vec<String>),
    EnableDebugViewChanged(bool),
    FilterLog,
}

#[tracker::track]
pub struct DebugView {
    #[tracker::do_not_track]
    pub log: Vec<String>,
    #[tracker::do_not_track]
    pub textbuf: gtk::TextBuffer,
    #[tracker::do_not_track]
    pub textview: Option<gtk::TextView>,
    #[tracker::do_not_track]
    pub searchbar: Option<gtk::SearchBar>,
    #[tracker::do_not_track]
    pub search_entry: Option<gtk::SearchEntry>,
    #[tracker::do_not_track]
    pub dropdown: Option<gtk::DropDown>,

    pub enable_debug_view: bool,
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
                set_child: search_entry = &gtk::SearchEntry {
                    set_hexpand: true,
                    connect_changed[sender] => move |_| {
                        sender.input(Self::Input::FilterLog);
                    }
                },
                connect_entry: &search_entry,
            },
            gtk::ScrolledWindow {
                set_hexpand: true,
                set_vexpand: true,
                #[name(textview)]
                gtk::TextView {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_editable: false,
                    set_monospace: true,
                    set_buffer: Some(&model.textbuf)
                },
            }
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        self.reset();

        match message {
            Self::Input::FilterLog => {
                let searchbar = self.searchbar.as_ref().unwrap().clone();
                let search_entry = self.search_entry.as_ref().unwrap().clone();
                let search_text = search_entry.text().to_string().trim().to_lowercase();
                // TODO: add log level filtering
                let log_level = LogLevel::iter()
                    .as_slice()
                    .get(self.dropdown.as_ref().unwrap().selected() as usize)
                    .unwrap();
                println!("log level: {}", log_level.to_string());
                let log = match searchbar.is_search_mode() && !search_text.is_empty() {
                    true => self
                        .log
                        .iter()
                        .filter(|row| row.to_lowercase().contains(&search_text))
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>()
                        .concat(),
                    false => self.log.concat(),
                };
                self.textbuf.set_text(&log);
                let textbuf = self.textbuf.clone();
                let textview = self.textview.as_ref().unwrap().clone();
                gtk::glib::idle_add_local_once(move || {
                    let end_mark = textbuf.create_mark(None, &textbuf.end_iter(), false);
                    textview.scroll_mark_onscreen(&end_mark);
                });
            }
            Self::Input::LogUpdated(n_log) => {
                self.log = n_log;
                sender.input(Self::Input::FilterLog);
            }
            Self::Input::EnableDebugViewChanged(val) => self.set_enable_debug_view(val),
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = Self {
            tracker: 0,
            log: vec![],
            textbuf: gtk::TextBuffer::builder().build(),
            textview: None,
            enable_debug_view: init.enable_debug_view,
            searchbar: None,
            search_entry: None,
            dropdown: None,
        };

        let widgets = view_output!();
        model.searchbar = Some(widgets.searchbar.clone());
        model.search_entry = Some(widgets.search_entry.clone());
        model.textview = Some(widgets.textview.clone());
        model.dropdown = Some(widgets.log_level_dropdown.clone());

        {
            let dd_sender = sender.clone();
            widgets
                .log_level_dropdown
                .connect_selected_notify(move |dd| {
                    dd_sender.input(Self::Input::FilterLog);
                });
        }

        ComponentParts { model, widgets }
    }
}
