use crate::config::Config;
use gtk4::{
    gio::{
        prelude::{ApplicationCommandLineExt, ApplicationExt},
        Application, ApplicationCommandLine,
    },
    glib::{self, IsA},
};

#[derive(Debug, Clone)]
pub struct CmdLineOpts {
    pub start: bool,
    pub profile_uuid: Option<String>,
    pub skip_depcheck: bool,
}

impl CmdLineOpts {
    const OPT_START: (&'static str, char) = ("start", 'S');
    const OPT_LIST_PROFILES: (&'static str, char) = ("list-profiles", 'l');
    const OPT_PROFILE: (&'static str, char) = ("profile", 'p');
    const OPT_SKIP_DEPCHECK: (&'static str, char) = ("skip-dependency-check", 'd');

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
    }

    /// returns true if the application should quit
    pub fn handle_non_activating_opts(cmdline: &ApplicationCommandLine) -> bool {
        if cmdline.options_dict().contains(Self::OPT_LIST_PROFILES.0) {
            println!("Available profiles\nUUID: \"name\"");
            let profiles = Config::get_config().profiles();
            profiles.iter().for_each(|p| {
                println!("{}: \"{}\"", p.uuid, p.name);
            });
            return true;
        }
        false
    }

    pub fn from_cmdline(cmdline: &ApplicationCommandLine) -> Self {
        let opts = cmdline.options_dict();
        Self {
            start: opts.contains(Self::OPT_START.0),
            profile_uuid: match opts.lookup::<String>(Self::OPT_PROFILE.0) {
                Err(_) => None,
                Ok(None) => None,
                Ok(Some(variant)) => Some(variant),
            },
            skip_depcheck: opts.contains(Self::OPT_SKIP_DEPCHECK.0),
        }
    }
}
