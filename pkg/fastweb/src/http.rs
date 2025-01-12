
use std::fmt::{Display, Formatter, Result};
pub enum HttpStatus {
    StatusOK,
    StatusBadRequest,
    StatusNotFound,
    StatusCreated
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

    pub fn get_message(&self) -> &str {
        return match self {
            HttpStatus::StatusOK => "OK",
            HttpStatus::StatusNotFound => "Not Found",
            HttpStatus::StatusBadRequest => "Bad Request",
            HttpStatus::StatusCreated => "Created",
        };
    }
}

#[derive(Debug, Clone)]
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
}

impl HttpContentType {
    fn to_str(&self) -> &str {
        return match self {
            HttpContentType::HTML => "text/html",
            HttpContentType::JSON => "application/json",
        };
    }
}

impl Display for HttpContentType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.to_str())
    }
}