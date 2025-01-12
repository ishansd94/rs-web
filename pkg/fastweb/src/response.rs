use crate::http::{HttpStatus, HttpContentType};


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

impl Response {

    pub fn status(&self) -> &HttpStatus {
        &self.status
    }

    pub fn build(&self) -> String {

        match self.content_type {
            HttpContentType::HTML => {
                return format!(
                    "HTTP/1.1 {} {}\r\nContent-Type: {}\r\n\r\n{}",
                    self.status.to_code(),
                    self.status.get_message(),
                    self.content_type,
                    self.content
                );
            }
            HttpContentType::JSON => {
                return format!(
                    "HTTP/1.1 {} {}\r\nContent-Type: {}\r\n\r\n{}",
                    self.status.to_code(),
                    self.status.get_message(),
                    self.content_type,
                    self.content
                );
            }
            
        }
    }
}