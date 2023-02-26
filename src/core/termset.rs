use crate::core::err::*;
use libc::{self, EBADF, ENOTTY};
use nix::errno::errno;
use std::mem;

#[cfg(target_os = "linux")]
pub struct Termset {
    entry_config: Box<libc::termios>,
    config: libc::termios,
}

type TCSA = i32;

pub const TCSA_NOW: TCSA = libc::TCSANOW as TCSA;
pub const TCSA_DRAIN: TCSA = libc::TCSADRAIN as TCSA;
pub const TCSA_FLUSH: TCSA = libc::TCSAFLUSH as TCSA;

type LFlag = u32;

/// characters are echoed as they are received
pub const ECHO: LFlag = libc::ECHO as LFlag;
/// input is line by line
pub const ICANON: LFlag = libc::ICANON as LFlag;
/// suspend process on CTRL-C or CTRL-Z
pub const ISIG: LFlag = libc::ISIG as LFlag;

impl Termset {
    /// TODO: docs
    pub fn new() -> Result<Self, TermsetCreationError> {
        let termios = unsafe {
            // using things from <termios.h> here. <termios.h> defines the structure of the
            // termios file, which provides the terminal interface for POSIX compatibility. My
            // understanding of what this means is that stdin contains some file attributes
            // which you can modify to change the way the terminal works. This is just the POSIX
            // compliant way of doing this
            let mut termios = mem::MaybeUninit::uninit();
            let e = libc::tcgetattr(libc::STDIN_FILENO, termios.as_mut_ptr());
            if e == -1 {
                return Err(match errno() {
                    EBADF => TermsetCreationError::BadFileDescriptor,
                    ENOTTY => TermsetCreationError::NotATerminal,
                    _ => unreachable!(),
                });
            } else {
                termios.assume_init()
            }
        };

        Ok(Termset {
            entry_config: Box::new(termios),
            config: termios,
        })
    }

    /// Push updates to the terminal, optional_actions is an optional bitset
    /// Find out about the actions https://www.ibm.com/docs/en/aix/7.2?topic=files-termiosh-file
    pub fn update(&self, optional_actions: Option<TCSA>) {
        let optional_actions = optional_actions.unwrap_or(TCSA_FLUSH);
        unsafe {
            libc::tcsetattr(
                libc::STDIN_FILENO,
                optional_actions as libc::c_int,
                &self.config as *const libc::termios,
            );
        }
    }

    /// Reset the terminal to its entry settings
    pub fn reset(&self) {
        unsafe {
            libc::tcsetattr(
                libc::STDIN_FILENO,
                libc::TCSANOW,
                &*self.entry_config as *const libc::termios,
            );
        }
    }

    // /// register a signal hook to restore the terminal on `SIGINT`
    // pub fn restore_on_sigint(&'static mut self) {
    //     unsafe {
    //         signal_hook::low_level::register(libc::SIGINT,  move || {self.restore()});
    //     }
    // }

    /// Restore instantly, consuming this object
    pub fn restore(self) {
        self.reset();
    }

    pub fn disable_lflag(&mut self, flags: LFlag) {
        self.config.c_lflag &= !flags;
    }
}

impl Drop for Termset {
    fn drop(&mut self) {
        self.reset();
    }
}
