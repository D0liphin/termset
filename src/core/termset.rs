use libc;

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

// characters are echoed as they are received
pub const ECHO: LFlag = libc::ECHO as LFlag;
// input is line by line
pub const ICANON: LFlag = libc::ICANON as LFlag;

impl Termset {
    pub fn new() -> Self {
        let termios = unsafe {
            // using things from <termios.h> here. <termios.h> defines the structure of the
            // termios file, which provides the terminal interface for POSIX compatibility. My
            // understanding of what this means is that stdin contains some file attributes
            // which you can modify to change the way the terminal works. This is just the POSIX
            // compliant way of doing this
            let mut termios: libc::termios = std::mem::zeroed();
            libc::tcgetattr(libc::STDIN_FILENO, &mut termios as *mut libc::termios);
            termios
        };
        Termset {
            entry_config: Box::new(termios),
            config: termios,
        }
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

#[cfg(target_os = "linux")]
pub fn stdin_read_byte() -> Option<u8> {
    unsafe {
        let mut byte: u8 = 0;
        let bytes_read = libc::read(
            libc::STDIN_FILENO,
            &mut byte as *mut u8 as *mut libc::c_void,
            1,
        );
        if bytes_read == 1 {
            Some(byte)
        } else {
            None
        }
    }
}
