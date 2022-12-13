use lazy_static::lazy_static;
use std::{fmt, io, mem, net};

pub const DEFAULT_ADDRESS: &'static str = "127.0.0.1:7777";

#[cfg(feature = "default_remote")]
lazy_static! {
    pub static ref REMOTE: RemoteTerminal =
        RemoteTerminal::new(DEFAULT_ADDRESS).expect("could not initialize default remote terminal");
    static ref PROCESS_TAG: String = format!("[{}] ", std::process::id());
}

#[macro_export]
macro_rules! panic_if_err {
    ($result:expr) => {
        match $result {
            Ok(_) => (),
            Err(e) => panic!("{}", e),
        }
    };
}

/// println! to the default remote terminal at `rtc::DEFAULT_ADDRESS`
#[cfg(feature = "default_remote")]
#[macro_export]
macro_rules! rtc_println {
    () => {
        $crate::panic_if_err!($crate::REMOTE.print_tagged("\n"));
    };
    ($($arg:tt)*) => {{
        let mut message = format!($($arg)*);
        message.push('\n');
        $crate::panic_if_err!($crate::REMOTE.print_tagged(&[message.as_bytes()]));
    }};
}

struct MessageHeader {
    #[allow(dead_code)]
    size: u16,
}

pub struct RemoteTerminal {
    socket: net::UdpSocket,
}

#[derive(Debug)]
pub enum RemoteTerminalError {
    MessageTooLarge,
    IoError(io::Error),
}

impl From<io::Error> for RemoteTerminalError {
    fn from(error: io::Error) -> Self {
        Self::IoError(error)
    }
}

impl fmt::Display for RemoteTerminalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "cannot send a message with length greater than u16::MAX bytes"
        )
    }
}

impl RemoteTerminal {
    pub fn new(address: &str) -> io::Result<Self> {
        let socket = net::UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(address)?;
        Ok(Self { socket })
    }

    /// concatenate `message_parts` and send it to the remote terminal, attaching the header
    /// 
    /// You probably shouldn't use this, but do so at your own peril!!
    pub fn send(&self, message_parts: &[&[u8]]) -> Result<(), RemoteTerminalError> {
        // get headed size
        let mut len = 0;
        for &part in message_parts.iter() {
            len += part.len();
        }
        len += mem::size_of::<MessageHeader>();
        if len > u16::MAX as _ {
            return Err(RemoteTerminalError::MessageTooLarge);
        }

        let mut headed_message = Vec::<u8>::with_capacity(len);
        headed_message.extend(unsafe {
            mem::transmute::<MessageHeader, [u8; mem::size_of::<MessageHeader>()]>(MessageHeader {
                size: len as u16,
            })
        });

        for &part in message_parts.iter() {
            headed_message.extend(part);
        }
        self.socket.send(&headed_message)?;

        Ok(())
    }

    /// Prints a tagged message to the remote terminal, the tag includes the process id of the
    /// the process from which the message is sent.
    ///
    /// The message is a list of message parts to copy from (so that you don't need to copy
    /// the parts yourself before sending the message)
    pub fn print_tagged(&self, message_parts: &[&[u8]]) -> Result<(), RemoteTerminalError> {
        let mut message_parts_vec = Vec::with_capacity(message_parts.len() + 1);
        message_parts_vec.push(PROCESS_TAG.as_bytes());
        for &part in message_parts {
            message_parts_vec.push(part);
        }
        self.send(&message_parts_vec)?;
        Ok(())
    }

    /// Basic println function, which panics if it errors, instead of returning a result, it
    /// contains a process id tag
    pub fn println_tagged(&self, message: &str) {
        panic_if_err!(self.print_tagged(&[message.as_bytes()]));
    }
}
