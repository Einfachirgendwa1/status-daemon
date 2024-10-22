use std::{
    net::{Shutdown, TcpListener},
    // sync::{Arc, Mutex},
    thread,
};

use anyhow::Context;
use log::{set_logger, set_max_level, Log};
// use once_cell::sync::Lazy;
use sd_lib::{/* Message, */ print_record, Mode, Transmission, ADDRESS};

// static mut MESSAGES: Lazy<Arc<Mutex<Vec<Message>>>> =
//     Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

struct Logger {}

impl Log for Logger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            print_record(record);
        }
    }

    fn flush(&self) {}
}

fn main() {
    set_logger(&Logger {}).unwrap();
    set_max_level(log::LevelFilter::Trace);

    let listener = TcpListener::bind(ADDRESS)
        .context(format!("Failed to bind to address {ADDRESS}."))
        .unwrap();

    for stream in listener.incoming() {
        thread::spawn(move || {
            let mut stream = stream.context("Connection failed!").unwrap();

            loop {
                let transmission = Transmission::recieve(&mut stream).unwrap();

                match transmission {
                    Mode::Message(message) => message.display(),
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
