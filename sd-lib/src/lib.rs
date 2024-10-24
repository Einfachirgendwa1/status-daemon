use core::str;
use std::{
    env::current_exe,
    fmt::Display,
    io::{Read, Write},
    net::TcpStream,
};

use anyhow::Result;
use colored::Colorize;
use log::{debug, error, info, trace, warn, Level};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

unsafe fn sketchy<A, B: Copy>(a: A) -> B {
    *(&a as *const A as *const B)
}

pub const ADDRESS: &'static str = "127.0.0.1:1500";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecievedMessage {
    pub message: Message,
    pub origin: u32,
}

pub trait Transmission: Serialize + DeserializeOwned + Clone {
    fn transmit(&self, stream: &mut TcpStream) -> Result<()> {
        let message = serde_json::to_string(&self)?;

        stream.write(&message.len().to_le_bytes())?;
        stream.write(&message.as_bytes())?;

        Ok(())
    }

    fn recieve(stream: &mut TcpStream) -> Result<Self> {
        let mut length = [0; 8];
        if stream.read(&mut length)? == 0 {
            todo!("TcpStream already closed.");
        }
        let length = usize::from_le_bytes(length);

        let mut binary = vec![0; length];
        if stream.read(&mut binary)? != length {
            todo!("Error reading from TcpStream.");
        }

        Ok(serde_json::from_slice(&binary)?)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum RandomProgramToDaemon {
    Message(Message),
    Exit(u8),
    Auth(Auth),
    NewClient,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum DaemonToClient {
    RecievedMessage(RecievedMessage),
}

impl Transmission for RandomProgramToDaemon {}
impl Transmission for DaemonToClient {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    level: usize,
    message: String,
}

impl Message {
    pub fn new(level: Level, message: String) -> Self {
        Self {
            level: level as usize,
            message,
        }
    }

    pub fn display(&self) {
        // INFO: Maybe use log::log!() instead?
        match unsafe { sketchy(self.level) } {
            Level::Error => error!("{self}"),
            Level::Warn => warn!("{self}"),
            Level::Info => info!("{self}"),
            Level::Debug => debug!("{self}"),
            Level::Trace => trace!("{self}"),
        }
    }

    pub fn send(self, stream: &mut TcpStream) -> Result<()> {
        RandomProgramToDaemon::Message(self).transmit(stream)
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Auth {
    pub identifier: String,
    pub name: String,
    pub description: Option<String>,
    pub icon_path: Option<String>,
}

impl Auth {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

impl Default for Auth {
    fn default() -> Self {
        let identifier = current_exe()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        Self {
            name: identifier.clone(),
            identifier,
            description: None,
            icon_path: None,
        }
    }
}

pub fn print_record(record: &log::Record) {
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
