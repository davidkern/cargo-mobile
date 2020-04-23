#![forbid(unsafe_code)]

use cargo_mobile::{
    init, opts,
    util::{
        self,
        cli::{self, Exec, GlobalFlags, Report, Reportable, TextWrapper},
    },
    NAME,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(bin_name = cli::bin_name(NAME), settings = cli::SETTINGS)]
pub struct Input {
    #[structopt(flatten)]
    flags: GlobalFlags,
    #[structopt(subcommand)]
    command: Command,
}

#[derive(Clone, Debug, StructOpt)]
pub enum Command {
    #[structopt(
        name = "init",
        about = "Creates a new project in the current working directory"
    )]
    Init {
        #[structopt(flatten)]
        clobbering: cli::Clobbering,
        #[structopt(
            long,
            about = "Open in default code editor",
            parse(from_flag = opts::OpenIn::from_flag),
        )]
        open: opts::OpenIn,
        #[structopt(
            long,
            about = "Only do some steps",
            value_name = "STEPS",
            possible_values = init::STEPS,
            value_delimiter(" "),
        )]
        only: Option<Vec<String>>,
        #[structopt(
            long,
            about = "Skip some steps",
            value_name = "STEPS",
            possible_values = init::STEPS,
            value_delimiter(" "),
        )]
        skip: Option<Vec<String>>,
    },
    #[structopt(name = "open", about = "Open project in default code editor")]
    Open,
    #[structopt(name = "update", about = "Update `cargo-mobile`")]
    Update,
}

#[derive(Debug)]
pub enum Error {
    InitFailed(init::Error),
    OpenFailed(util::OpenInEditorError),
    UpdateFailed(bossy::Error),
}

impl Reportable for Error {
    fn report(&self) -> Report {
        match self {
            Self::InitFailed(err) => err.report(),
            Self::OpenFailed(err) => {
                Report::error("Failed to open project in default code editor", err)
            }
            Self::UpdateFailed(err) => Report::error("Failed to update `cargo-mobile`", err),
        }
    }
}

impl Exec for Input {
    type Report = Error;

    fn global_flags(&self) -> GlobalFlags {
        self.flags
    }

    fn exec(self, wrapper: &TextWrapper) -> Result<(), Self::Report> {
        let Self {
            flags: GlobalFlags { interactivity, .. },
            command,
        } = self;
        match command {
            Command::Init {
                clobbering: cli::Clobbering { clobbering },
                open,
                only,
                skip,
            } => init::exec(interactivity, clobbering, open, only, skip, wrapper)
                .map(|_| ())
                .map_err(Error::InitFailed),
            Command::Open => util::open_in_editor(".").map_err(Error::OpenFailed),
            Command::Update => bossy::Command::impure("cargo")
                .with_args(&[
                    "install",
                    "--force",
                    "--git",
                    "ssh://git@bitbucket.org/brainium/cargo-mobile.git",
                ])
                .run_and_wait()
                .map(|_| ())
                .map_err(Error::UpdateFailed),
        }
    }
}

fn main() {
    cli::exec::<Input>(NAME)
}
