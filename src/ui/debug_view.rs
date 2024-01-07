use crate::log_level::LogLevel;
use crate::log_parser::MonadoLog;
use crate::ui::app::{DebugOpenDataAction, DebugOpenPrefixAction};
use crate::ui::util::copy_text;
use gtk::glib::clone;
use gtk::prelude::*;
use relm4::prelude::*;
use relm4::{ComponentSender, SimpleComponent};
use zoha_vte4::{Terminal, TerminalExt};

#[derive(Debug)]
pub enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Debug)]
pub enum DebugViewMsg {
    LogUpdated(Vec<String>),
    ClearLog,
    DoSearch,
    SearchFindMatch(SearchDirection),
    LogLevelChanged(LogLevel),
    XRServiceActiveChanged(bool),
    SetColorScheme,
}

#[tracker::track]
pub struct DebugView {
    xrservice_active: bool,
    #[tracker::do_not_track]
    searchbar: Option<gtk::SearchBar>,
    #[tracker::do_not_track]
    search_entry: Option<gtk::SearchEntry>,
    #[tracker::do_not_track]
    dropdown: Option<gtk::DropDown>,
    #[tracker::do_not_track]
    log_level: LogLevel,
    #[tracker::do_not_track]
    vte_terminal: Terminal,
}

pub struct DebugViewInit {}

const MAX_SCROLLBACK: u32 = 2000;

#[relm4::component(pub)]
impl SimpleComponent for DebugView {
    type Init = DebugViewInit;
    type Input = DebugViewMsg;
    type Output = ();

    menu! {
        debug_menu: {
            section! {
                "Open _Data Folder" => DebugOpenDataAction,
                "Open _Prefix Folder" => DebugOpenPrefixAction,
            },
        }
    }

    view! {
        adw::ToolbarView {
            set_hexpand: true,
            set_vexpand: true,
            add_top_bar: hb = &adw::HeaderBar {
                set_hexpand: true,
                set_vexpand: false,
                add_css_class: "flat",
                pack_end: debug_menu_btn = &gtk::MenuButton {
                    set_icon_name: "view-more-symbolic",
                    set_tooltip_text: Some("Debug Actions..."),
                    set_menu_model: Some(&debug_menu),
                },
                pack_end: search_toggle = &gtk::ToggleButton {
                    set_icon_name: "edit-find-symbolic",
                    set_tooltip_text: Some("Filter Log"),
                },
                pack_start: &log_level_dropdown,
            },
            add_top_bar: searchbar = &gtk::SearchBar {
                set_margin_start: 1,
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
                            sender.input(Self::Input::DoSearch);
                        },
                        connect_activate[sender] => move |_| {
                            sender.input(Self::Input::SearchFindMatch(SearchDirection::Forward));
                        },
                    },
                    gtk::Button {
                        set_icon_name: "go-up-symbolic",
                        set_tooltip_text: Some("Previous Match"),
                        connect_clicked[sender] => move |_| {
                            sender.input(Self::Input::SearchFindMatch(SearchDirection::Backward))
                        },
                    },
                    gtk::Button {
                        set_icon_name: "go-down-symbolic",
                        set_tooltip_text: Some("Next Match"),
                        connect_clicked[sender] => move |_| {
                            sender.input(Self::Input::SearchFindMatch(SearchDirection::Forward))
                        },
                    },
                },
                connect_entry: &search_entry,
            },
            #[wrap(Some)]
            set_content: sw = &gtk::ScrolledWindow {
                set_hexpand: true,
                set_vexpand: true,
                model.vte_terminal.clone(),
            }
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        self.reset();

        match message {
            Self::Input::XRServiceActiveChanged(active) => {
                self.set_xrservice_active(active);
                if let Some(dropdown) = self.dropdown.as_ref() {
                    dropdown.set_sensitive(!active);
                }
            }
            Self::Input::LogLevelChanged(lvl) => {
                self.log_level = lvl;
            }
            Self::Input::DoSearch => {
                let searchbar = self.searchbar.as_ref().unwrap().clone();
                let search_entry = self.search_entry.as_ref().unwrap().clone();
                let search_text = search_entry.text().to_string();
                if searchbar.is_search_mode() && !search_text.is_empty() {
                    if let Ok(regex) = zoha_vte4::Regex::for_search(&search_text, 0) {
                        self.vte_terminal.search_set_regex(Some(&regex), 0);
                    }
                } else {
                    self.vte_terminal.search_set_regex(None, 0);
                }
            }
            Self::Input::SearchFindMatch(direction) => match direction {
                SearchDirection::Forward => {
                    self.vte_terminal.search_find_next();
                }
                SearchDirection::Backward => {
                    self.vte_terminal.search_find_previous();
                }
            },
            Self::Input::LogUpdated(n_log) => {
                for row in n_log {
                    let txt = match MonadoLog::new_from_str(row.as_str()) {
                        Some(o) => match o.level >= self.log_level {
                            false => None,
                            true => Some(format!(
                                "{lvl}\t[{file}:{func}]\r\n\t{msg}\r\n",
                                lvl = o.level,
                                file = o.file,
                                func = o.func,
                                msg = o.message.replace("\n", "\r\n")
                            )),
                        },
                        None => Some(row),
                    };
                    if let Some(t) = txt {
                        self.vte_terminal.feed(t.as_bytes())
                    }
                }
            }
            Self::Input::ClearLog => {
                self.vte_terminal.feed("\x1bc".as_bytes());
            }
            Self::Input::SetColorScheme => {
                Self::set_color_scheme(&self.vte_terminal);
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let log_level_dropdown = gtk::DropDown::from_strings(
            LogLevel::iter()
                .map(|lvl| lvl.to_string())
                .collect::<Vec<String>>()
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>()
                .as_slice(),
        );
        if let Some(btn) = log_level_dropdown.first_child() {
            btn.add_css_class("flat");
        }
        log_level_dropdown.connect_selected_notify(clone!(@strong sender => move |dd| {
            sender.input(Self::Input::LogLevelChanged(
                *LogLevel::iter()
                    .as_slice()
                    .get(dd.selected() as usize)
                    .unwrap(),
            ));
        }));

        adw::StyleManager::default().connect_dark_notify(clone!(@strong sender => move |_| {
            sender.input(Self::Input::SetColorScheme);
        }));

        let mut model = Self {
            xrservice_active: false,
            tracker: 0,
            searchbar: None,
            search_entry: None,
            dropdown: None,
            log_level: LogLevel::Trace,
            vte_terminal: {
                let t = Terminal::builder()
                    .scroll_on_output(true)
                    .scrollback_lines(MAX_SCROLLBACK)
                    .scroll_unit_is_pixels(true)
                    .vexpand(true)
                    .hexpand(true)
                    .build();
                t.set_clear_background(false);
                t.search_set_wrap_around(true);
                Self::set_color_scheme(&t);
                t
            },
        };

        {
            let sc = gtk::ShortcutController::new();
            let term = model.vte_terminal.clone();
            sc.add_shortcut(gtk::Shortcut::new(
                gtk::ShortcutTrigger::parse_string("<Control>c"),
                Some(gtk::CallbackAction::new(move |_, _| {
                    if let Some(text) = term.text_selected(zoha_vte4::Format::Text) {
                        copy_text(text.as_str());
                    }
                    true
                })),
            ));
            let term = model.vte_terminal.clone();
            sc.add_shortcut(gtk::Shortcut::new(
                gtk::ShortcutTrigger::parse_string("<Control>a"),
                Some(gtk::CallbackAction::new(move |_, _| {
                    term.select_all();
                    true
                })),
            ));
            model.vte_terminal.add_controller(sc);
        }

        let widgets = view_output!();
        model.searchbar = Some(widgets.searchbar.clone());
        model.search_entry = Some(widgets.search_entry.clone());
        model.dropdown = Some(log_level_dropdown.clone());

        ComponentParts { model, widgets }
    }
}

impl DebugView {
    fn set_color_scheme(term: &Terminal) {
        if adw::StyleManager::default().is_dark() {
            term.set_color_foreground(&gtk::gdk::RGBA::new(1.0, 1.0, 1.0, 1.0));
        } else {
            term.set_color_foreground(&gtk::gdk::RGBA::new(0.0, 0.0, 0.0, 1.0));
        }
    }
}
