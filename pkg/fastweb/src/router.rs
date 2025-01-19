use std::collections::HashMap;
use std::io::prelude::*;
use std::process::exit;
use std::{
    error::Error,
    fmt::Debug,
    fmt::Display,
    fs,
    net::{TcpListener, TcpStream},
    sync::Arc,
};

use logger::{debug, info};
use workers::ThreadPool;

use crate::{
    http::{HttpContentType, HttpMethod, HttpStatus},
    request, response, Configuration, TLS,
};
use crate::{DOUBLE_PATH_SEPARATOR, EMPTY, LEFT_BRACKET, PATH_SEPARATOR, RIGHT_BRACKET};

type HandlerFunc = Arc<dyn Fn(request::Request) -> response::Response + Send + Sync + 'static>;

pub struct RouteTable(pub Vec<(String, Route)>);

impl Display for RouteTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.iter().for_each(|(_, route)| {
            write!(f, "{} {}\n", route.method, route.path).unwrap();
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
    pub fn get_matches(&self, qualified_path: &str) -> Option<Vec<&Route>> {

        debug!("RouteTable.get_matches: qualified_path - {}", qualified_path);

        let routes = self
            .0
            .iter()
            .filter(|r| qualified_path.starts_with(&r.0))
            .map(|(_, route)| route)
            .collect::<Vec<&Route>>();

        return (!routes.is_empty()).then_some(routes);
    }

    pub fn insert(&mut self, route: Route) {
        self.0.push((route.base_path.to_string(), route));
    }
}

pub struct Route {
    method: HttpMethod,
    path: String,
    base_path: String,
    handler: HandlerFunc,
    path_params: Option<Vec<String>>,
    tokens: usize,
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

    pub fn tls(&mut self, key_file: String, cert_file: String, ca_file: String) -> &mut Self {
        self.configuration.tls = Some(TLS {
            key_file,
            cert_file,
            ca_file,
        });
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
                        handle(stream, &routes, &buffer_size);
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

        let tokens = sanitized_path.split(PATH_SEPARATOR).collect::<Vec<&str>>();

        let mut path_params = vec![];

        for token in tokens.clone() {
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

fn handle(mut stream: TcpStream, routes: &RouteTable, buffer_size: &usize) {
    let mut buffer = vec![0; *buffer_size];
    let bytes_read = stream.read(&mut buffer).unwrap();
    let request_str = std::str::from_utf8(&buffer[..bytes_read]).unwrap();

    let mut request = request::parse(request_str);

    debug!("Parsed request\n{:?}", request);

    let method = request.method().to_string();
    let path = request.path().to_string();

    let routes = routes.get_matches(request.qualified_path());

    debug!("Routes matched\n{:?}", routes);

    let tokens = request.path().split(PATH_SEPARATOR).collect::<Vec<&str>>();
    let token_count = tokens.len();

    match routes {
        //Exact match
        Some(routes) => {
            let route = routes
                .iter()
                .filter(|r| {
                    r.tokens == token_count && r.method.to_string() == request.method().to_string()
                })
                .next();

            match route {
                Some(route) => {
                    let path_params_list: Vec<&str> = if request.qualified_path() == route.base_path {
                        vec![]
                    } else {
                        request
                            .qualified_path()
                            .strip_prefix(route.base_path.as_str())
                            .unwrap()
                            .split('/')
                            .filter(|s| !s.is_empty())
                            .collect()
                    };

                    let mut path_params = HashMap::new();

                    for (index, param) in path_params_list.iter().enumerate() {
                        path_params.insert(
                            route.path_params.as_ref().unwrap()[index].clone(),
                            param.to_string(),
                        );
                    }

                    request.set_path_params(path_params);

                    let response = (route.handler)(request);

                    let status = response.status();

                    debug!("{} {} {}", method, path, status);

                    stream.write(response.build().as_bytes()).unwrap();
                }
                None => {
                    let content = error_html();
                    let response =
                        response::new(HttpStatus::StatusNotFound, content, HttpContentType::HTML);
                    stream.write(response.build().as_bytes()).unwrap();
                }
            }
        }

        //No Matches
        None => {
            let content = error_html();
            let response =
                response::new(HttpStatus::StatusNotFound, content, HttpContentType::HTML);
            stream.write(response.build().as_bytes()).unwrap();
        }
    }

    stream.flush().unwrap();
}

fn error_html() -> String {
    return fs::read_to_string("public/404.html").unwrap();
}
