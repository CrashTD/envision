use crate::termcolor::TermColor;

use super::{term_widget::TermWidget, SENDER_IO_ERR_MSG};
use adw::prelude::*;
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
    can_close: bool,
    build_status: BuildStatus,

    #[tracker::do_not_track]
    pub win: Option<adw::Dialog>,
    #[tracker::do_not_track]
    build_status_label: Option<gtk::Label>,
    #[tracker::do_not_track]
    term: TermWidget,
    #[tracker::do_not_track]
    parent: gtk::Widget,
}

#[derive(Debug)]
pub enum BuildWindowMsg {
    Present,
    UpdateTitle(String),
    UpdateBuildStatus(BuildStatus),
    UpdateContent(Vec<String>),
    UpdateCanClose(bool),
}

#[derive(Debug)]
pub enum BuildWindowOutMsg {
    CancelBuild,
}

#[derive(Debug)]
pub struct BuildWindowInit {
    pub parent: gtk::Widget,
}

#[relm4::component(pub)]
impl SimpleComponent for BuildWindow {
    type Init = BuildWindowInit;
    type Input = BuildWindowMsg;
    type Output = BuildWindowOutMsg;

    view! {
        #[name(win)]
        adw::Dialog {
            set_presentation_mode: adw::DialogPresentationMode::BottomSheet,
            set_content_height: 2000,
            set_content_width: 1000,
            #[track = "model.changed(BuildWindow::can_close())"]
            set_can_close: model.can_close,
            #[wrap(Some)]
            set_child: tbview = &adw::ToolbarView {
                set_top_bar_style: adw::ToolbarStyle::Flat,
                set_bottom_bar_style: adw::ToolbarStyle::Flat,
                set_vexpand: true,
                set_hexpand: true,
                add_top_bar: top_bar = &adw::HeaderBar {
                    set_show_end_title_buttons: false,
                    set_show_start_title_buttons: false,
                    #[wrap(Some)]
                    set_title_widget: title_label = &adw::WindowTitle {
                        #[track = "model.changed(BuildWindow::title())"]
                        set_title: &model.title,
                    },
                },
                #[wrap(Some)]
                set_content: content = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_vexpand: true,
                    set_hexpand: true,
                    set_margin_all: 12,
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_hexpand: true,
                        set_halign: gtk::Align::Center,
                        set_spacing: 12,
                        #[name(build_status_label)]
                        gtk::Label {
                            #[track = "model.changed(BuildWindow::build_status())"]
                            set_markup: match &model.build_status {
                                BuildStatus::Building => String::default(),
                                BuildStatus::Done => "Build done, you can close this window".into(),
                                BuildStatus::Error(code) => {
                                    format!("Build failed: \"{code}\"")
                                }
                            }.as_str(),
                            #[track = "model.changed(BuildWindow::build_status())"]
                            set_visible: match &model.build_status {
                                BuildStatus::Building => false,
                                BuildStatus::Done | BuildStatus::Error(_) => true,
                            },
                            add_css_class: "title-2",
                            set_wrap: true,
                            set_wrap_mode: gtk::pango::WrapMode::Word,
                            set_justify: gtk::Justification::Center,
                        },
                    },
                    model.term.container.clone(),
                },
                add_bottom_bar: bottom_bar = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_halign: gtk::Align::Center,
                    set_hexpand: true,
                    set_margin_bottom: 24,
                    set_spacing: 12,
                    gtk::Button {
                        add_css_class: "pill",
                        set_halign: gtk::Align::Center,
                        set_label: "Close",
                        #[track = "model.changed(BuildWindow::can_close())"]
                        set_visible: model.can_close,
                        connect_clicked[win] => move |_| {
                            win.close();
                        },
                    },
                    // this
                    gtk::Button {
                        #[track = "model.changed(BuildWindow::build_status())"]
                        set_visible: matches!(&model.build_status, BuildStatus::Building),
                        add_css_class: "destructive-action",
                        add_css_class: "pill",
                        set_label: "Cancel build",
                        connect_clicked[sender] => move |_| {
                            sender.output(Self::Output::CancelBuild).expect(SENDER_IO_ERR_MSG);
                        }
                    },
                    // ^^^
                },
            }
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        self.reset();

        match message {
            Self::Input::Present => {
                self.term.set_color_scheme();
                sender.input(BuildWindowMsg::UpdateBuildStatus(BuildStatus::Building));
                self.term.clear();
                self.win.as_ref().unwrap().present(Some(&self.parent));
            }
            Self::Input::UpdateTitle(t) => {
                self.set_title(t);
            }
            Self::Input::UpdateContent(c) => {
                if !c.is_empty() {
                    let n_lines = c.concat();
                    self.term.feed(&n_lines);
                }
            }
            Self::Input::UpdateBuildStatus(status) => {
                let label = self.build_status_label.as_ref().unwrap();
                label.remove_css_class("success");
                label.remove_css_class("error");
                match status {
                    BuildStatus::Done => {
                        label.add_css_class("success");
                        sender.input(BuildWindowMsg::UpdateContent(vec![
                            TermColor::Blue.colorize("Build completed!\n")
                        ]));
                    }
                    BuildStatus::Error(_) => {
                        label.add_css_class("error");
                        sender.input(BuildWindowMsg::UpdateContent(vec![
                            TermColor::Blue.colorize("Build failed!\n")
                        ]));
                    }
                    _ => {}
                }
                if status != BuildStatus::Building {
                    sender.input(Self::Input::UpdateCanClose(true));
                }
                self.set_build_status(status);
            }
            Self::Input::UpdateCanClose(val) => {
                self.set_can_close(val);
            }
        };
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = Self {
            tracker: 0,
            title: "".into(),
            can_close: false,
            build_status: BuildStatus::Building,
            win: None,
            build_status_label: None,
            parent: init.parent,
            term: {
                let t = TermWidget::new();
                t.container.add_css_class("card");
                t.container.set_margin_all(12);
                t.container.set_overflow(gtk::Overflow::Hidden);
                t
            },
        };
        let widgets = view_output!();
        model.win = Some(widgets.win.clone());
        model.build_status_label = Some(widgets.build_status_label.clone());
        ComponentParts { model, widgets }
    }
}
