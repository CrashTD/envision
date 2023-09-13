use crate::{cmd_runner::CmdRunner, profile::Profile, runner::Runner};
use adw::prelude::*;
use gtk::glib;
use relm4::prelude::*;
use std::{cell::Cell, collections::HashMap, path::Path, rc::Rc, time::Duration};

use super::alert::alert;

const NO_FILE_MSG: &str = "(No file selected)";
const CALIBRATION_RUN_TIME_SECONDS: f64 = 30.0;

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
    calibration_done_page: Option<adw::StatusPage>,
    #[tracker::do_not_track]
    calibration_cancelled_page: Option<adw::StatusPage>,
    #[tracker::do_not_track]
    page1: Option<adw::StatusPage>,
    #[tracker::do_not_track]
    filefilter_listmodel: gtk::gio::ListStore,

    #[tracker::do_not_track]
    calibration_running: Rc<Cell<bool>>,
    #[tracker::do_not_track]
    first_run_done: bool,
    #[tracker::do_not_track]
    calibration_seconds_elapsed: f64,

    #[tracker::do_not_track]
    profile: Option<Profile>,
    #[tracker::do_not_track]
    calibration_runner: Option<CmdRunner>,
}

#[derive(Debug)]
pub enum LibsurviveSetupMsg {
    Present(Profile),
    CalibrateLibsurvive,
    SetSteamLighthousePath(Option<String>),
    ChooseFileDialog,
    TickCalibrationRunner,
    Cancel,
}

impl LibsurviveSetupWindow {
    fn create_calibration_runner(&mut self, survive_cli_path: String) -> CmdRunner {
        let lh_path = self.steam_lighthouse_path.clone();
        let mut env = HashMap::new();
        env.insert(
            "LD_LIBRARY_PATH".to_string(),
            format!("{pfx}/lib", pfx = self.profile.as_ref().unwrap().prefix),
        );
        CmdRunner::new(
            Some(env),
            survive_cli_path,
            vec!["--steamvr-calibration".into(), lh_path],
        )
    }

    fn on_close_request(
        sender: &relm4::ComponentSender<Self>,
        carousel: &adw::Carousel,
        calibration_running: &Rc<Cell<bool>>,
        progressbar: &gtk::ProgressBar,
        page1: &adw::StatusPage,
    ) -> gtk::glib::Propagation {
        if calibration_running.get() {
            return gtk::glib::Propagation::Stop;
        }
        carousel.scroll_to(page1, true);
        progressbar.set_fraction(0.0);
        sender.input(LibsurviveSetupMsg::SetSteamLighthousePath(None));
        gtk::glib::Propagation::Proceed
    }
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
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 24,
                            set_hexpand: true,
                            #[name(progressbar)]
                            gtk::ProgressBar {
                                set_margin_top: 12,
                                set_margin_bottom: 12,
                                set_margin_start: 24,
                                set_margin_end: 24,
                                set_fraction: 0.0,
                                set_hexpand: true,
                            },
                            gtk::Button {
                                add_css_class: "destructive-action",
                                add_css_class: "pill",
                                set_label: "Cancel",
                                connect_clicked[sender] => move |_| {
                                    sender.input(Self::Input::Cancel)
                                }
                            },
                        }
                    },
                    #[name(calibration_cancelled_page)]
                    adw::StatusPage {
                        set_hexpand: true,
                        set_vexpand: true,
                        set_title: "Calibration Import Cancelled",
                        set_description: Some(
                            "You can re-run the calibration process any time.",
                        ),
                        gtk::Button {
                            set_hexpand: true,
                            set_halign: gtk::Align::Center,
                            add_css_class: "pill",
                            add_css_class: "suggested-action",
                            set_label: "Ok",
                            connect_clicked[win] => move |_| {
                                win.close();
                            },
                        }
                    },
                    #[name(calibration_done_page)]
                    adw::StatusPage {
                        set_hexpand: true,
                        set_vexpand: true,
                        set_title: "Calibration Imported",
                        set_description: Some(concat!(
                            "If you experience bad quality tracking in the form ",
                            "of jittering or general floatiness, you probably ",
                            "need to refine your SteamVR calibration, then ",
                            "import it again into Libsurvive.",
                        )),
                        gtk::Button {
                            set_hexpand: true,
                            set_halign: gtk::Align::Center,
                            add_css_class: "pill",
                            add_css_class: "suggested-action",
                            set_label: "I Understand",
                            connect_clicked[win] => move |_| {
                                win.close();
                            },
                        }
                    },
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
            Self::Input::Present(prof) => {
                self.profile = Some(prof);
                self.win.as_ref().unwrap().present();
            }
            Self::Input::CalibrateLibsurvive => {
                if self.steam_lighthouse_path == NO_FILE_MSG {
                    return;
                }
                let path_s = self.steam_lighthouse_path.clone();
                let lh_path = Path::new(&path_s);
                if lh_path.is_file() {
                    if let Some(survive_cli_path) =
                        self.profile.as_ref().unwrap().get_survive_cli_path()
                    {
                        let mut runner = self.create_calibration_runner(survive_cli_path);
                        self.calibration_running.set(true);
                        self.first_run_done = false;
                        self.calibration_seconds_elapsed = 0.0;
                        runner.start();
                        {
                            let timer_sender = sender.clone();
                            let cont = self.calibration_running.clone();
                            glib::timeout_add_local(Duration::from_millis(1000), move || {
                                timer_sender.input(LibsurviveSetupMsg::TickCalibrationRunner);
                                match cont.get() {
                                    true => glib::ControlFlow::Continue,
                                    false => glib::ControlFlow::Break,
                                }
                            });
                        }
                        self.calibration_runner = Some(runner);
                        self.carousel
                            .as_ref()
                            .unwrap()
                            .scroll_to(self.loading_page.as_ref().unwrap(), true);
                    } else {
                        let parent = self.win.as_ref().map(|w| w.clone().upcast::<gtk::Window>());
                        alert(
                            "Survive CLI not found",
                            Some(concat!(
                                "You might need to build this profile first, ",
                                "or maybe Libsurvive isn't enabled for this profile"
                            )),
                            parent.as_ref(),
                        );
                    }
                }
            }
            Self::Input::SetSteamLighthousePath(n_path) => {
                self.set_steam_lighthouse_path(match n_path {
                    None => NO_FILE_MSG.into(),
                    Some(p) => p,
                });
            }
            Self::Input::ChooseFileDialog => {
                let chooser = gtk::FileDialog::builder()
                    .modal(true)
                    .title("Select SteamVR Calibration")
                    .filters(&self.filefilter_listmodel)
                    .build();
                let fd_sender = sender.clone();
                chooser.open(
                    Some(&self.win.as_ref().unwrap().clone()),
                    gtk::gio::Cancellable::NONE,
                    move |res| {
                        if let Ok(file) = res {
                            if let Some(path) = file.path() {
                                fd_sender.input(LibsurviveSetupMsg::SetSteamLighthousePath(Some(
                                    path.to_str().unwrap().to_string(),
                                )))
                            }
                        }
                    },
                );
            }
            Self::Input::TickCalibrationRunner => {
                self.calibration_seconds_elapsed += 1.0;
                self.progressbar.as_ref().unwrap().set_fraction(
                    (self.calibration_seconds_elapsed / (CALIBRATION_RUN_TIME_SECONDS * 2.0))
                        .min(1.0),
                );
                if !self.first_run_done
                    && self.calibration_seconds_elapsed >= CALIBRATION_RUN_TIME_SECONDS
                {
                    if self.calibration_runner.is_some() {
                        self.first_run_done = true;
                        let runner = self.calibration_runner.as_mut().unwrap();
                        runner.terminate();
                        let mut n_runner = self.create_calibration_runner(
                            self.profile
                                .as_ref()
                                .unwrap()
                                .get_survive_cli_path()
                                .unwrap(),
                        );
                        n_runner.start();
                        self.calibration_runner = Some(n_runner);
                    }
                } else if self.calibration_seconds_elapsed >= (CALIBRATION_RUN_TIME_SECONDS * 2.0) {
                    let runner = self.calibration_runner.as_mut().unwrap();
                    runner.terminate();
                    self.calibration_running.set(false);
                    self.carousel
                        .as_ref()
                        .unwrap()
                        .scroll_to(self.calibration_done_page.as_ref().unwrap(), true);
                }
            }
            Self::Input::Cancel => {
                if self.calibration_running.get() {
                    if let Some(runner) = self.calibration_runner.as_mut() {
                        runner.terminate();
                    }
                    self.calibration_running.set(false);
                    self.carousel
                        .as_ref()
                        .unwrap()
                        .scroll_to(self.calibration_cancelled_page.as_ref().unwrap(), true);
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let json_filter = gtk::FileFilter::new();
        json_filter.add_mime_type("application/json");

        let mut model = LibsurviveSetupWindow {
            win: None,
            progressbar: None,
            carousel: None,
            page1: None,
            loading_page: None,
            calibration_done_page: None,
            calibration_cancelled_page: None,
            steam_lighthouse_path: NO_FILE_MSG.into(),
            filefilter_listmodel: gtk4::gio::ListStore::builder()
                .item_type(gtk::FileFilter::static_type())
                .build(),
            profile: None,
            calibration_runner: None,
            calibration_running: Rc::new(Cell::new(false)),
            first_run_done: false,
            calibration_seconds_elapsed: 0.0,
            tracker: 0,
        };

        model.filefilter_listmodel.append(&json_filter);

        let widgets = view_output!();

        model.win = Some(widgets.win.clone());
        model.progressbar = Some(widgets.progressbar.clone());
        model.carousel = Some(widgets.carousel.clone());
        model.loading_page = Some(widgets.loading_page.clone());
        model.calibration_done_page = Some(widgets.calibration_done_page.clone());
        model.calibration_cancelled_page = Some(widgets.calibration_cancelled_page.clone());
        model.page1 = Some(widgets.page1.clone());

        {
            let cr_sender = sender.clone();
            let cr_carousel = widgets.carousel.clone();
            let cr_calibration_running = model.calibration_running.clone();
            let cr_progressbar = widgets.progressbar.clone();
            let cr_page1 = widgets.page1.clone();
            model.win.as_ref().unwrap().connect_close_request(move |_| {
                Self::on_close_request(
                    &cr_sender,
                    &cr_carousel,
                    &cr_calibration_running,
                    &cr_progressbar,
                    &cr_page1,
                )
            });
        };

        ComponentParts { model, widgets }
    }
}
