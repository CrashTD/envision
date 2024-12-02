use crate::config::Config;
use gtk::{
    gio::{
        prelude::{ApplicationCommandLineExt, ApplicationExt},
        Application, ApplicationCommandLine,
    },
    glib::{self, prelude::IsA},
};
use tracing::error;

#[derive(Debug, Clone)]
pub struct CmdLineOpts {
    pub start: bool,
    pub list_profiles: bool,
    pub profile_uuid: Option<String>,
    pub skip_depcheck: bool,
    pub check_dependencies_for: Option<String>,
}

impl CmdLineOpts {
    const OPT_START: (&'static str, char) = ("start", 'S');
    const OPT_LIST_PROFILES: (&'static str, char) = ("list-profiles", 'l');
    const OPT_PROFILE: (&'static str, char) = ("profile", 'p');
    const OPT_SKIP_DEPCHECK: (&'static str, char) = ("skip-dependency-check", 'd');
    const OPT_CHECK_DEPS_FOR: (&'static str, char) = ("check-deps-for", 'c');

    pub fn init(app: &impl IsA<Application>) {
        app.add_main_option(
            Self::OPT_START.0,
            glib::Char::try_from(Self::OPT_START.1).unwrap(),
            glib::OptionFlags::IN_MAIN,
            glib::OptionArg::None,
            "Start the XR Service right away",
            None,
        );
        app.add_main_option(
            Self::OPT_LIST_PROFILES.0,
            glib::Char::try_from(Self::OPT_LIST_PROFILES.1).unwrap(),
            glib::OptionFlags::IN_MAIN,
            glib::OptionArg::None,
            "List the available profiles",
            None,
        );
        app.add_main_option(
            Self::OPT_PROFILE.0,
            glib::Char::try_from(Self::OPT_PROFILE.1).unwrap(),
            glib::OptionFlags::IN_MAIN,
            glib::OptionArg::String,
            "Switch to the profile indicated by the UUID",
            None,
        );
        app.add_main_option(
            Self::OPT_SKIP_DEPCHECK.0,
            glib::Char::try_from(Self::OPT_SKIP_DEPCHECK.1).unwrap(),
            glib::OptionFlags::IN_MAIN,
            glib::OptionArg::None,
            "Skip dependency checks when building profiles",
            None,
        );
        app.add_main_option(
            Self::OPT_CHECK_DEPS_FOR.0,
            glib::Char::try_from(Self::OPT_CHECK_DEPS_FOR.1).unwrap(),
            glib::OptionFlags::IN_MAIN,
            glib::OptionArg::String,
            "Prints missing dependencies for given profile id; returns nothing if all dependencies are satisfied",
            None,
        );
    }

    /// returns an exit code if the application should quit immediately
    pub fn handle_non_activating_opts(&self) -> Option<i32> {
        if self.list_profiles {
            println!("Available profiles\nUUID: \"name\"");
            let profiles = Config::get_config().profiles();
            profiles.iter().for_each(|p| {
                println!("{}: \"{}\"", p.uuid, p.name);
            });
            return Some(0);
        }
        if let Some(prof_id) = self.check_dependencies_for.as_ref() {
            let profiles = Config::get_config().profiles();
            if let Some(prof) = profiles.iter().find(|p| &p.uuid == prof_id) {
                let deps = prof.missing_dependencies();
                if deps.is_empty() {
                    return Some(0);
                }
                for dep in deps {
                    println!("{}", dep.name);
                }
                return Some(1);
            } else {
                error!("No profile found for uuid: `{prof_id}`");
                return Some(404);
            }
        }
        None
    }

    pub fn from_cmdline(cmdline: &ApplicationCommandLine) -> Self {
        let opts = cmdline.options_dict();
        Self {
            start: opts.contains(Self::OPT_START.0),
            list_profiles: opts.contains(Self::OPT_LIST_PROFILES.0),
            profile_uuid: opts
                .lookup::<String>(Self::OPT_PROFILE.0)
                .unwrap_or_default(),
            skip_depcheck: opts.contains(Self::OPT_SKIP_DEPCHECK.0),
            check_dependencies_for: opts
                .lookup::<String>(Self::OPT_CHECK_DEPS_FOR.0)
                .unwrap_or_default(),
        }
    }
}
