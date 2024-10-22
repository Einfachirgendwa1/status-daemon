use std::{
    fmt::Display,
    io::{Read, Write},
    net::TcpStream,
};

use anyhow::{Context, Result};
use colored::Colorize;
use log::{debug, error, info, trace, warn, Level};

unsafe fn sketchy<A, B: Copy>(a: A) -> B {
    *(&a as *const A as *const B)
}

pub const ADDRESS: &'static str = "127.0.0.1:1500";

pub const TRANSMISSION_VERSION: u32 = 1;
pub struct Transmission {
    version: u32,
    mode: Mode,
}

pub enum Mode {
    Message(Message),
    Exit(u8),
}

impl Transmission {
    pub fn new(mode: Mode) -> Self {
        Transmission {
            version: TRANSMISSION_VERSION,
            mode,
        }
    }

    pub fn transmit(&self, stream: &mut TcpStream) -> Result<()> {
        stream
            .write(&self.version.to_le_bytes())
            .context("Failed to send version number to daemon.")?;

        let message = match self.mode {
            Mode::Message(ref message) => {
                let mut vec = vec!['M' as u8];
                vec.extend_from_slice(message.make_sendeable().as_slice());
                vec
            }
            Mode::Exit(code) => {
                vec!['E' as u8, code]
            }
        };

        stream.write(&message.len().to_le_bytes())?;
        stream.write(&message)?;

        Ok(())
    }

    pub fn recieve(stream: &mut TcpStream) -> Result<Mode> {
        let mut version = [0; 4];

        if stream.read(&mut version)? == 0 {
            todo!("TcpStream already closed.");
        }
        let version = u32::from_le_bytes(version);

        match version {
            1 => Self::recieve_v1(stream),
            _ => todo!("Invalid version"),
        }
    }

    pub fn recieve_v1(stream: &mut TcpStream) -> Result<Mode> {
        let mut length = [0; 8];

        if stream.read(&mut length)? == 0 {
            todo!("TcpStream already closed.");
        }
        let length = usize::from_le_bytes(length);

        let mut binary = vec![0; length];

        if stream.read(&mut binary)? == 0 {
            todo!("TcpStream already closed.");
        }

        match binary[0] as char {
            'M' => Ok(Mode::Message(Message::from_sendeable(&binary[1..])?)),
            'E' => Ok(Mode::Exit(binary[1])),
            _ => todo!(),
        }
    }
}

pub const MESSAGE_VERSION: u32 = 1;

#[derive(Debug, Clone)]
pub struct Message {
    version: u32,
    level: Level,
    message: String,
}

impl Message {
    pub fn new(level: Level, message: String) -> Self {
        Self {
            version: MESSAGE_VERSION,
            level,
            message,
        }
    }

    pub fn display(&self) {
        // INFO: Maybe use log::log!() instead?
        match self.level {
            Level::Error => error!("{self}"),
            Level::Warn => warn!("{self}"),
            Level::Info => info!("{self}"),
            Level::Debug => debug!("{self}"),
            Level::Trace => trace!("{self}"),
        }
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

impl Message {
    pub fn send(self, stream: &mut TcpStream) -> Result<()> {
        Transmission::new(Mode::Message(self)).transmit(stream)
    }

    pub fn make_sendeable(&self) -> Vec<u8> {
        let mut binary = Vec::new();

        binary.append(&mut (self.version as u32).to_le_bytes().to_vec());
        binary.append(&mut (self.level as u32).to_le_bytes().to_vec());
        binary.append(&mut self.message.as_bytes().to_vec());

        binary
    }

    pub fn from_sendeable(bytes: &[u8]) -> Result<Self> {
        let mut array = [0; 4];
        array.copy_from_slice(&bytes[..4]);
        let version = u32::from_le_bytes(array);

        match version {
            1 => Ok(Self::from_sendeable_v1(bytes)),
            version => Err(error::InvalidVersion::new(version).anyhow())
                .context("Failed to read a message."),
        }
    }

    fn from_sendeable_v1(bytes: &[u8]) -> Self {
        let mut level = [0; 4];
        level.copy_from_slice(&bytes[4..8]);
        let level = u32::from_le_bytes(level) as usize;
        let level = unsafe { sketchy(level) };

        let message = String::from_utf8_lossy(&bytes[8..]).to_string();

        Self {
            version: 1,
            level,
            message,
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
