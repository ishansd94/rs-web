pub mod http;
pub mod router;
pub mod request;
pub mod response;

use router::{RouteTable, RouterBuilder};

static CRLF : &str = "\r\n";
static PATH_SEPARATOR : &str = "/";
static DOUBLE_PATH_SEPARATOR : &str = "//";
static QUERY_PARAM_KEY_VALUE_SEPARATOR : &str = "=";
static QUERY_PARAM_SEPARATOR : &str = "&";
static QUERY_PARAM_START : &str = "?";
static EMPTY: &str = "";
static LEFT_BRACKET: &str = "{";
static RIGHT_BRACKET: &str = "}";

// fn handle_connection(mut stream: TcpStream) {
//     let mut buffer = [0; 1024];
//     stream.read(&mut buffer).unwrap();
//     println!("Received: {}", String::from_utf8_lossy(&buffer[..]));
// }

pub struct TLS {
    key_file: String,
    cert_file: String,
    ca_file: String,
}

pub struct Logging {
    level: String,
}

pub struct Configuration {
    port: u16,
    host: String,
    buffer_size: usize,
    logging: Logging,
    tls: Option<TLS>,
    workers: usize,
}

pub fn new() -> RouterBuilder {
    RouterBuilder {
        configuration: Configuration {
            port: 8080,
            host: "0.0.0.0".to_string(),
            buffer_size: 1024,
            logging: Logging {
                level: "info".to_string(),
            },
            tls: None,
            workers: 2,
        },
        routes: RouteTable(Vec::new()),
    }
}

#[macro_export]
macro_rules! handler {
    ($closure:expr) => {
        Arc::new($closure)
    };
}