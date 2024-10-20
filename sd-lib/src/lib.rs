use std::{io::Write, net::TcpStream};

use anyhow::{Context, Result};
use log::Level;

pub const ADDRESS: &'static str = "127.0.0.1:1500";

pub const MESSAGE_VERSION: u32 = 1;
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
}

unsafe fn sketchy<A, B: Copy>(a: A) -> B {
    *(&a as *const A as *const B)
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
    pub fn send(&self, stream: &mut TcpStream) -> Result<()> {
        stream
            .write(vec!['M' as u8].as_slice())
            .context("Failed send character 'M' to daemon.")?;
        stream
            .write(&self.make_sendeable())
            .context("Failed to send message to daemon.")?;
        Ok(())
    }

    fn make_sendeable(&self) -> Vec<u8> {
        format!("{}{}{}", self.version, self.level as u8, self.message)
            .as_bytes()
            .to_vec()
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
