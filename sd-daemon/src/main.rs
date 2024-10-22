use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    net::{Shutdown, TcpListener},
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};

use anyhow::Context;
use clap::{ArgAction, Parser};
use log::{error, set_logger, set_max_level, warn, Level, Log};
use once_cell::sync::Lazy;
use sd_lib::{print_record, Message, Mode, ADDRESS};

static mut MESSAGES: Lazy<Arc<Mutex<Vec<(u32, Message)>>>> =
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

fn handle_message(index: u32, message: Message) {
    message.display();
    unsafe {
        MESSAGES.lock().unwrap().push((index, message));
    }
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

            let Mode::Auth(auth) = Mode::recieve(&mut stream).unwrap() else {
                error!("The first transmission of the client wasn't an auth transmission!");
                return;
            };

            let mut clients = OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .truncate(false)
                .open("clients.txt")
                .unwrap();

            let mut index = 0;
            let mut found = false;

            let mut content = String::new();
            clients.read_to_string(&mut content).unwrap();
            for line in content.split('\n') {
                if line.starts_with(&format!("{} ", auth.name)) {
                    found = true;
                    break;
                }
                index += 1;
            }

            if !found {
                clients
                    .write(
                        format!("{} {}\n", auth.name, serde_json::to_string(&auth).unwrap())
                            .as_bytes(),
                    )
                    .unwrap();
            }

            loop {
                let transmission = Mode::recieve(&mut stream).unwrap();

                match transmission {
                    Mode::Message(message) => handle_message(index, message),
                    Mode::Exit(exitcode) => {
                        handle_message(index, Message::new(
                            Level::Info,
                            format!("Client will exit with {exitcode}. Closing connection."),
                        ));
                        stream.shutdown(Shutdown::Both).unwrap();
                        return;
                    }
                    Mode::Auth(_) => warn!("Client sent an auth message. Only the first transmission should be an auth message."),
                }
            }
        });
    }
}

fn save_logs() {
    let mut logfile = File::create("logfile.txt").unwrap();

    loop {
        let mut lock = unsafe { MESSAGES.lock().unwrap() };
        let messages = lock.to_vec();
        lock.clear();
        drop(lock);

        for (index, message) in messages {
            let message = serde_json::to_string(&message).unwrap();
            logfile
                .write(format!("{index} {message}\n").as_bytes())
                .unwrap();
        }
        sleep(Duration::from_secs(10));
    }
}
