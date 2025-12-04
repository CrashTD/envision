use gtk::{gdk, glib::clone, prelude::*};
use vte4::{Terminal, prelude::*};

use super::util::copy_text;

const MAX_SCROLLBACK: u32 = 2000;

#[derive(Debug, Clone)]
pub struct TermWidget {
    pub container: gtk::Box,
    pub term: Terminal,
    popover_menu: gtk::PopoverMenu,
}

const ADW_LIGHT_FG: &str = "#000000";
const ADW_DARK_FG: &str = "#ffffff";
const ADW_PALETTE: [&str; 16] = [
    "#241f31", "#c01c28", "#2ec27e", "#f5c211", "#1e78e4", "#9841bb", "#0ab9dc", "#c0bfbc",
    "#5e5c64", "#ed333b", "#57e389", "#f8e45c", "#51a1ff", "#c061cb", "#4fd2fd", "#f6f5f4",
];

impl TermWidget {
    pub fn new() -> Self {
        let term = Terminal::builder()
            .scroll_on_output(true)
            .scrollback_lines(MAX_SCROLLBACK)
            .scroll_unit_is_pixels(true)
            .vexpand(true)
            .hexpand(true)
            .build();
        term.set_clear_background(false);
        term.search_set_wrap_around(true);
        let container = gtk::Box::builder().hexpand(true).vexpand(true).build();

        let sw = gtk::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .child(&term)
            .build();

        container.append(&sw);

        let action_grp = gtk::gio::SimpleActionGroup::new();
        let selectall_action = gtk::gio::SimpleAction::new("selectall", None);
        selectall_action.connect_activate(clone!(
            #[strong]
            term,
            move |_, _| {
                term.select_all();
            }
        ));
        action_grp.add_action(&selectall_action);
        let copy_action = gtk::gio::SimpleAction::new("copy", None);
        copy_action.connect_activate(clone!(
            #[strong]
            term,
            move |_, _| {
                Self::copy_selected(&term);
            }
        ));
        action_grp.add_action(&copy_action);

        container.insert_action_group("term", Some(&action_grp));

        let menu = gtk::gio::Menu::new();
        menu.append(Some("Select all"), Some("term.selectall"));
        menu.append(Some("Copy"), Some("term.copy"));

        let popover_menu = gtk::PopoverMenu::builder()
            .pointing_to(&term.allocation())
            .autohide(true)
            .menu_model(&menu)
            .build();

        container.append(&popover_menu);

        let this = Self {
            container,
            term,
            popover_menu,
        };

        this.setup_shortcuts();
        this.setup_rightclick_gesture();

        this
    }

    fn setup_rightclick_gesture(&self) {
        let gesture = gtk::GestureClick::builder()
            .button(gtk::gdk::BUTTON_SECONDARY)
            .build();
        gesture.connect_pressed(clone!(
            #[strong(rename_to=popover)]
            self.popover_menu,
            move |gesture, _, x, y| {
                gesture.set_state(gtk::EventSequenceState::Claimed);
                popover.set_pointing_to(Some(&gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
                popover.popup();
            }
        ));
        self.term.add_controller(gesture);
    }

    fn copy_selected(term: &Terminal) {
        if let Some(text) = term.text_selected(vte4::Format::Text) {
            copy_text(text.as_str());
        }
    }

    fn setup_shortcuts(&self) {
        let sc = gtk::ShortcutController::new();
        ["<Control>c", "<Shift><Control>c"]
            .iter()
            .for_each(|combo| {
                sc.add_shortcut(gtk::Shortcut::new(
                    gtk::ShortcutTrigger::parse_string(combo),
                    Some(gtk::CallbackAction::new(clone!(
                        #[strong(rename_to = term)]
                        self.term,
                        move |_, _| {
                            Self::copy_selected(&term);
                            gtk::glib::Propagation::Proceed
                        }
                    ))),
                ));
            });
        ["<Control>a", "<Shift><Control>a"]
            .iter()
            .for_each(|combo| {
                sc.add_shortcut(gtk::Shortcut::new(
                    gtk::ShortcutTrigger::parse_string(combo),
                    Some(gtk::CallbackAction::new(clone!(
                        #[strong(rename_to = term)]
                        self.term,
                        move |_, _| {
                            term.select_all();
                            gtk::glib::Propagation::Proceed
                        }
                    ))),
                ));
            });
        self.term.add_controller(sc);
    }

    pub fn set_color_scheme(&self) {
        let fg = if adw::StyleManager::default().is_dark() {
            ADW_DARK_FG
        } else {
            ADW_LIGHT_FG
        };
        let rgba_palette = ADW_PALETTE
            .iter()
            .map(|c| gdk::RGBA::parse(*c).unwrap())
            .collect::<Vec<gdk::RGBA>>();
        self.term.set_colors(
            Some(&gdk::RGBA::parse(fg).unwrap()),
            None,
            // better way to convert to vec of references?
            rgba_palette.iter().collect::<Vec<&gdk::RGBA>>().as_slice(),
        );
    }

    pub fn feed(&self, txt: &str) {
        self.term.feed(txt.replace('\n', "\r\n").as_bytes())
    }

    pub fn clear(&self) {
        self.term.feed("\x1bc".as_bytes());
    }

    pub fn set_search_term(&self, term: Option<&str>) {
        self.term.search_set_regex(
            term.and_then(|txt| vte4::Regex::for_search(txt, 0).ok())
                .as_ref(),
            0,
        );
    }

    pub fn search_next(&self) {
        self.term.search_find_next();
    }

    pub fn search_prev(&self) {
        self.term.search_find_previous();
    }
}
