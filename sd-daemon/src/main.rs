use std::{
    net::{Shutdown, TcpListener},
    // sync::{Arc, Mutex},
    thread,
};

use anyhow::Context;
// use once_cell::sync::Lazy;
use sd_lib::{/* Message, */ Mode, Transmission, ADDRESS};

// static mut MESSAGES: Lazy<Arc<Mutex<Vec<Message>>>> =
//     Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

fn main() {
    let listener = TcpListener::bind(ADDRESS)
        .context(format!("Failed to bind to address {ADDRESS}."))
        .unwrap();

    for stream in listener.incoming() {
        thread::spawn(move || {
            let mut stream = stream.context("Connection failed!").unwrap();

            loop {
                let transmission = Transmission::recieve(&mut stream).unwrap();

                match transmission {
                    Mode::Message(message) => println!("{message}"),
                    Mode::Exit(exitcode) => {
                        println!("Client will exit with code {exitcode}. Closing connection.");
                        stream.shutdown(Shutdown::Both).unwrap();
                        return;
                    }
                }
            }
        });
    }
}
