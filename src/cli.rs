use clap::{Parser, Subcommand};

pub mod cmd;
mod scrap_config;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: SubCommands,
}

#[derive(Subcommand)]
pub enum SubCommands {
    #[command(about = "Build scraps")]
    Build,

    #[command(about = "Init scraps project")]
    Init { project_name: String },

    #[command(about = "Serve the site. result of build.")]
    Serve,
}
