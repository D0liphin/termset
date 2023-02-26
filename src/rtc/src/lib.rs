use lazy_static::lazy_static;
use std::{fmt, io, mem, net, time};
pub mod server;
pub use crate::server::*;

pub const DEFAULT_ADDRESS: &'static str = "127.0.0.1:7777";

#[cfg(feature = "default_remote")]
lazy_static! {
    pub static ref REMOTE: RemoteTerminal =
        RemoteTerminal::new(DEFAULT_ADDRESS).expect("could not initialize default remote terminal");
}

lazy_static! {
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
        $crate::panic_if_err!($crate::REMOTE.println_tagged(""));
    };
    ($($arg:tt)*) => {{
        let message = format!($($arg)*);
        $crate::panic_if_err!($crate::REMOTE.println_tagged(&message));
    }};
}

struct MessageHeader {
    #[allow(dead_code)]
    size: u16,
}

// TODO: implement io::Write instead of fmt::Write
/// represents a link to a remote terminal via Udp. There are alll sorts of issues with
/// this implementation, namely busy waiting using `peek_from` which is explicitly
/// advised against. I don't really care since I have a lot of CPU cores to work with,
/// but this really needs fixing some day.
///
/// ## Advised usage:
/// ```rs
/// use rtc::RemoteTerminal;
/// use once_cell::sync::Lazy;
///
/// static RT: Lazy<RemoteTerminal> = Lazy::new(|| RemoteTerminal::new("127.0.0.1:7777"));
///
/// fn main() {
///     write!(RT, "Hello, world!");
/// }
/// ```
///
/// ## Also possible usage:
/// ```rs
/// use rtc::rtc_println;
///
/// fn main() {
///     rtc_println!("Hello, world!");
/// }
/// ```
pub struct RemoteTerminal {
    socket: net::UdpSocket,
    options: u32,
    start_time: time::SystemTime,
}

// TODO: rethink this implementation -- it forces us to act like `RemoteTerminal` is mutable
// when it isn't. Perhaops use a buffered writer which *is* actually mutable?
impl fmt::Write for RemoteTerminal {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let result = if (self.options & Self::PRINT_PROC_ID) > 0 {
            self.print_tagged(s)
        } else {
            self.send(&[s.as_bytes()])
        };
        match result {
            Err(_) => Err(fmt::Error),
            Ok(_) => Ok(()),
        }
    }
    fn write_fmt(&mut self, fmt_args: fmt::Arguments<'_>) -> fmt::Result {
        self.write_str(&format!("{}", fmt_args))
    }
}

const MAX_UDP_DATAGRAM_BYTES: usize = 65_507;

impl RemoteTerminal {
    /// The maximum number of bytes that you can send to the remote terminal, this is
    /// less than the max UDP datagram size as messages includes a custom `MessageHeader`
    pub const MAX_MESSAGE_SIZE: usize = MAX_UDP_DATAGRAM_BYTES - mem::size_of::<MessageHeader>();

    /// default option that includes the process ID when printing to the remote terminal
    pub const PRINT_PROC_ID: u32 = 0b1;

    pub const DEFAULT_OPTIONS: u32 = Self::PRINT_PROC_ID;

    /// create a link to a new terminal with default options
    pub fn new(address: &str) -> io::Result<Self> {
        Self::new_with_options(address, Self::DEFAULT_OPTIONS)
    }

    /// create a link to a new terminal with no options as default (all must be specified)
    ///
    /// probably just don't use this... the default options are nice and useful :)
    pub fn new_with_options(address: &str, options: u32) -> io::Result<Self> {
        let socket = net::UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(address)?;
        Ok(Self {
            socket,
            options,
            start_time: time::SystemTime::now(),
        })
    }

    fn print_process_info(&self, message: &str) -> io::Result<()> {
        self.send(&[format!("+{:-^78}+\n| {: ^76} |\n+{:-^78}+\n", "", message, "").as_bytes()])
    }

    /// prints a marker containing the process number and the time
    pub fn print_process_start(&self) -> io::Result<()> {
        self.print_process_info(&format!(
            "start [{}] at {}",
            std::process::id(),
            chrono::DateTime::<chrono::Local>::from(self.start_time).time()
        ))
    }

    /// prints a marker containing the process number and the time
    pub fn print_process_end(&self) -> io::Result<()> {
        let now = time::SystemTime::now();
        self.print_process_info(&format!(
            "finish [{}] at {:?} | took {:?}",
            std::process::id(),
            chrono::DateTime::<chrono::Local>::from(now).time(),
            if let Ok(duration) = now.duration_since(self.start_time) {
                format!("{:?}", duration)
            } else {
                String::from("???")
            }
        ))
    }

    /// concatenate `message_parts` and send it to the remote terminal, attaching the header
    ///
    /// You probably shouldn't use this, but do so at your own peril!!
    pub fn send(&self, message_parts: &[&[u8]]) -> io::Result<()> {
        // get headed size
        let mut len = 0;
        for &part in message_parts.iter() {
            len += part.len();
        }
        len += mem::size_of::<MessageHeader>();
        if len > u16::MAX as _ {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "cannot send datagram larger than {} bytes, got {} bytes",
                    Self::MAX_MESSAGE_SIZE,
                    len
                ),
            ));
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
    pub fn print_tagged(&self, message: &str) -> io::Result<()> {
        self.send(&[PROCESS_TAG.as_bytes(), message.as_bytes()])
    }

    /// Basic println function, which panics if it errors, instead of returning a result, it
    /// contains a process id tag
    pub fn println_tagged(&self, message: &str) -> io::Result<()> {
        self.send(&[PROCESS_TAG.as_bytes(), message.as_bytes(), b"\n"])
    }

    /// print an exact message to the remote terminal, this immediately flushes
    pub fn print(&self, message: &str) -> io::Result<()> {
        self.send(&[message.as_bytes()])
    }
}
