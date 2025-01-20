use crate::http::{Encoding, HttpContentType, HttpStatus};
use crate::CRLF;
use flate2::{write::GzEncoder, Compression};
use logger::debug;
use std::io::Write;
use badserde::json::ToJson;

// #[derive(Default)]
pub struct Response {
    pub status: HttpStatus,
    headers: Vec<String>,
    pub content: String,
    pub content_type: HttpContentType,
    encoding: Option<Encoding>,
}

pub fn new(status: HttpStatus, content: String, content_type: HttpContentType) -> Response {
    Response {
        status,
        headers: Vec::new(),
        content,
        content_type,
        encoding: None,
    }
}

pub fn html(status: HttpStatus, content: String) -> Response {
    return new(status, content, HttpContentType::HTML);
}

pub fn json<T: ToJson>(status: HttpStatus, content: T ) -> Response {
    return new(status, content.to_json(), HttpContentType::JSON);
}

pub fn text(status: HttpStatus, content: String) -> Response {
    return new(status, content, HttpContentType::TEXT);
}

impl Response {
    pub fn status(&self) -> &HttpStatus {
        &self.status
    }

    fn protocol(&self) -> &str {
        return "HTTP/1.1";
    }

    fn set_header(&mut self, key: &str, value: &str) {
        debug!("Setting Response Header: {} : {}", key, value);
        self.headers.push(format!("{}: {}", key, value));
    }

    pub fn set_encoding(&mut self, encoding: &Option<Encoding>) {
        self.encoding = encoding.clone();
        match encoding {
            Some(e) => self.set_header("Content-Encoding", e.to_str()),
            None => (),
        }
    }

    pub fn build(&mut self) -> Vec<u8> {
        debug!("Encoding set to: {:?}", self.encoding);

        let (content, headers) = {
            let content = match &self.encoding {
                Some(enc) => match enc {
                    Encoding::GZIP => {
                        debug!("Compressing response content with GZIP");
                        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                        encoder.write_all(self.content.as_bytes()).unwrap();
                        encoder.finish().unwrap()
                    }
                    _ => self.content.clone().into_bytes(),
                },
                None => self.content.clone().into_bytes(),
            };

            // Prepare headers
            self.headers
                .push(format!("Content-Type: {}", self.content_type.to_str()));
            self.headers
                .push(format!("Content-Length: {}", content.len()));
            let headers = self.headers.join(CRLF);

            (content, headers)
        };

        let resp = format!(
            "{} {} {}{}{}{}{}",
            self.protocol(),
            self.status.to_code(),
            self.status.to_str(),
            CRLF,
            headers,
            CRLF,
            CRLF
        );

        debug!("Response Raw\n{:?} {}", resp, self.content);

        let mut resp = resp.into_bytes();
        resp.extend(&content);

        return resp;
    }
}
