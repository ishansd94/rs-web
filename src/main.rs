use std::{collections::HashMap, sync::Arc};

use badserde::json::ToJson;
use fastweb;
use fastweb::handler;
use fastweb::http::HttpStatus;
use fastweb::request::Request;
use logger::{self};

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

            let mut  content: HashMap<String, String> = HashMap::new();

            content.insert(String::from("count"), r.path_params().get("count").unwrap().to_string());

            return fastweb::response::json(HttpStatus::StatusOK, content);
        }),
    );

    router.post(
        "/ping",
        handler!(|r: Request| {
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
