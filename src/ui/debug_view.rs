use gtk::prelude::*;
use relm4::prelude::*;
use relm4::{ComponentSender, SimpleComponent};
use relm4_icons::icon_name;

#[derive(Debug)]
pub enum DebugLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl DebugLevel {
    pub fn from_string(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "debug" => Self::Debug,
            "info" => Self::Info,
            "warning" => Self::Warning,
            "error" => Self::Error,
            _ => Self::Debug,
        }
    }
}

pub const DEBUG_LEVEL_STRINGS: [&str; 4] = ["Debug", "Info", "Warning", "Error"];

#[derive(Debug)]
pub enum DebugViewMsg {
    DebugLevelChanged(DebugLevel),
    SearchTermChanged(String),
    LogUpdated(Vec<String>),
    EnableDebugViewChanged(bool),
}

#[tracker::track]
pub struct DebugView {
    #[tracker::do_not_track]
    pub debug_level: DebugLevel,
    #[tracker::do_not_track]
    pub search_term: String,
    #[tracker::do_not_track]
    pub log: String,
    #[tracker::do_not_track]
    pub textbuf: gtk::TextBuffer,
    #[tracker::do_not_track]
    pub textview: Option<gtk::TextView>,
    #[tracker::do_not_track]
    pub searchbar: Option<gtk::SearchBar>,
    #[tracker::do_not_track]
    pub search_entry: Option<gtk::SearchEntry>,

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
                        set_icon_name: icon_name::LOUPE,
                        set_tooltip_text: Some("Filter Log"),
                    },
                    pack_start: log_level_dropdown = &gtk::DropDown::from_strings(&DEBUG_LEVEL_STRINGS) {
                    },
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
            Self::Input::DebugLevelChanged(level) => {}
            Self::Input::SearchTermChanged(term) => {}
            Self::Input::LogUpdated(n_log) => {
                let searchbar = self.searchbar.as_ref().unwrap().clone();
                let search_entry = self.search_entry.as_ref().unwrap().clone();
                let mut search_text: String = search_entry.text().into();
                search_text = search_text.trim().to_lowercase();
                if searchbar.is_search_mode() && !search_text.is_empty() {
                    self.log = n_log
                        .iter()
                        .filter(|row| row.to_lowercase().contains(&search_text))
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>()
                        .concat();
                } else {
                    self.log = n_log.concat();
                }
                self.textbuf.set_text(&self.log);
                let textbuf = self.textbuf.clone();
                let textview = self.textview.as_ref().unwrap().clone();
                gtk::glib::idle_add_local_once(move || {
                    let end_mark = textbuf.create_mark(None, &textbuf.end_iter(), false);
                    textview.scroll_mark_onscreen(&end_mark);
                });
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
            debug_level: DebugLevel::Debug,
            search_term: "".into(),
            log: "".into(),
            textbuf: gtk::TextBuffer::builder().build(),
            textview: None,
            enable_debug_view: init.enable_debug_view,
            searchbar: None,
            search_entry: None,
        };

        let widgets = view_output!();
        model.searchbar = Some(widgets.searchbar.clone());
        model.search_entry = Some(widgets.search_entry.clone());
        model.textview = Some(widgets.textview.clone());

        ComponentParts { model, widgets }
    }
}
