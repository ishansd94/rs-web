use std::{
    collections::HashMap,
    error::Error,
    fmt::{Debug, Display},
    fs,
    io::prelude::*,
    net::{TcpListener, TcpStream},
    sync::Arc,
};

use logger::{debug, error, info};
use workers::ThreadPool;

use crate::{
    http::{HttpContentType, HttpMethod, HttpStatus},
    request,
    response::{self, Response},
    Configuration
};
use crate::{DOUBLE_PATH_SEPARATOR, EMPTY, LEFT_BRACKET, PATH_SEPARATOR, RIGHT_BRACKET};

type HandlerFunc = Arc<dyn Fn(request::Request) -> response::Response + Send + Sync + 'static>;

pub struct RouteTable(pub Vec<(String, HttpMethod, Route)>);

impl Display for RouteTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.iter().for_each(|(path, method, route)| {
            write!(f, "{} {} {:?}\n", path, method, route).unwrap();
        });
        Ok(())
    }
}

impl Clone for RouteTable {
    fn clone(&self) -> Self {
        RouteTable(self.0.clone())
    }
}

impl RouteTable {
    pub fn find(&self, qualified_path: &str, http_method: &HttpMethod) -> Option<&Route> {
        debug!(
            "Matching for Qualified path {}, HTTP method {}",
            qualified_path, http_method
        );

        let req_segments = qualified_path.split('/').collect::<Vec<&str>>();

        let matches = self
            .0
            .iter()
            .filter(|(_, route_method, route)| {
                if route_method != http_method {
                    return false;
                }

                if req_segments.len() != route.segments.len() {
                    return false;
                }

                for (index, segment) in route.segments.iter().enumerate() {
                    if segment.starts_with(LEFT_BRACKET) && segment.ends_with(RIGHT_BRACKET) {
                        continue;
                    }

                    if segment != &req_segments[index] {
                        return false;
                    }
                }

                return true;
            })
            .map(|(_, _, route)| route)
            .collect::<Vec<&Route>>();

        if !matches.is_empty() {
            return Some(matches[0]);
        }

        return None;
    }

    pub fn insert(&mut self, route: Route) {
        self.0
            .push((route.base_path.clone(), route.method.clone(), route));
    }
}

pub struct Route {
    method: HttpMethod,
    path: String,
    base_path: String,
    handler: HandlerFunc,
    path_params: Option<Vec<String>>,
    tokens: usize,
    segments: Vec<String>,
}

impl Clone for Route {
    fn clone(&self) -> Self {
        Route {
            method: self.method.clone(),
            path: self.path.clone(),
            base_path: self.base_path.clone(),
            handler: Arc::clone(&self.handler),
            path_params: self.path_params.clone(),
            tokens: self.tokens,
            segments: self.segments.clone(),
        }
    }
}

impl Debug for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.method, self.path)
    }
}

impl Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.method, self.path)
    }
}

pub struct RouterBuilder {
    pub configuration: Configuration,
    pub routes: RouteTable,
}

impl RouterBuilder {
    pub fn port(&mut self, port: u16) -> &mut Self {
        self.configuration.port = port;
        self
    }

    pub fn host(&mut self, host: String) -> &mut Self {
        self.configuration.host = host;
        self
    }

    pub fn buffer_size(&mut self, buffer_size: usize) -> &mut Self {
        self.configuration.buffer_size = buffer_size;
        self
    }

    pub fn logging_level(&mut self, level: String) -> &mut Self {
        self.configuration.logging.level = level;
        self
    }

    pub fn workers(&mut self, workers: usize) -> &mut Self {
        self.configuration.workers = workers;
        self
    }

    fn get_bind_address(&self) -> String {
        format!("{}:{}", self.configuration.host, self.configuration.port)
    }

    pub fn serve(&self) -> Result<(), Box<dyn Error>> {
        info!(
            "Starting server on {}:{}",
            self.configuration.host, self.configuration.port
        );

        let listener = TcpListener::bind(self.get_bind_address())?;

        let pool = ThreadPool::new(self.configuration.workers);

        let routes = Arc::new(self.get_routes());

        info!("Registering routes..");
        info!("\n{}", routes);

        for stream in listener.incoming() {
            let routes = Arc::clone(&routes);
            let buffer_size = self.configuration.buffer_size;

            match stream {
                Ok(stream) => {
                    pool.execute(move || {
                        if let Err(e) = handle(stream, &routes, &buffer_size) {
                            error!("Error handling request {}", e)
                        }
                    });
                }
                Err(e) => {
                    return Err(Box::new(e));
                }
            }
        }

        return Ok(());
    }

    pub fn add_route(&mut self, path: &str, method: HttpMethod, handler: HandlerFunc) -> &Self {
        let sanitized_path = path.replace(DOUBLE_PATH_SEPARATOR, PATH_SEPARATOR);

        let tokens: Vec<String> = sanitized_path
            .split(PATH_SEPARATOR)
            .map(|s| s.to_string())
            .collect();

        let mut path_params = vec![];

        for token in &tokens {
            if token.starts_with(LEFT_BRACKET) && token.ends_with(RIGHT_BRACKET) {
                let param = token
                    .replace(LEFT_BRACKET, EMPTY)
                    .replace(RIGHT_BRACKET, EMPTY)
                    .to_lowercase();
                path_params.push(param);
            }
        }

        let base_path = sanitized_path
            .split(PATH_SEPARATOR)
            .filter(|token| !token.starts_with(LEFT_BRACKET) && !token.ends_with(RIGHT_BRACKET))
            .collect::<Vec<&str>>()
            .join(PATH_SEPARATOR);

        self.routes.insert(Route {
            method: method,
            path: sanitized_path.clone(),
            base_path: base_path,
            handler: handler,
            path_params: match path_params.len() {
                0 => None,
                _ => Some(path_params),
            },
            tokens: tokens.len(),
            segments: tokens,
        });

        return self;
    }

    pub fn get(&mut self, path: &str, handler: HandlerFunc) -> &Self {
        self.add_route(path, HttpMethod::GET, handler);
        return self;
    }

    pub fn post(&mut self, path: &str, handler: HandlerFunc) -> &Self {
        self.add_route(path, HttpMethod::POST, handler);
        return self;
    }

    pub fn get_routes(&self) -> RouteTable {
        return self.routes.clone();
    }
}

fn handle(
    mut stream: TcpStream,
    routes: &RouteTable,
    buffer_size: &usize,
) -> Result<(), Box<dyn Error>> {
    debug!("Accepted connection from: {}", stream.peer_addr().unwrap());

    let mut buffer = vec![0; *buffer_size];
    let bytes_read = stream.read(&mut buffer)?;
    let raw_request = std::str::from_utf8(&buffer[..bytes_read])?;

    let mut request = request::parse(raw_request);

    debug!("Parsed request\n{:?}", request);

    let method = request.method().to_string();
    let path = request.path().to_string();
    let qualified_path = request.qualified_path().to_string();

    let route = routes.find(&qualified_path, request.method());
    let enc = request.encoding();

    debug!("Route matched\n{:?}", route);

    let mut response = match route {
        Some(route) => {
            let path_params_list: Vec<&str> = if qualified_path == route.base_path {
                vec![]
            } else {
                qualified_path
                    .strip_prefix(&route.base_path)
                    .unwrap_or("")
                    .split('/')
                    .filter(|s| !s.is_empty())
                    .collect()
            };

            let mut path_params = HashMap::new();

            if let Some(route_params) = &route.path_params {
                for (index, param) in path_params_list.iter().enumerate() {
                    if let Some(param_name) = route_params.get(index) {
                        path_params.insert(param_name.clone(), param.to_string());
                    }
                }
            }

            request.set_path_params(path_params);

            (route.handler)(request)
        }

        //Path matched but no matches for tokens or method
        None => not_found(),
    };

    response.set_encoding(&enc);

    stream.write(&response.build())?;
    info!("{} {} {}", method, path, response.status());
    stream.flush()?;

    Ok(())
}

fn error_html() -> String {
    return fs::read_to_string("public/404.html").unwrap();
}

fn not_found() -> Response {
    let content = error_html();
    response::new(HttpStatus::StatusNotFound, content, HttpContentType::HTML)
}
