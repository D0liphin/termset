use libc::c_void;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StdinEscSeq {
    MoveUp(u32),
    MoveRight(u32),
    MoveDown(u32),
    MoveLeft(u32),
}

impl StdinEscSeq {
    /// Tries to convert this to an `StdoutEscSeq` -- what this means might not be obvious...
    /// An example of one that *is* obvious would be `MoveUp(u32)`, which is translated to
    /// the StdoutEscSeq of the same name
    pub fn as_stdout_esc_seq(&self) -> Option<StdoutEscSeq> {
        match *self {
            Self::MoveUp(count) => Some(StdoutEscSeq::MoveUp(count)),
            Self::MoveRight(count) => Some(StdoutEscSeq::MoveRight(count)),
            Self::MoveDown(count) => Some(StdoutEscSeq::MoveDown(count)),
            Self::MoveLeft(count) => Some(StdoutEscSeq::MoveLeft(count)),
            // _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StdoutEscSeq {
    MoveUp(u32),
    MoveRight(u32),
    MoveDown(u32),
    MoveLeft(u32),
    SaveScreen,
    RestoreScreen,
    SaveCursorPosition,
    RestorCursorPosition,
    EraseEntireScreen,
}

macro_rules! esc_seq {
    [$($value:expr),*] => {
        {
            let mut seq = vec![];
            seq.extend(&ESC_SEQ_PREFIX);
            $(
                seq.extend(format!("{}", $value).as_bytes());
            )*
            seq
        }
    };
}

impl StdoutEscSeq {
    pub fn as_bytes(&self) -> Vec<u8> {
        match *self {
            Self::MoveUp(count) => esc_seq![count, 'A'],
            Self::MoveRight(count) => esc_seq![count, 'C'],
            Self::MoveDown(count) => esc_seq![count, 'B'],
            Self::MoveLeft(count) => esc_seq![count, 'D'],
            Self::SaveScreen => esc_seq!["?47h"],
            Self::RestoreScreen => esc_seq!["?47l"],
            Self::SaveCursorPosition => vec![ESC_ASCII, 7], // TODO: fix
            Self::RestorCursorPosition => vec![ESC_ASCII, 8],
            Self::EraseEntireScreen => esc_seq!["2J"],
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Token<'a> {
    Esc(StdinEscSeq),
    Char(&'a str),
}

/// The size of the TokenReader's internal buffer
pub const BUF_SIZE: usize = 4096;

pub const ESC_ASCII: u8 = 0x1b;
pub const ESC_SEQ_PREFIX: [u8; 2] = [0x1b, 0x5b];
pub const MAX_ESC_SEQ_LENGTH: usize = 1 + ESC_SEQ_PREFIX.len();

pub struct TokenReader {
    buf: [u8; BUF_SIZE],
    index: usize,
    end: usize,
}

impl TokenReader {
    pub fn new() -> Self {
        Self {
            buf: [0; BUF_SIZE],
            index: 0,
            end: 0,
        }
    }

    /// Update internal buffer with a read from stdin
    fn update_buf(&mut self) {
        // TODO: make unchecked after testing
        // shift unused bytes to the front
        self.buf.copy_within(self.index..self.end, 0);
        let dst_index = self.end - self.index;

        // TODO: https://man7.org/linux/man-pages/man2/read.2.html
        //       - implement errno mapping
        // TODO: make this only read `MAX_ESC_SEQ_LENGTH + 1` to start, then try again
        //       if we read that many bytes, this time filling the whole buffer (or
        //       as much as we can)
        let bytes_read = unsafe {
            libc::read(
                libc::STDIN_FILENO,
                self.buf.as_mut_ptr().add(dst_index) as *mut c_void,
                BUF_SIZE - dst_index,
            )
        };

        self.index = 0;

        // failed read, retry instantly
        if bytes_read == -1 {
            self.end = 0;
            return;
        }

        self.end = bytes_read as usize;
    }

    unsafe fn get_relative(&self, offset: usize) -> &u8 {
        self.buf.get_unchecked(self.index + offset)
    }

    unsafe fn get_relative_slice(&self, slice_start: usize, slice_end: usize) -> &[u8] {
        std::slice::from_raw_parts(self.get_relative(0) as *const u8, slice_end - slice_start)
    }

    fn bytes_left(&self) -> usize {
        self.end - self.index
    }

    /// read the next bytes as an esc seq and segfault/panic/something-else if we fail
    /// (that's why it's `unsafe`)
    unsafe fn read_esc_seq(&mut self) -> StdinEscSeq {
        let old_index = self.index;
        self.index += ESC_SEQ_PREFIX.len() + 1;
        match self.buf[old_index + 2] {
            b'A' => StdinEscSeq::MoveUp(1),
            b'B' => StdinEscSeq::MoveDown(1),
            b'C' => StdinEscSeq::MoveRight(1),
            b'D' => StdinEscSeq::MoveLeft(1),
            _ => unimplemented!(),
        }
    }

    /// if we can read a char, return `(true, char_size_in_bytes)`
    fn can_read_char(&self) -> (bool, usize) {
        let byte = self.buf[self.index];

        // TODO: check if this is ok!
        if byte == ESC_ASCII {
            return (false, 0);
        }

        // TODO: test
        fn utf8_codepoint_length(first_byte: u8) -> i8 {
            let mut shifted = first_byte >> 3;
            if shifted == 0b11110 {
                return 4;
            }
            shifted >>= 1;
            if shifted == 0b1110 {
                return 3;
            }
            shifted >>= 1;
            if shifted == 0b110 {
                return 2;
            }
            shifted >>= 2;
            if shifted == 0b0 {
                return 1;
            }
            return -1;
        }

        // this can't really be invalid utf8, but if it is, just loop forever ig
        // TODO: umm... fix this?
        let len = utf8_codepoint_length(byte);
        if len == -1 {
            return (false, 0);
        }

        // check we're not cut off
        let len = len as usize;
        if self.bytes_left() < len {
            return (false, 0);
        }

        // check this is valid utf8
        (
            std::str::from_utf8(unsafe { self.get_relative_slice(0, len) }).is_ok(),
            len,
        )
    }

    /// read the next bytes as a char and segfault/panic/something-else if we fail
    /// (that's why it's `unsafe`)
    unsafe fn read_char(&mut self, len: usize) -> &str {
        let old_index = self.index;
        self.index += len;
        std::str::from_utf8_unchecked(&self.buf[old_index..self.index])
    }

    /// Get the next Token from stdin
    pub fn next(&mut self) -> Token {
        loop {
            if self.end == 0 {
                self.update_buf();
                continue;
            }

            // because of Polonius the Crab... something something... slow borrow checker
            // if enabled... all the code goes in here:

            let fits_esc_seq_prefix = self.bytes_left() >= ESC_SEQ_PREFIX.len();
            let fits_esc_seq = self.bytes_left() >= MAX_ESC_SEQ_LENGTH;
            if fits_esc_seq_prefix
                && unsafe { [*self.get_relative(0), *self.get_relative(1)] } == ESC_SEQ_PREFIX
            {
                if fits_esc_seq {
                    return unsafe { Token::Esc(self.read_esc_seq()) };
                } else {
                    self.update_buf();
                    continue;
                }
            }

            let (can_read_char, char_len) = self.can_read_char();
            if can_read_char {
                return unsafe { Token::Char(self.read_char(char_len)) };
            }

            self.update_buf();
        }
    }
}