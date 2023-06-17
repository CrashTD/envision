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
    pub win: Option<adw::Window>,
    #[tracker::do_not_track]
    build_status_label: Option<gtk::Label>,
    #[tracker::do_not_track]
    pub scrolled_win: Option<gtk::ScrolledWindow>,
}

#[derive(Debug)]
pub enum BuildWindowMsg {
    Present,
    UpdateTitle(String),
    UpdateBuildStatus(BuildStatus),
    UpdateContent(String),
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
                #[name(scrolled_win)]
                gtk::ScrolledWindow {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_margin_all: 12,
                    add_css_class: "card",
                    set_overflow: gtk::Overflow::Hidden,
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
            BuildWindowMsg::Present => {
                self.win.as_ref().unwrap().present();
                sender.input(BuildWindowMsg::UpdateBuildStatus(BuildStatus::Building));
            }
            BuildWindowMsg::UpdateTitle(t) => {
                self.set_title(t);
            }
            BuildWindowMsg::UpdateContent(c) => {
                if self.content != c {
                    self.set_content(c);
                    self.textbuf.set_text(&self.content);
                    let sw = self.scrolled_win.as_ref().unwrap().clone();
                    let adj = sw.vadjustment();
                    // upper means highest value = lowest point in the sw
                    adj.set_value(adj.upper());
                    sw.set_vadjustment(Some(&adj));
                }
            }
            BuildWindowMsg::UpdateBuildStatus(status) => {
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
            BuildWindowMsg::UpdateCanClose(val) => {
                self.set_can_close(val);
            }
        };
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = BuildWindow {
            tracker: 0,
            title: "".into(),
            content: "".into(),
            can_close: false,
            textbuf: gtk::TextBuffer::builder().build(),
            build_status: BuildStatus::Building,
            win: None,
            build_status_label: None,
            scrolled_win: None,
        };
        let widgets = view_output!();
        model.scrolled_win = Some(widgets.scrolled_win.clone());
        model.win = Some(widgets.win.clone());
        model.build_status_label = Some(widgets.build_status_label.clone());
        ComponentParts { model, widgets }
    }
}
