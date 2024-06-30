use std::{net::SocketAddr, path::PathBuf};

use crate::serve::service::ScrapsService;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use crate::libs::error::ScrapResult;

pub struct ServeCommand {
    public_dir_path: PathBuf,
}

impl ServeCommand {
    pub fn new(public_dir_path: &PathBuf) -> ServeCommand {
        ServeCommand {
            public_dir_path: public_dir_path.to_owned(),
        }
    }

    #[tokio::main]
    pub async fn run(&self, addr: &SocketAddr) -> ScrapResult<()> {
        let listener = TcpListener::bind(&addr).await?;
        println!("\nListening on http://{addr}\n");

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);

            let service = ScrapsService::new(&self.public_dir_path);
            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                    println!("Failed to serve connection: {err:?}");
                }
            });
        }
    }
}
