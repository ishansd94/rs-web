use logger::debug;

use crate::http::{HttpStatus, HttpContentType};
use crate::CRLF;

pub struct Response {
    pub status: HttpStatus,
    pub content: String,
    pub content_type: HttpContentType,
}

pub fn new(status: HttpStatus, content: String, content_type: HttpContentType) -> Response {
    Response {
        status,
        content,            
        content_type,   
    }
}

pub fn html(status: HttpStatus, content: String) -> Response {
    return new(status, content, HttpContentType::HTML);
}

pub fn json(status: HttpStatus, content: String) -> Response {
    return new(status, content, HttpContentType::JSON);
}

pub fn text(status: HttpStatus, content: String) -> Response {
    return new(status, content, HttpContentType::JSON);
}

impl Response {

    pub fn status(&self) -> &HttpStatus {
        &self.status
    }

    fn protocol(&self) -> &str {
        return "HTTP/1.1"
    }

    fn headers() {

    }

    pub fn build(&self) -> String {

        let resp = format!("{} {} {}{}Content-Length: {}{}Content-Type: {}{}{}{}",
                self.protocol(), 
                self.status.to_code(),
                self.status.to_str(),
                CRLF,
                self.content.len(),
                CRLF,
                self.content_type.to_string(),
                CRLF,
                CRLF,
                self.content
        );

        debug!("Response Raw\n{:?}", resp);

        return resp;
    }
}