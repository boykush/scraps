use std::{net::SocketAddr, path::PathBuf};

use crate::{libs::error::result::ScrapResult, serve::cmd::ServeCommand};

pub fn run() -> ScrapResult<()> {
    let public_dir_path = PathBuf::from("public");
    let command = ServeCommand::new(&public_dir_path);

    let addr: SocketAddr = ([127, 0, 0, 1], 1112).into();
    command.run(&addr)
}
