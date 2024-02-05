use gtk4::{
    gio::{prelude::ApplicationExt, Application, ApplicationCommandLine},
    glib::{self, IsA},
};
use zoha_vte4::ApplicationCommandLineExt;

#[derive(Debug, Clone)]
pub struct CmdLineOpts {
    pub start: bool,
}

impl CmdLineOpts {
    pub fn init(app: &impl IsA<Application>) {
        app.add_main_option(
            "start",
            glib::Char::try_from('S').unwrap(),
            glib::OptionFlags::IN_MAIN,
            glib::OptionArg::None,
            "Start the XR Service right away",
            None,
        );
    }

    pub fn from_cmdline(cmdline: &ApplicationCommandLine) -> Self {
        Self {
            start: cmdline.options_dict().contains("start"),
        }
    }
}
