use gtk::prelude::*;
use relm4::prelude::*;

#[tracker::track]
pub struct BuildWindow {
    title: String,
    content: String,
    can_close: bool,

    #[tracker::do_not_track]
    pub textbuf: gtk::TextBuffer,
    #[tracker::do_not_track]
    pub win: Option<adw::Window>,
    #[tracker::do_not_track]
    pub scrolled_win: Option<gtk::ScrolledWindow>,
}

#[derive(Debug)]
pub enum BuildWindowMsg {
    Present,
    UpdateTitle(String),
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
            gtk::Box {
                set_vexpand: true,
                set_hexpand: true,
                set_orientation: gtk::Orientation::Vertical,
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
                            set_label: &model.title,
                            add_css_class: "title",
                        },
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
                        win.hide();
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
            },
            BuildWindowMsg::UpdateTitle(t) => {
                self.set_title(t);
            },
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
            },
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
            win: None,
            scrolled_win: None,
        };
        let widgets = view_output!();
        model.scrolled_win = Some(widgets.scrolled_win.clone());
        model.win = Some(widgets.win.clone());
        ComponentParts { model, widgets }
    }
}
