use std::{io::Read, net::TcpListener, thread};

use anyhow::Context;
use sd_lib::ADDRESS;

fn main() {
    let listener = TcpListener::bind(ADDRESS)
        .context(format!("Failed to bind to address {ADDRESS}."))
        .unwrap();

    for stream in listener.incoming() {
        thread::spawn(move || {
            let mut stream = stream.context("Connection failed!").unwrap();

            let mut buf = String::new();
            loop {
                stream
                    .read_to_string(&mut buf)
                    .context("Failed to read from Stream.")
                    .unwrap();
            }
        });
    }
}
