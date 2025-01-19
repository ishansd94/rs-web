use std::{collections::HashMap, fmt::Debug};

use crate::http::HttpMethod;

use super::{CRLF, QUERY_PARAM_KEY_VALUE_SEPARATOR, QUERY_PARAM_SEPARATOR, QUERY_PARAM_START, EMPTY};


#[derive(Debug)]
pub struct Request {
    method: HttpMethod,
    path: String,
    qualified_path: String,
    query_params: HashMap<String, String>,
    path_params: HashMap<String, String>,
    headers: HashMap<String, String>,
    body: String,
    raw: String,
}

impl Request {
    pub fn method(&self) -> &HttpMethod {
        &self.method
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn qualified_path(&self) -> &str {
        &self.qualified_path
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn query_params(&self) -> &HashMap<String, String> {
        &self.query_params
    }

    pub fn path_params(&self) -> &HashMap<String, String> {
        &self.path_params
    }

    pub fn set_path_params(&mut self, path_params: HashMap<String, String>) {
        self.path_params = path_params;
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }
    
}

pub fn parse(request_raw: &str) -> Request {
    let mut lines = request_raw.split(CRLF);

    // Parse the request line
    let request_meta = lines.next().unwrap();
    let mut request_meta_parts = request_meta.split_whitespace();
    let method = request_meta_parts.next().unwrap();
    let path = request_meta_parts.next().unwrap();
    let _ = request_meta_parts.next(); // Ignore the HTTP version

    // Parse the headers
    let headers = lines
        .by_ref()
        .take_while(|line| !line.is_empty())
        .map(|line| {
            let mut parts = line.splitn(2, ": ");
            let key = parts.next().unwrap().to_string();
            let value = parts.next().unwrap_or(EMPTY).to_string();
            (key, value)
        })
        .collect();

    let mut path_parts = path.split(QUERY_PARAM_START);
    let qualified_path = path_parts.nth(0).unwrap();
    let query_params_str = path_parts.nth(0).unwrap_or_default();
    
    let mut query_params = HashMap::new();

    if !query_params_str.is_empty() {
        query_params = query_params_str
                        .split(QUERY_PARAM_SEPARATOR)
                        .map(|param| {
                            let mut param_parts = param.split(QUERY_PARAM_KEY_VALUE_SEPARATOR);
                            let key = param_parts.next().unwrap().to_string();
                            let value = param_parts.next().unwrap_or(EMPTY).to_string();
                            (key, value)
                        })
                        .collect();
    }

    // Parse the body
    let body = lines.collect::<Vec<&str>>().join(CRLF).trim_matches(char::from(0)).to_string();

    // Construct and return the Request
    return Request {
        method: HttpMethod::from_str(method).unwrap(),
        path: path.to_string(),
        qualified_path: qualified_path.to_string(),
        headers,
        body,
        query_params,
        path_params: HashMap::new(),
        raw: request_raw.to_string(),
    }
}