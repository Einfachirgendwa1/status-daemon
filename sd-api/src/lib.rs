use std::{
    net::TcpStream,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use log::Level;
use sd_lib::{Message, ADDRESS};

static mut STREAM: Option<Arc<Mutex<TcpStream>>> = None;

pub fn init() -> Result<()> {
    try_connect();

    Ok(())
}

pub fn send_test_message(message: &str) -> Result<()> {
    if let Some(stream) = unsafe { STREAM.as_ref() } {
        Message::new(Level::Info, message.to_string()).send(&mut stream.lock().unwrap())
    } else {
        todo!()
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

fn start_daemon() {
    todo!("Daemon is not running")
}
