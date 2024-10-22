use std::{
    error::Error,
    fmt::Display,
    net::TcpStream,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use colored::Colorize;
use log::{set_logger, set_max_level, Level, LevelFilter, Log};
use sd_lib::{Message, Mode, Transmission, ADDRESS};

static mut STREAM: Option<Arc<Mutex<TcpStream>>> = None;

pub fn init() {
    try_connect();
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

pub fn send_test_message(message: &str) {
    if let Some(stream) = unsafe { STREAM.as_ref() } {
        Message::new(Level::Info, message.to_string())
            .send(&mut stream.lock().unwrap())
            .unwrap()
    } else {
        todo!()
    }
}

pub fn close_connection(exit_code: u8) {
    if let Some(stream) = unsafe { STREAM.as_ref() } {
        Transmission::new(Mode::Exit(exit_code))
            .transmit(&mut stream.lock().unwrap())
            .unwrap();
    } else {
        todo!()
    }
}

pub struct RecommendedLogger {
    pub report: bool,
}

impl Log for RecommendedLogger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.report {
            report(&record)
        }

        if self.enabled(record.metadata()) {
            println!(
                "{}",
                match record.level() {
                    Level::Error => format!("[ERROR] {}", record.args()).red(),
                    Level::Warn => format!("[WARN ] {}", record.args()).yellow(),
                    Level::Info => format!("[INFO ] {}", record.args()).cyan(),
                    Level::Debug => format!("[DEBUG] {}", record.args()).green(),
                    Level::Trace => format!("[TRACE] {}", record.args()).black(),
                }
            );
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
    if set_logger(&RecommendedLogger { report: true }).is_err() {
        Err(SetLoggerError {})?;
    }
    set_max_level(LevelFilter::Trace);

    Ok(())
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

fn start_daemon() {
    todo!("Daemon is not running")
}
