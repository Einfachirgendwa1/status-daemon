use std::{
    error::Error,
    fmt::Display,
    net::TcpStream,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use log::{set_logger, set_max_level, LevelFilter, Log};
use sd_lib::{print_record, Auth, Message, RandomProgramToDaemon, Transmission, ADDRESS};

static mut STREAM: Option<Arc<Mutex<TcpStream>>> = None;

pub fn init() {
    try_connect();
    send_auth(Auth::default());
}

pub fn report(record: &log::Record) {
    if let Some(stream) = unsafe { STREAM.as_ref() } {
        Message::new(record.level(), record.args().to_string())
            .send(&mut stream.lock().unwrap())
            .unwrap()
    } else {
        todo!()
    }
}

pub struct RecommendedLogger {}

impl Log for RecommendedLogger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        report(&record);
        if self.enabled(record.metadata()) {
            print_record(record);
        }
    }

    fn flush(&self) {}
}

/// Like [`log::SetLoggerError`] but it actually impls [`Error`]
#[derive(Debug)]
struct SetLoggerError {}

impl Display for SetLoggerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to set logger.")
    }
}

impl Error for SetLoggerError {}

pub fn use_recommended_logger() -> Result<()> {
    if set_logger(&RecommendedLogger {}).is_err() {
        Err(SetLoggerError {})?;
    }
    set_max_level(LevelFilter::Trace);

    Ok(())
}

pub fn close_connection(exitcode: u8) {
    if let Some(stream) = unsafe { STREAM.as_ref() } {
        RandomProgramToDaemon::Exit(exitcode)
            .transmit(&mut stream.lock().unwrap())
            .unwrap()
    }
}

fn try_connect() {
    if let Ok(stream) = TcpStream::connect(ADDRESS) {
        unsafe { STREAM = Some(Arc::new(Mutex::new(stream))) }
    } else {
        start_daemon();
        if let Ok(stream) = TcpStream::connect(ADDRESS) {
            unsafe { STREAM = Some(Arc::new(Mutex::new(stream))) }
        } else {
            todo!("Failed to connect even after starting daemon.");
        }
    }
}

fn send_auth(auth: Auth) {
    if let Some(stream) = unsafe { STREAM.as_ref() } {
        RandomProgramToDaemon::Auth(auth)
            .transmit(&mut stream.lock().unwrap())
            .unwrap()
    }
}

fn start_daemon() {
    todo!("Daemon is not running")
}
