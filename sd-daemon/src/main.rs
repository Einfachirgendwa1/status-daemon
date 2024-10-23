use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    net::{Shutdown, TcpListener},
    sync::{
        mpsc::{self, channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use anyhow::Context;
use clap::{ArgAction, Parser};
use log::{error, set_logger, set_max_level, warn, Level, Log};
use sd_lib::{print_record, Message, Mode, ADDRESS};
use serde::Serialize;

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

#[derive(Debug, Clone, Serialize)]
struct RecievedMessage {
    message: Message,
    origin: u32,
}

fn handle_message(
    recieved_message: RecievedMessage,
    save_rx: Arc<Mutex<Sender<RecievedMessage>>>,
    write_rx: Arc<Mutex<Sender<RecievedMessage>>>,
) {
    recieved_message.message.display();
    save_rx
        .lock()
        .unwrap()
        .send(recieved_message.clone())
        .unwrap();
    write_rx.lock().unwrap().send(recieved_message).unwrap();
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

    let (save_rx, save_tx) = channel();
    thread::spawn(|| save(save_tx));
    let save_rx = Arc::new(Mutex::new(save_rx));

    let (write_rx, write_tx) = channel();
    thread::spawn(|| write(write_tx));
    let write_rx = Arc::new(Mutex::new(write_rx));

    for stream in listener.incoming() {
        let save_rx = save_rx.clone();
        let write_rx = write_rx.clone();

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
                let save_rx = save_rx.clone();
                let write_rx = write_rx.clone();
                let transmission = Mode::recieve(&mut stream).unwrap();

                match transmission {
                    Mode::Message(message) => {let recieved_message = RecievedMessage {message, origin:index};handle_message(recieved_message, save_rx, write_rx)}
                    Mode::Exit(exitcode) =>{let recieved_message = RecievedMessage {message: Message::new(
                            Level::Info,
                            format!("Client will exit with {exitcode}. Closing connection."),
                        ), origin:index};
                        handle_message(recieved_message, save_rx, write_rx);
                        stream.shutdown(Shutdown::Both).unwrap();
                        return;
                    }
                    Mode::Auth(_) => warn!("Client sent an auth message. Only the first transmission should be an auth message."),
                }
            }
        });
    }
}

fn save(tx: Receiver<RecievedMessage>) {
    let mut logfile = File::create("logfile.txt").unwrap();

    loop {
        let message = tx.recv().unwrap();
        let msg = format!(
            "{} {}\n",
            message.origin,
            serde_json::to_string(&message).unwrap()
        );

        logfile.write(msg.as_bytes()).unwrap();
    }
}

fn write(tx: mpsc::Receiver<RecievedMessage>) {
    let mut senders = Vec::new();

    let mut content;
    if let Ok(mut senders_txt) = File::open("clients.txt") {
        content = String::new();
        senders_txt.read_to_string(&mut content).unwrap();

        content
            .split('\n')
            .filter(|x| !x.is_empty())
            .filter_map(|x| x.split_once(' '))
            .for_each(|x| senders.push(x));
    }

    loop {
        let msg = tx.recv().unwrap();
        dbg!(&msg);
    }
}
