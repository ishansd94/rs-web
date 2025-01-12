use std::collections::HashMap;
use std::io::prelude::*;
use std::{
    error::Error,
    fmt::Debug,
    fmt::Display,
    fs,
    net::{TcpListener, TcpStream},
    sync::Arc,
};

use logger::info;
use workers::ThreadPool;

use crate::{
    http::{HttpContentType, HttpMethod, HttpStatus},
    request, response, Configuration, TLS,
};

type HandlerFunc = Arc<dyn Fn(request::Request) -> response::Response + Send + Sync + 'static>;


pub struct RouteTable(pub Vec<(String, Route)>);

impl Display for RouteTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.iter().for_each(|(_, route)| {
            info!("{} {}", route.method, route.abs_path);
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
    pub fn get_matches(&self, path: &str) -> Option<Vec<&Route>> {
        let routes =  self.0.iter().filter(|r| {
                path.starts_with(&r.0)
        })
        .map(|(_, route)|  route)
        .collect::<Vec<&Route>>();
        
        match routes.len() {
            0 => None,
            _ => Some(routes)
        }
    }

    pub fn insert(&mut self, route: Route) {
        self.0.push((route.path.clone(), route));
    }
}


pub struct Route {
    method: HttpMethod,
    abs_path: String,
    path: String,
    handler: HandlerFunc,
    path_params: Option<Vec<String>>,
    tokens: usize,
}

impl Clone for Route {
    fn clone(&self) -> Self {
        Route {
            method: self.method.clone(),
            abs_path: self.abs_path.clone(),
            path: self.path.clone(),
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

        info!("Starting server on {}:{}", self.configuration.host, self.configuration.port);

        let listener = TcpListener::bind(self.get_bind_address())?;

        let pool = ThreadPool::new(self.configuration.workers);

        let routes = Arc::new(self.get_routes());


        info!("Registering routes..");
        info!( "{}", routes);

        for stream in listener.incoming() {

            let routes = Arc::clone(&routes);
            let buffer_size = self.configuration.buffer_size;

            match stream {
                Ok(stream) => {
                    let routes = Arc::clone(&routes);
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
        let normalized_path = path.replace("//", "/");

        let tokens = normalized_path.split("/").collect::<Vec<&str>>();

        let mut path_params = vec![];

        for token in tokens.clone()  {
            if token.starts_with("{") && token.ends_with("}") {
                let param = token
                                     .replace("{", "")
                                     .replace("}", "")
                                     .to_lowercase();
                path_params.push(param);
            }
        }

        let path = normalized_path                         
                            .split("/")
                            .filter(|token| !token.starts_with("{") && !token.ends_with("}"))
                            .collect::<Vec<&str>>()
                            .join("/");
        
        self.routes.insert(
            Route {
                method: method,
                abs_path: normalized_path.clone(),
                path: path.clone(),
                handler: handler,
                path_params: match path_params.len() {
                    0 => None,
                    _ => Some(path_params)                    
                },
                tokens: tokens.len(),
            },
        );

        return self;
    }

    pub fn get(&mut self, path: &str, handler: HandlerFunc) -> &Self {
        self.add_route(path, HttpMethod::GET, handler);
        return self;
    }

    pub fn get_routes(&self) -> RouteTable {
        return self.routes.clone();
    }
}

fn handle(mut stream: TcpStream, routes: &RouteTable, buffer_size: &usize) {
    
    let mut buffer = vec![0; *buffer_size];
    stream.read(&mut buffer).unwrap();

    let mut request = request::parse(std::str::from_utf8(&buffer).unwrap());

    let routes = routes.get_matches(request.path());

    
    let tokens = request.path().split("/").collect::<Vec<&str>>();
    let token_count = tokens.len();

    match routes {
        //Exact match
        Some(routes) => {

            let route = routes
                                          .iter()
                                          .filter(|r| { r.tokens == token_count && r.method.to_string() == request.method().to_string() })
                                          .next();

            match route {
                Some(route) => {

                    let path_params_list: Vec<&str> = request.path()
                                                           .strip_prefix(&route.path)
                                                           .unwrap_or("")
                                                           .split("?").collect::<Vec<&str>>()[0]
                                                           .split('/')
                                                           .filter(|s| !s.is_empty())
                                                           .collect();

                    let mut path_params = HashMap::new();

                    for (index, param) in path_params_list.iter().enumerate() {
                        path_params.insert(route.path_params.as_ref().unwrap()[index].clone(), param.to_string());
                    }

                    request.set_path_params(path_params);

                    let response = (route.handler)(request);

                    stream.write(response.build().as_bytes()).unwrap();  
                },
                None => {
                    let content = error_html();
                    let response = response::new(HttpStatus::StatusNotFound, content, HttpContentType::HTML);
                    stream.write(response.build().as_bytes()).unwrap();
                }
                
            }
        },

        //No Matches
        None => {
            let content = error_html();
            let response = response::new(HttpStatus::StatusNotFound, content, HttpContentType::HTML);
            stream.write(response.build().as_bytes()).unwrap();
        }    
    }

    stream.flush().unwrap();
}



fn error_html() -> String {
    return fs::read_to_string("public/404.html").unwrap();
}
