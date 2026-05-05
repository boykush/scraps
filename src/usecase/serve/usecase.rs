use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};

use crate::usecase::serve::service::ScrapsService;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use crate::error::ScrapsResult;

pub struct ServeUsecase {
    output_dir_path: PathBuf,
}

impl ServeUsecase {
    pub fn new(output_dir_path: &Path) -> ServeUsecase {
        ServeUsecase {
            output_dir_path: output_dir_path.to_path_buf(),
        }
    }

    #[tokio::main]
    pub async fn execute(&self, addr: &SocketAddr) -> ScrapsResult<()> {
        let listener = TcpListener::bind(&addr).await?;

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);

            let service = ScrapsService::new(&self.output_dir_path);
            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                    println!("Failed to serve connection: {err:?}");
                }
            });
        }
    }
}
