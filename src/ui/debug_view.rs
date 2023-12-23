use crate::log_level::LogLevel;
use crate::log_parser::MonadoLog;
use crate::ui::app::{DebugOpenDataAction, DebugOpenPrefixAction};
use gtk::glib::clone;
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
    FilterLog(SearchDirection),
    LogLevelChanged(LogLevel),
    XRServiceActiveChanged(bool),
    SetColorScheme,
}

#[tracker::track]
pub struct DebugView {
    xrservice_active: bool,
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
}

pub struct DebugViewInit {}

const MAX_SCROLLBACK: i32 = 2000;

#[relm4::component(pub)]
impl SimpleComponent for DebugView {
    type Init = DebugViewInit;
    type Input = DebugViewMsg;
    type Output = ();

    menu! {
        debug_menu: {
            section! {
                "Open Envision _Data Folder" => DebugOpenDataAction,
                "Open _Prefix Folder" => DebugOpenPrefixAction,
            },
        }
    }

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            set_vexpand: true,
            adw::HeaderBar {
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
            #[name(searchbar)]
            gtk::SearchBar {
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
                add_css_class: "undershoot-top",
                #[name(textview)]
                sourceview5::View {
                    add_css_class: "sourceview-transparent-bg",
                    set_margin_start: 1,
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
            Self::Input::XRServiceActiveChanged(active) => {
                self.set_xrservice_active(active);
                if let Some(dropdown) = self.dropdown.as_ref() {
                    dropdown.set_sensitive(!active);
                }
            }
            Self::Input::LogLevelChanged(lvl) => {
                self.log_level = lvl;
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
                for row in n_log {
                    let txt = match MonadoLog::new_from_str(row.as_str()) {
                        Some(o) => match o.level >= self.log_level {
                            false => None,
                            true => Some(format!(
                                "{lvl}\t[{file}:{func}]\n\t{msg}\n",
                                lvl = o.level,
                                file = o.file,
                                func = o.func,
                                msg = o.message
                            )),
                        },
                        None => Some(row),
                    };
                    if let Some(t) = txt {
                        self.textbuf
                            .insert(&mut self.textbuf.end_iter(), t.as_str());
                    }
                }
                if !self.searchbar.as_ref().unwrap().is_search_mode() {
                    let textbuf = self.textbuf.clone();
                    let textview = self.textview.as_ref().unwrap().clone();
                    gtk::glib::idle_add_local_once(move || {
                        let end_mark = textbuf.create_mark(None, &textbuf.end_iter(), false);
                        textview.scroll_mark_onscreen(&end_mark);
                    });
                }
                while self.textbuf.line_count() > MAX_SCROLLBACK {
                    let mut start = self.textbuf.start_iter();
                    let mut end = self.textbuf.start_iter();
                    end.forward_line();
                    self.textbuf.delete(&mut start, &mut end);
                }
            }
            Self::Input::ClearLog => {
                self.textbuf.set_text("");
            }
            Self::Input::SetColorScheme => {
                Self::set_color_scheme(&self.textbuf);
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let textbuf = sourceview5::Buffer::builder()
            .highlight_syntax(false)
            .enable_undo(false)
            .build();
        let search_settings = sourceview5::SearchSettings::builder()
            .wrap_around(true)
            .case_sensitive(false)
            .build();
        let search_ctx = sourceview5::SearchContext::builder()
            .buffer(&textbuf)
            .settings(&search_settings)
            .build();

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
        Self::set_color_scheme(&textbuf);

        let mut model = Self {
            xrservice_active: false,
            tracker: 0,
            textbuf,
            textview: None,
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
        model.dropdown = Some(log_level_dropdown.clone());
        model.scrolledwin = Some(widgets.scrolledwin.clone());

        ComponentParts { model, widgets }
    }
}

impl DebugView {
    fn set_color_scheme(textbuf: &sourceview5::Buffer) {
        let sourceview_scheme_name = if adw::StyleManager::default().is_dark() {
            "Adwaita-dark"
        } else {
            "Adwaita"
        };
        if let Some(scheme) = &sourceview5::StyleSchemeManager::new().scheme(sourceview_scheme_name)
        {
            textbuf.set_style_scheme(Some(scheme));
        } else {
            println!("gtksourceview style scheme not found")
        }
    }
}
