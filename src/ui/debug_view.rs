use super::term_widget::TermWidget;
use crate::log_level::LogLevel;
use crate::log_parser::MonadoLog;
use crate::ui::app::{DebugCopyEnvVarsAction, DebugOpenDataAction, DebugOpenPrefixAction};
use crate::ui::util::copy_text;
use gtk::glib::clone;
use gtk::prelude::*;
use relm4::prelude::*;
use relm4::{ComponentSender, SimpleComponent};
use vte4::TerminalExt;

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

#[derive(Debug)]
pub enum DebugViewOutMsg {
    StartWithDebug,
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
    term: TermWidget,
}

pub struct DebugViewInit {}

#[relm4::component(pub)]
impl SimpleComponent for DebugView {
    type Init = DebugViewInit;
    type Input = DebugViewMsg;
    type Output = DebugViewOutMsg;

    menu! {
        debug_menu: {
            section! {
                "Open _Data Folder" => DebugOpenDataAction,
                "Open _Prefix Folder" => DebugOpenPrefixAction,
                "Copy Profile _Environment Variables" => DebugCopyEnvVarsAction,
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
                pack_start: run_debug_btn = &gtk::Button {
                    set_icon_name: "run-debug-symbolic",
                    set_tooltip_text: Some("Start with gdbserver"),
                    connect_clicked[sender] => move |_| {
                        sender.output(Self::Output::StartWithDebug)
                            .expect("Sender output failed");
                    },
                    #[track = "model.changed(Self::xrservice_active())"]
                    set_sensitive: !model.xrservice_active
                },
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
            set_content: Some(&model.term.container),
        }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
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
                    self.term.set_search_term(Some(&search_text));
                } else {
                    self.term.set_search_term(None);
                }
            }
            Self::Input::SearchFindMatch(direction) => match direction {
                SearchDirection::Forward => {
                    self.term.search_next();
                }
                SearchDirection::Backward => {
                    self.term.search_prev();
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
                                msg = o.message.replace('\n', "\r\n")
                            )),
                        },
                        None => Some(row),
                    };
                    if let Some(t) = txt {
                        self.term.feed(&t);
                    }
                }
            }
            Self::Input::ClearLog => {
                self.term.clear();
            }
            Self::Input::SetColorScheme => {
                self.term.set_color_scheme();
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
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
            term: TermWidget::new(),
        };
        model.term.set_color_scheme();

        {
            let sc = gtk::ShortcutController::new();
            let term = model.term.term.clone();
            sc.add_shortcut(gtk::Shortcut::new(
                gtk::ShortcutTrigger::parse_string("<Control>c"),
                Some(gtk::CallbackAction::new(move |_, _| {
                    if let Some(text) = term.text_selected(vte4::Format::Text) {
                        copy_text(text.as_str());
                    }
                    gtk::glib::Propagation::Proceed
                })),
            ));
            let term = model.term.term.clone();
            sc.add_shortcut(gtk::Shortcut::new(
                gtk::ShortcutTrigger::parse_string("<Control>a"),
                Some(gtk::CallbackAction::new(move |_, _| {
                    term.select_all();
                    gtk::glib::Propagation::Proceed
                })),
            ));
            model.term.term.add_controller(sc);
        }

        let widgets = view_output!();
        model.searchbar = Some(widgets.searchbar.clone());
        model.search_entry = Some(widgets.search_entry.clone());
        model.dropdown = Some(log_level_dropdown.clone());

        ComponentParts { model, widgets }
    }
}
