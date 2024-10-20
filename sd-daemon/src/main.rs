use std::{net::TcpListener, thread};

use anyhow::Context;
use sd_lib::ADDRESS;

fn main() {
    let listener = TcpListener::bind(ADDRESS)
        .context(format!("Failed to bind to address {ADDRESS}."))
        .unwrap();

    for stream in listener.incoming() {
        thread::spawn(move || {
            let stream = stream.context("Connection failed!").unwrap();
        });
    }
}
