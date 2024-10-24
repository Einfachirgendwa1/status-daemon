use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use anyhow::Context;
use clap::{ArgAction, Parser};
use log::{error, set_logger, set_max_level, warn, Level, Log};
use sd_lib::{
    print_record, DaemonToClient, Message, RandomProgramToDaemon, RecievedMessage, Transmission,
    ADDRESS,
};

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

fn handle_message(
    recieved_message: RecievedMessage,
    save_rx: Arc<Mutex<Sender<RecievedMessage>>>,
    write_rx: Arc<Mutex<Sender<WriteTransmission>>>,
) {
    recieved_message.message.display();
    save_rx
        .lock()
        .unwrap()
        .send(recieved_message.clone())
        .unwrap();
    write_rx
        .lock()
        .unwrap()
        .send(WriteTransmission::RecievedMessage(recieved_message))
        .unwrap();
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

            let auth = match RandomProgramToDaemon::recieve(&mut stream).unwrap() {
                RandomProgramToDaemon::Auth(auth) => auth,
                RandomProgramToDaemon::NewClient => {
                    write_rx
                        .lock()
                        .unwrap()
                        .send(WriteTransmission::NewClient(stream))
                        .unwrap();
                    return;
                }
                _ => {
                    error!("The first transmission of the client wasn't an Auth or NewClient transmission!");
                    return;
                }
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
                let transmission = RandomProgramToDaemon::recieve(&mut stream).unwrap();

                match transmission {
                    RandomProgramToDaemon::Message(message) => {
                        let recieved_message = RecievedMessage {
                            message,
                            origin: index,
                        };
                        handle_message(recieved_message, save_rx, write_rx);
                    }
                    RandomProgramToDaemon::Exit(exitcode) => {
                        let recieved_message = RecievedMessage {
                            message: Message::new(
                                Level::Info,
                                format!("Client will exit with {exitcode}. Closing connection."),
                            ),
                            origin: index,
                        };
                        handle_message(recieved_message, save_rx, write_rx);
                        stream.shutdown(Shutdown::Both).unwrap();
                        return;
                    }
                    RandomProgramToDaemon::Auth(_) => {
                        // https://github.com/rust-lang/rustfmt/issues/3206
                        warn!(
                            "{}{}",
                            "Client sent an Auth message.",
                            "Only the first transmission should be an Auth message."
                        );
                    }
                    RandomProgramToDaemon::NewClient => {
                        warn!(
                            "{}{}",
                            "Client sent a NewClient message.",
                            "Only the first transmission should be a NewClient message."
                        );
                    }
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

#[derive(Debug)]
enum WriteTransmission {
    RecievedMessage(RecievedMessage),
    NewClient(TcpStream),
}

fn write(tx: Receiver<WriteTransmission>) {
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

    let mut clients = Vec::new();

    loop {
        let transmission = tx.recv().unwrap();
        match transmission {
            WriteTransmission::NewClient(client) => clients.push(client),
            WriteTransmission::RecievedMessage(recieved_message) => {
                for mut client in &mut clients {
                    DaemonToClient::RecievedMessage(recieved_message.clone())
                        .transmit(&mut client)
                        .unwrap();
                }
            }
        }
    }
}
