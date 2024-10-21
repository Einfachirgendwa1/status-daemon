use std::{
    net::{Shutdown, TcpStream},
    sync::{Arc, Mutex},
};

use log::Level;
use sd_lib::{Message, ADDRESS};

static mut STREAM: Option<Arc<Mutex<TcpStream>>> = None;

pub fn init() {
    try_connect();
}

pub fn report(record: &log::Record) {
    if let Some(stream) = unsafe { STREAM.as_ref() } {
        if record.level() <= Level::Info {
            Message::new(record.level(), record.args().to_string())
                .send(&mut stream.lock().unwrap())
                .unwrap()
        } else {
            todo!()
        }
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

pub fn close_connection() {
    if let Some(stream) = unsafe { STREAM.as_ref() } {
        stream.lock().unwrap().shutdown(Shutdown::Both).unwrap()
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
