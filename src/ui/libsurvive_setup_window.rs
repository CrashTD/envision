use crate::{profile::Profile, runner::Runner};
use gtk::prelude::*;
use relm4::prelude::*;
use std::path::Path;

const NO_FILE_MSG: &str = "(No file selected)";

#[tracker::track]
pub struct LibsurviveSetupWindow {
    steam_lighthouse_path: String,

    #[tracker::do_not_track]
    win: Option<adw::Window>,
    #[tracker::do_not_track]
    progressbar: Option<gtk::ProgressBar>,
    #[tracker::do_not_track]
    carousel: Option<adw::Carousel>,
    #[tracker::do_not_track]
    loading_page: Option<adw::StatusPage>,
    #[tracker::do_not_track]
    filefilter_listmodel: gtk::gio::ListStore,

    #[tracker::do_not_track]
    profile: Option<Profile>,
    #[tracker::do_not_track]
    calibration_runner: Option<Runner>,
}

#[derive(Debug)]
pub enum LibsurviveSetupMsg {
    Present(Profile),
    CalibrateLibsurvive,
    SetSteamLighthousePath(Option<String>),
    ChooseFileDialog,
}

#[relm4::component(pub)]
impl SimpleComponent for LibsurviveSetupWindow {
    type Init = ();
    type Input = LibsurviveSetupMsg;
    type Output = ();

    view! {
        #[name(win)]
        adw::Window {
            set_modal: true,
            set_default_size: (520, 400),
            set_hide_on_close: true,
            connect_close_request[sender, carousel, page1, progressbar] => move |_| {
                carousel.scroll_to(&page1, true);
                progressbar.set_fraction(0.0);
                sender.input(LibsurviveSetupMsg::SetSteamLighthousePath(None));
                gtk::Inhibit(false)
            },
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: true,
                set_vexpand: true,
                set_spacing: 12,
                set_margin_all: 12,
                gtk::WindowHandle {
                    adw::HeaderBar {
                        add_css_class: "flat",
                        #[wrap(Some)]
                        set_title_widget: title_label = &gtk::Label {
                            set_label: "Setup Lighthouses",
                            add_css_class: "title",
                        },
                    }
                },
                #[name(carousel)]
                adw::Carousel {
                    set_allow_long_swipes: false,
                    set_allow_scroll_wheel: false,
                    set_allow_mouse_drag: false,
                    set_hexpand: true,
                    set_vexpand: true,
                    #[name(page1)]
                    adw::StatusPage {
                        set_hexpand: true,
                        set_vexpand: true,
                        set_title: "Let's Get Started",
                        set_description: Some(concat!(
                            "This procedure will guide you through importing ",
                            "your SteamVR lighthouse calibration into Libsurvive.",
                        )),
                        gtk::Button {
                            set_hexpand: true,
                            set_halign: gtk::Align::Center,
                            add_css_class: "pill",
                            add_css_class: "suggested-action",
                            set_label: "Start Setup",
                            connect_clicked[carousel, page2] => move |_| {
                                carousel.scroll_to(
                                    &page2, true
                                );
                            },
                        }
                    },
                    #[name(page2)]
                    adw::StatusPage {
                        set_hexpand: true,
                        set_vexpand: true,
                        set_title: "Train Your SteamVR Calibration",
                        set_description: Some(concat!(
                            "If you've already played extensively with SteamVR ",
                            "(either on Linux or on Windows), chances are you ",
                            "already have an adequate calibration profile; ",
                            "then you can skip this step.\n\n",
                            "If not, you can plug in your HMD, start SteamVR and ",
                            "slowly move your headset around your play space for ",
                            "a while. This will generate the necessary calibration ",
                            "data that will be imported by Libsurvive.",
                        )),
                        gtk::Button {
                            set_hexpand: true,
                            set_halign: gtk::Align::Center,
                            add_css_class: "pill",
                            add_css_class: "suggested-action",
                            set_label: "Next",
                            connect_clicked[carousel, page3] => move |_| {
                                carousel.scroll_to(
                                    &page3, true
                                );
                            },
                        }
                    },
                    #[name(page3)]
                    adw::StatusPage {
                        set_hexpand: true,
                        set_vexpand: true,
                        set_title: "Import Calibration into Libsurvive",
                        set_description: Some(concat!(
                            "Plug in your HMD and place it on the floor, ",
                            "preferably in the middle of your play space, and ",
                            "in direct line of sight of all of your lighthouses.\n\n",
                            "Then select the SteamVR calibration file from your disk.",
                            "\n\nFinally, you can press the \"Import Calibration\" button.",
                        )),
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 12,
                            adw::Bin {
                                set_hexpand: true,
                                add_css_class: "card",
                                set_halign: gtk::Align::Center,
                                gtk::Grid {
                                    set_column_spacing: 6,
                                    set_row_spacing: 6,
                                    set_margin_all: 12,
                                    set_hexpand: true,
                                    set_halign: gtk::Align::Center,
                                    attach: (
                                        &gtk::Label::builder()
                                            .use_markup(true)
                                            .label("<b>Linux</b>")
                                            .xalign(1.0)
                                            .build(),
                                        0, 0, 1, 1
                                    ),
                                    attach: (
                                        &gtk::Label::builder()
                                            .use_markup(true)
                                            .label("<tt>~/.steam/steam/config/lighthouse/lighthousedb.json</tt>")
                                            .wrap(true)
                                            .wrap_mode(gtk::pango::WrapMode::Char)
                                            .selectable(true)
                                            .xalign(0.0)
                                            .build(),
                                        1, 0, 1, 1
                                    ),
                                    attach: (
                                        &gtk::Label::builder()
                                            .use_markup(true)
                                            .label("<b>Windows</b>")
                                            .xalign(1.0)
                                            .build(),
                                        0, 1, 1, 1
                                    ),
                                    attach: (
                                        &gtk::Label::builder()
                                            .use_markup(true)
                                            .label(
                                                "<tt>\"C:\\Program Files (x86)\\steam\\config\\lighthouse\\lighthouse.json\"</tt>"
                                            )
                                            .wrap(true)
                                            .wrap_mode(gtk::pango::WrapMode::Char)
                                            .selectable(true)
                                            .xalign(0.0)
                                            .build(),
                                        1, 1, 1, 1
                                    ),
                                },
                            },
                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_spacing: 12,
                                set_hexpand: true,
                                gtk::Button {
                                    set_label: "Select File",
                                    connect_clicked[sender] => move |_| {
                                        sender.input(LibsurviveSetupMsg::ChooseFileDialog);
                                    }
                                },
                                gtk::Label {
                                    #[track = "model.changed(LibsurviveSetupWindow::steam_lighthouse_path())"]
                                    set_label: model.steam_lighthouse_path.as_str(),
                                    #[track = "model.changed(LibsurviveSetupWindow::steam_lighthouse_path())"]
                                    set_tooltip_text: Some(model.steam_lighthouse_path.as_str()),
                                    set_ellipsize: gtk::pango::EllipsizeMode::Start,
                                    set_hexpand: true,
                                    set_xalign: 1.0,
                                }
                            },
                            gtk::Button {
                                set_hexpand: true,
                                set_halign: gtk::Align::Center,
                                add_css_class: "pill",
                                add_css_class: "suggested-action",
                                set_label: "Import Calibration",
                                #[track = "model.changed(LibsurviveSetupWindow::steam_lighthouse_path())"]
                                set_sensitive: model.steam_lighthouse_path != NO_FILE_MSG,
                                connect_clicked[sender] => move |_| {
                                    sender.input(
                                        LibsurviveSetupMsg::CalibrateLibsurvive
                                    );
                                },
                            }
                        }
                    },
                    #[name(loading_page)]
                    adw::StatusPage {
                        set_hexpand: true,
                        set_vexpand: true,
                        set_title: "Importing Calibration...\nDo Not Touch your Headset!",
                        set_description: Some("Please stand by"),
                        #[name(progressbar)]
                        gtk::ProgressBar {
                            set_margin_top: 12,
                            set_margin_bottom: 12,
                            set_margin_start: 24,
                            set_margin_end: 24,
                            set_fraction: 0.0,
                            set_hexpand: true,
                        }
                    }
                },
                adw::CarouselIndicatorDots {
                    set_carousel: Some(&carousel),
                }
            }
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        self.reset();

        match message {
            LibsurviveSetupMsg::Present(prof) => {
                self.profile = Some(prof);
                self.win.as_ref().unwrap().present();
            }
            LibsurviveSetupMsg::CalibrateLibsurvive => {
                if self.steam_lighthouse_path == NO_FILE_MSG {
                    return;
                }
                let lh_path = Path::new(&self.steam_lighthouse_path);
                if lh_path.is_file() {
                    self.calibration_runner = Some(Runner::new(
                        None,
                        format!(
                            "{pfx}/bin/survive-cli",
                            pfx = self.profile.as_ref().unwrap().prefix
                        ),
                        vec![
                            "--steamvr-calibration".into(),
                            lh_path.to_str().unwrap().to_string(),
                        ],
                    ));
                    self.carousel
                        .as_ref()
                        .unwrap()
                        .scroll_to(self.loading_page.as_ref().unwrap(), true);
                }
            }
            LibsurviveSetupMsg::SetSteamLighthousePath(n_path) => {
                self.set_steam_lighthouse_path(match n_path {
                    None => NO_FILE_MSG.into(),
                    Some(p) => p,
                });
            }
            LibsurviveSetupMsg::ChooseFileDialog => {
                let chooser = gtk::FileDialog::builder()
                    .modal(true)
                    .title("Select SteamVR Calibration")
                    .filters(&self.filefilter_listmodel)
                    .build();
                let fd_sender = sender.clone();
                chooser.open(
                    Some(&self.win.as_ref().unwrap().clone()),
                    gtk::gio::Cancellable::NONE,
                    move |res| match res {
                        Ok(file) => {
                            let path = file.path();
                            if path.is_some() {
                                fd_sender.input(LibsurviveSetupMsg::SetSteamLighthousePath(Some(
                                    path.unwrap().to_str().unwrap().to_string(),
                                )))
                            }
                        }
                        _ => {}
                    },
                );
            }
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let json_filter = gtk::FileFilter::new();
        json_filter.add_mime_type("application/json");
        let mut model = LibsurviveSetupWindow {
            win: None,
            progressbar: None,
            carousel: None,
            loading_page: None,
            steam_lighthouse_path: NO_FILE_MSG.into(),
            filefilter_listmodel: gtk4::gio::ListStore::builder()
                .item_type(gtk::FileFilter::static_type())
                .build(),
            profile: None,
            calibration_runner: None,
            tracker: 0,
        };

        model.filefilter_listmodel.append(&json_filter);

        let widgets = view_output!();

        model.win = Some(widgets.win.clone());
        model.progressbar = Some(widgets.progressbar.clone());
        model.carousel = Some(widgets.carousel.clone());
        model.loading_page = Some(widgets.loading_page.clone());

        ComponentParts { model, widgets }
    }
}
