use std::{
    io::Read,
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
};

use anyhow::Context;
use once_cell::sync::Lazy;
use sd_lib::{Message, ADDRESS};

static mut MESSAGES: Lazy<Arc<Mutex<Vec<Message>>>> =
    Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

fn main() {
    let listener = TcpListener::bind(ADDRESS)
        .context(format!("Failed to bind to address {ADDRESS}."))
        .unwrap();

    for stream in listener.incoming() {
        thread::spawn(move || {
            let mut stream = stream.context("Connection failed!").unwrap();

            let mut buf = Vec::new();
            loop {
                stream
                    .read_to_end(&mut buf)
                    .context("Failed to read from stream.")
                    .unwrap();

                let message = Message::from_sendeable(&buf)
                    .context("Invalid message read from stream.")
                    .unwrap();

                unsafe { MESSAGES.lock().unwrap().push(message) }

                println!("")
            }
        });
    }
}
