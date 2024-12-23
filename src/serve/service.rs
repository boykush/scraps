use std::{fs::File, future::Future, io::Read, path::PathBuf, pin::Pin};

use http_body_util::Full;
use hyper::{
    body::{Bytes, Incoming},
    service::Service,
    Request, Response,
};
use percent_encoding::percent_decode_str;
use scraps_libs::error::{anyhow::Context, ScrapError};

#[derive(Clone)]
pub struct ScrapsService {
    pub public_dir_path: PathBuf,
}

impl ScrapsService {
    pub fn new(public_dir_path: &PathBuf) -> ScrapsService {
        ScrapsService {
            public_dir_path: public_dir_path.to_owned(),
        }
    }

    fn mk_response(s: String) -> Result<Response<Full<Bytes>>, ScrapError> {
        Ok(Response::builder().body(Full::new(Bytes::from(s))).unwrap())
    }

    fn mk_not_found_response() -> Result<Response<Full<Bytes>>, ScrapError> {
        // Return the 404 Not Found for other routes, and don't increment counter.
        Self::mk_response("oh no! not found".into())
    }

    fn mk_failed_url_decode_response() -> Result<Response<Full<Bytes>>, ScrapError> {
        Self::mk_response("oh no! failed url decode by utf8".into())
    }

    fn mk_page_response(file: &mut File) -> Result<Response<Full<Bytes>>, ScrapError> {
        let mut contents = String::new();
        let read = file
            .read_to_string(&mut contents)
            .context(ScrapError::FileLoad);

        match read {
            Ok(_) => Self::mk_response(contents),
            _ => Self::mk_not_found_response(),
        }
    }
}

impl Service<Request<Incoming>> for ScrapsService {
    type Response = Response<Full<Bytes>>;
    type Error = ScrapError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, request: Request<Incoming>) -> Self::Future {
        let requested_path = request.uri().path().replacen('/', "", 1); // remove head absolute slash;
        let allowed_path = self.public_dir_path.join(requested_path);
        let resolved_index_path = if allowed_path.is_dir() {
            allowed_path.join("index.html")
        } else {
            allowed_path
        };
        let decoded_file_name = {
            let file_name = resolved_index_path
                .file_name()
                .map_or("", |s: &std::ffi::OsStr| s.to_str().expect(""));
            percent_decode_str(file_name).decode_utf8()
        };
        let result = match decoded_file_name {
            Ok(name) => {
                let file_path = resolved_index_path.with_file_name(name.to_string());
                let file = File::open(file_path).context(ScrapError::FileLoad);
                match file {
                    Ok(mut f) => Self::mk_page_response(&mut f),
                    Err(_) => Self::mk_not_found_response(),
                }
            }
            Err(_) => Self::mk_failed_url_decode_response(),
        };

        Box::pin(async { result })
    }
}
