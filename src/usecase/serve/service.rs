use std::{
    fs::File,
    future::Future,
    io::Read,
    path::{Component, Path, PathBuf},
    pin::Pin,
    str::Utf8Error,
};

use crate::error::{ScrapsError, ServeError, anyhow::Context};
use http_body_util::Full;
use hyper::{
    Request, Response,
    body::{Bytes, Incoming},
    header,
    service::Service,
};
use percent_encoding::percent_decode_str;

#[derive(Clone)]
pub struct ScrapsService {
    pub output_dir_path: PathBuf,
}

impl ScrapsService {
    pub fn new(output_dir_path: &Path) -> ScrapsService {
        ScrapsService {
            output_dir_path: output_dir_path.to_owned(),
        }
    }

    fn mk_response(
        mime_type: &str,
        contents: Vec<u8>,
    ) -> Result<Response<Full<Bytes>>, ScrapsError> {
        Ok(Response::builder()
            .header(header::CONTENT_TYPE, mime_type)
            .body(Full::new(Bytes::from(contents)))
            .unwrap())
    }

    fn mk_not_found_response() -> Result<Response<Full<Bytes>>, ScrapsError> {
        // Return the 404 Not Found for other routes, and don't increment counter.
        Self::mk_response("text/plain", "oh no! not found".into())
    }

    fn mk_failed_url_decode_response() -> Result<Response<Full<Bytes>>, ScrapsError> {
        Self::mk_response("text/plain", "oh no! failed url decode by utf8".into())
    }

    fn mk_page_response(
        mime_type: &str,
        file: &mut File,
    ) -> Result<Response<Full<Bytes>>, ScrapsError> {
        let mut contents = Vec::new();
        let read = file
            .read_to_end(&mut contents)
            .context(ServeError::LoadFile);

        match read {
            Ok(_) => Self::mk_response(mime_type, contents),
            _ => Self::mk_not_found_response(),
        }
    }

    /// The file a request path names. Page paths are names as written, so
    /// every segment is percent-decoded, directories included. A segment that
    /// decodes to anything but one plain name is refused: `%2E%2E` or `%2F`
    /// would otherwise walk out of the output directory.
    fn resolve_file_path(&self, request_path: &str) -> Result<Option<PathBuf>, Utf8Error> {
        let mut file_path = self.output_dir_path.clone();
        for segment in request_path.split('/').filter(|s| !s.is_empty()) {
            let name = percent_decode_str(segment).decode_utf8()?;
            let mut components = Path::new(name.as_ref()).components();
            match (components.next(), components.next()) {
                (Some(Component::Normal(_)), None) => file_path.push(name.as_ref()),
                _ => return Ok(None),
            }
        }
        if file_path.is_dir() {
            file_path.push("index.html");
        }
        Ok(Some(file_path))
    }

    fn gen_mime_type_from(file_path: &Path) -> &str {
        match file_path.extension().and_then(|ext| ext.to_str()) {
            Some("html") => "text/html",
            Some("js") => "application/javascript",
            Some("json") => "text/json",
            Some("css") => "text/css",
            Some("wasm") => "application/wasm",
            _ => "application/octet-stream",
        }
    }
}

impl Service<Request<Incoming>> for ScrapsService {
    type Response = Response<Full<Bytes>>;
    type Error = ScrapsError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, request: Request<Incoming>) -> Self::Future {
        let result = match self.resolve_file_path(request.uri().path()) {
            Ok(Some(file_path)) => {
                let file = File::open(&file_path).context(ServeError::LoadFile);
                let mime_type = Self::gen_mime_type_from(file_path.as_path());
                match file {
                    Ok(mut f) => Self::mk_page_response(mime_type, &mut f),
                    Err(_) => Self::mk_not_found_response(),
                }
            }
            Ok(None) => Self::mk_not_found_response(),
            Err(_) => Self::mk_failed_url_decode_response(),
        };

        Box::pin(async { result })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use rstest::rstest;

    use super::*;
    use crate::test_fixtures::{TempScrapProject, temp_scrap_project};

    #[rstest]
    #[case::plain("/scraps/plain.html", "scraps/plain.html")]
    #[case::space(
        "/scraps/Reference/CLI%20Overview.html",
        "scraps/Reference/CLI Overview.html"
    )]
    #[case::question("/scraps/What%20is%20it%3F.html", "scraps/What is it?.html")]
    #[case::hash("/scraps/C%23.html", "scraps/C#.html")]
    #[case::percent("/scraps/100%25%20sure.html", "scraps/100% sure.html")]
    #[case::unicode("/scraps/%E6%97%A5%E6%9C%AC%E8%AA%9E.html", "scraps/日本語.html")]
    #[case::unencoded_unicode("/scraps/日本語.html", "scraps/日本語.html")]
    #[case::encoded_directory("/scraps/My%20Notes/a.html", "scraps/My Notes/a.html")]
    #[case::directory_index("/types/Big%20Table/", "types/Big Table/index.html")]
    #[case::directory_index_without_slash("/types/Big%20Table", "types/Big Table/index.html")]
    #[case::escaped_on_disk("/types/ADR%252FAccepted/", "types/ADR%2FAccepted/index.html")]
    #[case::root("/", "index.html")]
    fn it_resolves_a_percent_encoded_request_to_its_file(
        #[from(temp_scrap_project)] project: TempScrapProject,
        #[case] request_path: &str,
        #[case] file: &str,
    ) {
        let expected = project.output_dir.join(file);
        fs::create_dir_all(expected.parent().unwrap()).unwrap();
        fs::write(&expected, "page").unwrap();

        let service = ScrapsService::new(&project.output_dir);

        assert_eq!(
            service.resolve_file_path(request_path).unwrap(),
            Some(expected)
        );
    }

    #[rstest]
    #[case::parent("/../secret.html")]
    #[case::encoded_parent("/%2E%2E/secret.html")]
    #[case::encoded_separator("/scraps%2F..%2F..%2Fsecret.html")]
    #[case::encoded_nested_parent("/scraps/%2e%2e/%2e%2e/secret.html")]
    fn it_refuses_a_request_that_leaves_the_output_directory(
        #[from(temp_scrap_project)] project: TempScrapProject,
        #[case] request_path: &str,
    ) {
        let service = ScrapsService::new(&project.output_dir);

        assert_eq!(service.resolve_file_path(request_path).unwrap(), None);
    }

    #[rstest]
    fn it_reports_a_path_that_is_not_utf8(#[from(temp_scrap_project)] project: TempScrapProject) {
        let service = ScrapsService::new(&project.output_dir);

        assert!(service.resolve_file_path("/scraps/%FF.html").is_err());
    }
}
