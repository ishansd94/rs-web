use std::sync::Arc;

use fastweb;
use fastweb::http::HttpStatus;
use fastweb::request::Request;
use fastweb::handler;
use logger::{self, info};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::set_level(logger::Level::Debug);

    let mut router = fastweb::new();

    router.get(
        "/ping",
        handler!(|r: Request| {
            let content = String::from("pong");
            // println!("{:?}", r);
            return fastweb::response::text(HttpStatus::StatusOK, content);
        }),
    );

    router.get(
        "/ping/{count}",
        handler!(|r: Request| {
            // println!("raw: {:?}", r.raw());
            // println!("headers: {:?}", r.headers());
            // println!("path params: {:?}", r.path_params());
            // println!("query params: {:?}", r.query_params());
            // println!("body: {:?}", r.body());

            let count = r.path_params().get("count").unwrap();
            let content = format!("{{\"count\": \"{}\"}}", count);
            return fastweb::response::json(HttpStatus::StatusOK, content);
        }),
    );

    router.post(
        "/ping",
        handler!(|r: Request| {
            let params = r.query_params();
            let content = String::from("pong");
            return fastweb::response::text(HttpStatus::StatusOK, content);
        }),
    );

    router
        .host(String::from("0.0.0.0"))
        .port(8080)
        .buffer_size(1024)
        .workers(5)
        .serve()?;

    Ok(())
}


