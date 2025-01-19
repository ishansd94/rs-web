
use std::fmt::{Display, Formatter, Result};
pub enum HttpStatus {
    StatusOK,
    StatusBadRequest,
    StatusNotFound,
    StatusCreated
}

impl Display for HttpStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.to_code())
    }
}

impl Clone for HttpStatus {
    fn clone(&self) -> Self {
        match self {
            HttpStatus::StatusOK => HttpStatus::StatusOK,
            HttpStatus::StatusNotFound => HttpStatus::StatusNotFound,
            HttpStatus::StatusBadRequest => HttpStatus::StatusBadRequest,
            HttpStatus::StatusCreated => HttpStatus::StatusCreated
        }
    }
}


impl HttpStatus {
    pub fn to_code(&self) -> u32 {
        return match self {
            HttpStatus::StatusOK => 200,
            HttpStatus::StatusNotFound => 404,
            HttpStatus::StatusBadRequest => 400,
            HttpStatus::StatusCreated => 201,
        };
    }

    pub fn to_str(&self) -> &str {
        return match self {
            HttpStatus::StatusOK => "OK",
            HttpStatus::StatusNotFound => "Not Found",
            HttpStatus::StatusBadRequest => "Bad Request",
            HttpStatus::StatusCreated => "Created",
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    ALL,
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    PATCH,
}


impl HttpMethod {
    fn to_str(&self) -> &str {
        return match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::ALL => "*",
        };
    }

    pub fn from_str(s: &str) -> Option<HttpMethod> {
        return match s {
            "GET" => Some(HttpMethod::GET),
            "POST" => Some(HttpMethod::POST),
            _ => None,
        };
    }
}

impl Display for HttpMethod {

    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.to_str())
    }
}

pub enum HttpContentType {
    HTML,
    JSON,
    TEXT,
}

impl HttpContentType {
    pub fn to_str(&self) -> &str {
        return match self {
            HttpContentType::HTML => "text/html",
            HttpContentType::JSON => "application/json",
            HttpContentType::TEXT => "application/text"
        };
    }
}

impl Display for HttpContentType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.to_str())
    }
}

pub enum HttpProtocol {
    HTTP1
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Encoding {
    GZIP,
    #[default]
    None
}

impl Encoding {
    pub fn to_str(&self) -> &str {
        return match self {
            Encoding::GZIP => "gzip",
            Encoding::None => "none"
        };
    }

    pub fn get_supported() -> Vec<Encoding> {
        return vec![Encoding::GZIP];
    }

    pub fn from_str(s: &str) -> Option<Encoding> {
        return match s {
            "gzip" => Some(Encoding::GZIP),
            _ => None,
        };
    }
}

pub enum Headers {
    AcceptEncoding,
    ContentType,
    ContentLength
}

impl Headers {
    pub fn to_str(&self) -> &str {
        return match self {
            Headers::AcceptEncoding => "Accept-Encoding",
            Headers::ContentType => "Content-Type",
            Headers::ContentLength => "Content-Length"
        };
    }
}