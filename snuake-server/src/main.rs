extern crate futures;
extern crate snuake_shared;
extern crate tokio;

mod core;
mod websocket;

use std::env;

use futures::prelude::*;
use tokio::net::TcpListener;

fn main() {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr = addr.parse().unwrap();

    let (core_s, c) = core::core();

    let socket = TcpListener::bind(&addr).unwrap();
    println!("Listening on: {}", addr);

    let srv = socket
        .incoming()
        .map_err(|_| ())
        .for_each(move |tcp_sr| {
            websocket::new(tcp_sr, core_s.clone())
        })
        .map_err(|_| ());

    let all = srv.select(c)
        .then(|_| Ok(()) );
    tokio::run(all);
}
