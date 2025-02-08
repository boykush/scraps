use clap::{Parser, Subcommand};
use clap_verbosity_flag::{Verbosity, WarnLevel};

pub mod cmd;
mod config;
mod display;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: SubCommands,
}

#[derive(Subcommand)]
pub enum SubCommands {
    #[command(about = "Build scraps")]
    Build {
        #[command(flatten)]
        verbose: Verbosity<WarnLevel>,
    },

    #[command(about = "Init scraps project")]
    Init { project_name: String },

    #[command(about = "Serve the site with build scraps.")]
    Serve,

    #[command(about = "List a tags")]
    Tag,
}
