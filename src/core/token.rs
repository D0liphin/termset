/// returns the number of bytes that this character will have for it to
/// be a valid utf8 code point, returns None if idk what to do
fn is_utf8_starting_byte(byte: u8) -> bool {
    (byte & 0b11111000) == 0b11110000
        || (byte & 0b11110000) == 0b11100000
        || (byte & 0b11100000) == 0b11000000
        || (byte & 0b10000000) == 0b00000000
}

#[derive(Debug)]
pub enum EscSeq {
    UpArrow,
    RightArrow,
    DownArrow,
    LeftArrow,
    Unknown,
}

#[derive(Debug)]
pub enum Token<'a> {
    Esc(EscSeq),
    Char(&'a str),
}

/// Feed bytes from stdin to this to interpret lots of different escape codes
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct TokenBuilder {
    bytes: [u8; 4],
    index: usize,
}

impl TokenBuilder {
    /// construct a new zero-initialised TokenBuilder
    pub fn new() -> Self {
        Self {
            bytes: <[u8; 4]>::default(),
            index: 0,
        }
    }

    pub fn feed(&mut self, byte: u8) {
        if self.is_full() {
            panic!("cannot feed byte to full TokenBuilder")
        }
        unsafe { self.feed_unchecked(byte) }
    }

    pub unsafe fn feed_unchecked(&mut self, byte: u8) {
        *self.bytes.get_unchecked_mut(self.index) = byte;
        self.index += 1;
    }

    pub fn interpret(&self) -> Option<Token> {
        if self.index == 0 {
            return None;
        }
        unsafe { self.interpret_unchecked() }
    }

    pub unsafe fn interpret_unchecked<'a>(&'a self) -> Option<Token<'a>> {
        if self.is_ansi_control() {
            todo!()
        } else {
            match std::str::from_utf8(&self.bytes[..self.index]) {
                Ok(result) => Some(Token::Char(result)),
                Err(_) => None,
            }
        }
    }

    pub fn clear(&mut self) {
        self.index = 0;
    }

    pub fn is_ansi_control(&self) -> bool {
        if self.index == 0 {
            false
        } else {
            unsafe { self.is_ansi_control_unchecked() }
        }
    }

    pub unsafe fn is_ansi_control_unchecked(&self) -> bool {
        *self.bytes.get_unchecked(0) == 0x1b
    }

    pub fn is_full(&self) -> bool {
        self.index >= 4
    }
}
