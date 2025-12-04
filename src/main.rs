use anyhow::Result;
use constants::{APP_ID, APP_NAME, GETTEXT_PACKAGE, LOCALE_DIR, RESOURCES_BASE_PATH, resources};
use file_builders::{
    active_runtime_json::restore_active_runtime_backup,
    openvrpaths_vrpath::{get_current_openvrpaths, set_current_openvrpaths_to_steam},
};
use gettextrs::LocaleCategory;
use paths::get_logs_dir;
use relm4::{
    MessageBroker, RelmApp, adw,
    gtk::{self, gdk, gio, glib, prelude::*},
};
use std::{
    env,
    fs::{read_dir, remove_file},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};
use steam_linux_runtime_injector::{
    restore_sniper_runtime_entrypoint, restore_soldier_runtime_entrypoint,
};
use tracing::{error, warn};
use tracing_subscriber::{
    EnvFilter, Layer, filter::LevelFilter, layer::SubscriberExt, util::SubscriberInitExt,
};
use ui::{
    app::{App, AppInit, Msg},
    cmdline_opts::CmdLineOpts,
};

pub mod async_process;
pub mod build_tools;
pub mod builders;
pub mod cmd_runner;
pub mod config;
#[rustfmt::skip]
pub mod constants;
pub mod depcheck;
pub mod device_prober;
pub mod downloader;
pub mod env_var_descriptions;
pub mod file_builders;
pub mod gpu_profile;
pub mod is_appimage;
pub mod linux_distro;
pub mod log_parser;
pub mod openxr_prober;
pub mod paths;
pub mod profile;
pub mod profiles;
pub mod steam_linux_runtime_injector;
pub mod termcolor;
pub mod ui;
pub mod util;
pub mod vulkaninfo;
pub mod wivrn_dbus;
pub mod xdg;
pub mod xr_devices;

fn restore_steam_xr_files() {
    let openvrpaths = get_current_openvrpaths();
    if let Err(e) = restore_active_runtime_backup() {
        warn!("failed to restore active runtime to steam: {e}");
    }
    if let Some(ovrp) = openvrpaths
        && !file_builders::openvrpaths_vrpath::is_steam(&ovrp)
        && let Err(e) = set_current_openvrpaths_to_steam()
    {
        warn!("failed to restore openvrpaths to steam: {e}");
    }
    restore_sniper_runtime_entrypoint();
    restore_soldier_runtime_entrypoint();
}

const LOGS_MAX_SIZE_BYTES: u64 = 1000000000; // 1GB

fn remove_old_logs(dir: &Path, log_files: Option<Vec<PathBuf>>) -> anyhow::Result<()> {
    let log_files: Vec<PathBuf> = log_files
        .map::<anyhow::Result<Vec<PathBuf>>, _>(Ok)
        .unwrap_or_else(|| {
            let mut files: Vec<PathBuf> = read_dir(dir)?
                .filter_map(|de| {
                    let p = de.ok()?.path();
                    if p.is_file() && !p.is_symlink() {
                        Some(p)
                    } else {
                        None
                    }
                })
                .collect();
            files.sort_unstable();
            Ok(files)
        })?;
    let total_size = log_files
        .iter()
        .filter_map(|p| Some(p.metadata().ok()?.size()))
        .reduce(u64::saturating_add)
        .unwrap_or(0);
    // if size is under threshold, finish
    if total_size < LOGS_MAX_SIZE_BYTES {
        return Ok(());
    }
    // keep a minimum of 3 logs
    if log_files.len() <= 3 {
        return Ok(());
    }

    remove_file(log_files.first().ok_or_else(||
        anyhow::Error::msg(
            "Could not get first item in log files list, but they should be more than 3! This is a bug!"
        )
    )?)?;

    remove_old_logs(dir, Some(log_files))
}

fn main() -> Result<()> {
    if env::var("USER").unwrap_or_else(|_| env::var("USERNAME").unwrap_or_default()) == "root" {
        panic!("{APP_NAME} cannot run as root");
    }
    restore_steam_xr_files();
    // deferring error logging for this since tracing isn't initialized yet
    let old_logs_removal_res = remove_old_logs(&get_logs_dir(), None);

    let rolling_log_writer = tracing_appender::rolling::daily(get_logs_dir(), "log");
    let (non_blocking_appender, _appender_guard) =
        tracing_appender::non_blocking(rolling_log_writer);
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer().pretty().with_filter(
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::INFO.into())
                    .from_env_lossy(),
            ),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_writer(non_blocking_appender),
        )
        .init();

    if let Err(e) = old_logs_removal_res {
        error!("Failed to remove old log files: {e}");
    }

    // Prepare i18n
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, LOCALE_DIR).expect("Unable to bind the text domain");
    gettextrs::textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    gtk::init()?;
    glib::set_application_name(APP_NAME);

    {
        let res = gio::Resource::load(resources()).expect("Could not load gresource file");
        gio::resources_register(&res);
    }

    let provider = gtk::CssProvider::new();
    provider.load_from_resource(&format!("{RESOURCES_BASE_PATH}/style.css"));
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let icon_theme = gtk::IconTheme::for_display(&display);
        icon_theme.add_resource_path(&format!("{RESOURCES_BASE_PATH}/icons/"));
    }
    gtk::Window::set_default_icon_name(APP_ID);

    let main_app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .resource_base_path(format!("/{}", APP_ID.replace('.', "/")))
        .build();

    static BROKER: MessageBroker<Msg> = MessageBroker::new();
    CmdLineOpts::init(&main_app);
    let sender = BROKER.sender();
    main_app.connect_command_line(move |this, cmdline| {
        let opts = CmdLineOpts::from_cmdline(cmdline);
        if let Some(exit_code) = opts.handle_non_activating_opts() {
            this.quit();
            return exit_code;
        }
        this.activate();
        sender.emit(Msg::HandleCommandLine(opts));
        0
    });
    let app = RelmApp::from_app(main_app.clone()).with_broker(&BROKER);
    app.run_async::<App>(AppInit {
        application: main_app,
    });
    Ok(())
}
