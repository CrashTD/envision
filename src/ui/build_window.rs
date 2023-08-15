use gtk::prelude::*;
use relm4::prelude::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BuildStatus {
    Building,
    Done,
    Error(String),
}

#[tracker::track]
pub struct BuildWindow {
    title: String,
    content: String,
    can_close: bool,
    build_status: BuildStatus,

    #[tracker::do_not_track]
    pub textbuf: gtk::TextBuffer,
    #[tracker::do_not_track]
    pub textview: Option<gtk::TextView>,
    #[tracker::do_not_track]
    pub win: Option<adw::Window>,
    #[tracker::do_not_track]
    build_status_label: Option<gtk::Label>,
    #[tracker::do_not_track]
    scrolledwin: Option<gtk::ScrolledWindow>,
}

#[derive(Debug)]
pub enum BuildWindowMsg {
    Present,
    UpdateTitle(String),
    UpdateBuildStatus(BuildStatus),
    UpdateContent(Vec<String>),
    UpdateCanClose(bool),
}

#[relm4::component(pub)]
impl SimpleComponent for BuildWindow {
    type Init = ();
    type Input = BuildWindowMsg;
    type Output = ();

    view! {
        #[name(win)]
        adw::Window {
            set_modal: true,
            set_default_size: (520, 400),
            set_hide_on_close: true,
            gtk::Box {
                set_vexpand: true,
                set_hexpand: true,
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 12,
                gtk::WindowHandle {
                    set_vexpand: false,
                    set_hexpand: true,
                    adw::HeaderBar {
                        set_show_end_title_buttons: false,
                        set_show_start_title_buttons: false,
                        add_css_class: "flat",
                        #[wrap(Some)]
                        set_title_widget: title_label = &gtk::Label {
                            #[track = "model.changed(BuildWindow::title())"]
                            set_markup: &model.title,
                            add_css_class: "title",
                        },
                    }
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,
                    set_halign: gtk::Align::Center,
                    set_spacing: 12,
                    #[name(build_status_label)]
                    gtk::Label {
                        #[track = "model.changed(BuildWindow::build_status())"]
                        set_markup: match &model.build_status {
                            BuildStatus::Building => "Build in progress...".to_string(),
                            BuildStatus::Done => "Build done, you can close this window".to_string(),
                            BuildStatus::Error(code) => {
                                format!("Build failed: \"{c}\"", c = code)
                            }
                        }.as_str(),
                        add_css_class: "title-2",
                        set_wrap: true,
                        set_wrap_mode: gtk::pango::WrapMode::Word,
                        set_justify: gtk::Justification::Center,
                    }
                },
                #[name(scrolledwin)]
                gtk::ScrolledWindow {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_margin_all: 12,
                    add_css_class: "card",
                    set_overflow: gtk::Overflow::Hidden,
                    #[name(textview)]
                    gtk::TextView {
                        set_hexpand: true,
                        set_vexpand: true,
                        set_editable: false,
                        set_monospace: true,
                        set_left_margin: 6,
                        set_right_margin: 6,
                        set_top_margin: 6,
                        set_bottom_margin: 6,
                        set_buffer: Some(&model.textbuf),
                    },
                },
                gtk::Button {
                    add_css_class: "pill",
                    set_halign: gtk::Align::Center,
                    set_label: "Close",
                    set_margin_all: 12,
                    #[track = "model.changed(BuildWindow::can_close())"]
                    set_sensitive: model.can_close,
                    connect_clicked[win] => move |_| {

                        win.close();
                    },
                }
            }
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        self.reset();

        match message {
            Self::Input::Present => {
                self.win.as_ref().unwrap().present();
                sender.input(BuildWindowMsg::UpdateBuildStatus(BuildStatus::Building));
                self.set_content("".into());
                self.textbuf.set_text("");
            }
            Self::Input::UpdateTitle(t) => {
                self.set_title(t);
            }
            Self::Input::UpdateContent(c) => {
                if c.len() > 0 {
                    let is_at_bottom = {
                        let adj = self.scrolledwin.as_ref().unwrap().vadjustment();
                        (adj.upper() - adj.page_size() - adj.value()) <= 15.0
                    };

                    let n_lines = c.concat();
                    let mut n_content = self.content.clone();
                    n_content.push_str(n_lines.as_str());
                    self.set_content(n_content);
                    self.textbuf
                        .insert(&mut self.textbuf.end_iter(), n_lines.as_str());
                    let textbuf = self.textbuf.clone();
                    let textview = self.textview.as_ref().unwrap().clone();
                    if is_at_bottom {
                        gtk::glib::idle_add_local_once(move || {
                            let end_mark = textbuf.create_mark(None, &textbuf.end_iter(), false);
                            textview.scroll_mark_onscreen(&end_mark);
                        });
                    }
                }
            }
            Self::Input::UpdateBuildStatus(status) => {
                let label = self.build_status_label.as_ref().unwrap();
                label.remove_css_class("success");
                label.remove_css_class("error");
                match status {
                    BuildStatus::Done => label.add_css_class("success"),
                    BuildStatus::Error(_) => label.add_css_class("error"),
                    _ => {}
                }
                self.set_build_status(status);
            }
            Self::Input::UpdateCanClose(val) => {
                self.set_can_close(val);
            }
        };
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = Self {
            tracker: 0,
            title: "".into(),
            content: "".into(),
            can_close: false,
            textbuf: gtk::TextBuffer::builder().build(),
            textview: None,
            build_status: BuildStatus::Building,
            win: None,
            scrolledwin: None,
            build_status_label: None,
        };
        let widgets = view_output!();
        model.win = Some(widgets.win.clone());
        model.build_status_label = Some(widgets.build_status_label.clone());
        model.textview = Some(widgets.textview.clone());
        model.scrolledwin = Some(widgets.scrolledwin.clone());
        ComponentParts { model, widgets }
    }
}
