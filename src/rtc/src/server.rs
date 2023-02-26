use colored::Colorize;
use indoc::indoc;
use std::env;
use std::io;
use std::io::Write;
use std::mem::{size_of, transmute};
use std::net;

/// Prints an error to the user of this application
macro_rules! println_tagged_error {
    ($($arg:tt)*) => {
        println!("{}: {}", "error".red(), format!($($arg)*))
    }
}

/// Prints an error to the user of this application
macro_rules! println_tagged_warning {
    ($($arg:tt)*) => {
        println!("{}: {}", "warning".yellow(), format!($($arg)*))
    }
}

macro_rules! println_tagged_info {
    ($($arg:tt)*) => {
        println!("{}: {}", "info".green(), format!($($arg)*))
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
struct MessageHeader {
    size: u16,
}

const DEFAULT_ADDRESS: &'static str = "127.0.0.1:7777";

trait RecvFull {
    fn peek_header_from(&self) -> io::Result<MessageHeader>;
    fn recv_full_from(&self) -> io::Result<Vec<u8>>;
}

impl RecvFull for net::UdpSocket {
    /// Peek at the header of the next datagram from this socket, you can use this
    /// to get the length of the datagram. Assuming the datagram contains a `MessageHeader`
    /// as the first `size_of::<MessageHeader>()` bytes.
    ///
    /// This doesn't check if the datagram contains a header, it will just return
    /// a zeroed `MessageHeader` in that case
    fn peek_header_from(&self) -> io::Result<MessageHeader> {
        let mut buf = [0; std::mem::size_of::<MessageHeader>()];
        let _ = self.peek_from(&mut buf)?;
        let header: MessageHeader = unsafe { transmute(buf) };
        Ok(header)
    }

    /// Receive a full datagram into a `Vec<u8>`
    fn recv_full_from(&self) -> io::Result<Vec<u8>> {
        let header = self.peek_header_from()?;
        let datagram_length = header.size as usize;
        let mut datagram = Vec::<u8>::with_capacity(datagram_length);
        datagram.resize(datagram_length, 0);
        self.recv_from(datagram.as_mut_slice())?;
        Ok(datagram)
    }
}

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let address = if args.len() >= 2 {
        let arg = args[1].trim_end_matches(|c: char| c.is_whitespace() || c == '\'' || c == '"');
        match arg {
            "help" => {
                println!(
                    "{}",
                    indoc! {r#"
                        Remote terminal host for use with rtc
                        
                        USAGE:
                            remote-terminal [ADDRESS]?
                        
                        ADDRESS:
                            address is a valid IP address to start this host on, defaults to
                            127.0.0.1:7777, which is what rtc_println! prints to.
                    "#}
                );
                std::process::exit(1);
            }
            _ => arg,
        }
    } else {
        println_tagged_warning!("no address specified, using {}", DEFAULT_ADDRESS);
        DEFAULT_ADDRESS
    };
    println_tagged_info!("started remote terminal at {}!", address);

    let socket = match net::UdpSocket::bind(address) {
        Ok(s) => s,
        Err(e) => {
            println_tagged_error!("could not bind socket to {}", address);
            match e.kind() {
                io::ErrorKind::AddrInUse => println_tagged_error!("address is probably in use"),
                io::ErrorKind::AddrNotAvailable => {
                    println_tagged_error!("address is not available")
                }
                _ => (),
            }
            std::process::exit(1);
        }
    };

    loop {
        let datagram = match socket.recv_full_from() {
            Ok(d) => d,
            Err(_) => {
                println_tagged_error!("failed receiving datagram");
                continue;
            }
        };
        let message = match std::str::from_utf8(&datagram[size_of::<MessageHeader>()..]) {
            Ok(m) => m,
            Err(_) => {
                println_tagged_error!("datagram was not valid utf-8");
                continue;
            }
        };
        print!("{}", message);
        let _ = std::io::stdout().flush();
    }
}
