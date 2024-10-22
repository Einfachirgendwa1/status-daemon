use std::{
    fs::File,
    io::Write,
    net::{Shutdown, TcpListener},
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};

use anyhow::Context;
use clap::{ArgAction, Parser};
use log::{set_logger, set_max_level, Level, Log};
use once_cell::sync::Lazy;
use sd_lib::{print_record, Message, Mode, Transmission, ADDRESS};

static mut MESSAGES: Lazy<Arc<Mutex<Vec<Message>>>> =
    Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

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

#[derive(clap::Parser)]
struct Cli {
    #[arg(long, short, global = true, action = ArgAction::SetTrue)]
    quiet: Option<bool>,
}

fn main() {
    let Cli { quiet } = Cli::parse();

    if !quiet.unwrap_or(false) {
        set_logger(&Logger {}).unwrap();
        set_max_level(log::LevelFilter::Trace);
    }

    let listener = TcpListener::bind(ADDRESS)
        .context(format!("Failed to bind to address {ADDRESS}."))
        .unwrap();

    thread::spawn(save_logs);

    for stream in listener.incoming() {
        thread::spawn(move || {
            let mut stream = stream.context("Connection failed!").unwrap();

            loop {
                let transmission = Transmission::recieve(&mut stream).unwrap();

                match transmission {
                    Mode::Message(message) => {
                        handle_message(message);
                    }
                    Mode::Exit(exitcode) => {
                        handle_message(Message::new(
                            Level::Info,
                            format!("Client will exit with code {exitcode}. Closing connection."),
                        ));
                        stream.shutdown(Shutdown::Both).unwrap();
                        return;
                    }
                }
            }
        });
    }
}

fn handle_message(message: Message) {
    message.display();
    unsafe {
        MESSAGES.lock().unwrap().push(message);
    }
}

fn save_logs() {
    let mut testfile = File::create("testfile").unwrap();

    loop {
        let mut lock = unsafe { MESSAGES.lock().unwrap() };
        let messages = lock.to_vec();
        lock.clear();
        drop(lock);

        for message in messages {
            testfile.write(&message.make_sendeable()).unwrap();
        }
        sleep(Duration::from_secs(10));
    }
}
