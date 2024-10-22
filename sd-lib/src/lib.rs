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
use serde::{Deserialize, Serialize};

unsafe fn sketchy<A, B: Copy>(a: A) -> B {
    *(&a as *const A as *const B)
}

pub const ADDRESS: &'static str = "127.0.0.1:1500";

pub const TRANSMISSION_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
pub enum Mode {
    Message(Message),
    Exit(u8),
    Auth(Auth),
}

impl Mode {
    pub fn transmit(&self, stream: &mut TcpStream) -> Result<()> {
        let message = serde_json::to_string(&self)?;

        stream.write(&message.len().to_le_bytes())?;
        stream.write(&message.as_bytes())?;

        Ok(())
    }

    pub fn recieve(stream: &mut TcpStream) -> Result<Mode> {
        let mut length = [0; 8];
        if stream.read(&mut length)? == 0 {
            todo!("TcpStream already closed.");
        }
        let length = usize::from_le_bytes(length);

        let mut binary = vec![0; length];
        if stream.read(&mut binary)? != length {
            todo!("Error reading from TcpStream.");
        }

        Ok(serde_json::from_str(str::from_utf8(&binary)?)?)
    }
}

pub const MESSAGE_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    version: u32,
    level: usize,
    message: String,
}

impl Message {
    pub fn new(level: Level, message: String) -> Self {
        Self {
            version: MESSAGE_VERSION,
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
        Mode::Message(self).transmit(stream)
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub mod error {
    use std::{error::Error, fmt::Display};

    #[derive(Debug)]
    pub struct InvalidVersion {
        version: u32,
    }

    impl Display for InvalidVersion {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "Version {} not supported for this operation",
                self.version
            )
        }
    }

    impl Error for InvalidVersion {}

    impl InvalidVersion {
        pub fn new(version: u32) -> Self {
            Self { version }
        }

        pub fn anyhow(self) -> anyhow::Error {
            self.into()
        }
    }
}

pub const AUTH_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct Auth {
    version: u32,
    pub identifier: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon_path: Option<String>,
}

impl Auth {
    pub fn new(name: String) -> Self {
        Self {
            name: Some(name),
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
            version: AUTH_VERSION,
            name: Some(identifier.clone()),
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
