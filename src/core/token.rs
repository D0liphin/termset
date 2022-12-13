// /// returns the number of bytes that this character will have for it to
// /// be a valid utf8 code point, returns None if idk what to do
// fn is_utf8_starting_byte(byte: u8) -> bool {
//     (byte & 0b11111000) == 0b11110000
//         || (byte & 0b11110000) == 0b11100000
//         || (byte & 0b11100000) == 0b11000000
//         || (byte & 0b10000000) == 0b00000000
// }
use std::{mem, ops, slice::EscapeAscii};

use rtc::rtc_println;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EscSeq {
    MoveUp(u32),
    MoveRight(u32),
    MoveDown(u32),
    MoveLeft(u32),
    Unknown,
}

#[derive(Debug)]
pub enum Token<'a> {
    Esc(EscSeq),
    Char(&'a str),
}

/// Feed bytes from stdin to this to interpret lots of different escape codes
///
/// There are lots of different things that ANSI terminals need to recognise.
/// Firstly, it will need to recognise utf-8 code points, but it will also need
/// to recognise various escape codes.
///
/// This struct is designed to be fed input bytes one after another and then tested
/// to see if it contains a meaningful sequence. This can be either a codepoint,
/// or an escape sequence, hence the return `Token` enum having two variants.
///
/// ```rs
/// let tb = TokenBuilder::new();
///
///
/// ```
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct TokenBuilder {
    bytes: [u8; 4],
    index: usize,
}

impl TokenBuilder {
    /// construct a new zero-initialised TokenBuilder
    pub fn new() -> Self {
        Self {
            bytes: [0; 4],
            index: 0,
        }
    }

    /// Feeds a byte to this TokenBuilder. After feeding a byte, you should use
    /// `TokenBuilder.interpret(&self)` to determine whether or not the token
    /// is valid.
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

    pub fn interpret<'a>(&'a self) -> Option<Token<'a>> {
        if self.index == 0 {
            return None;
        }
        if self.bytes[0] == 0x1b {
            if let Some(esc_seq) = self.get_esc_seq() {
                Some(Token::Esc(esc_seq))
            } else {
                None
            }
        } else {
            match std::str::from_utf8(&self.bytes[..self.index]) {
                Ok(result) => Some(Token::Char(result)),
                Err(_) => None,
            }
        }
    }

    /// Retrun a slice representing the bytes in this token builder, up to
    /// the internal index (so the returned slice will only be as long as
    /// the number of bytes fed to this TokenBuilder after the most recent 
    /// clear)
    pub fn bytes(&self) -> &[u8] {
        &self.bytes[0..self.index]
    }

    pub fn clear(&mut self) {
        self.index = 0;
    }

    /// Returns `None` if this is not an escape sequence and returns `Some(EsqSeq)`
    /// if it is, in fact and escape sequence.
    pub fn get_esc_seq(&self) -> Option<EscSeq> {
        if !self.is_ansi_control() {
            return None;
        } else {
            let byte_2 = self.bytes[2];
            Some(match byte_2 {
                b'A' => EscSeq::MoveUp(1),
                b'B' => EscSeq::MoveDown(1),
                b'C' => EscSeq::MoveRight(1),
                b'D' => EscSeq::MoveLeft(1),
                _ => return None,
            })
        }
    }

    /// Checks if this token contains the ANSI conrol escape sequence at the front
    /// (`[0x1b, 0x5b, ?]`) as well as at least 3 characters
    pub fn is_ansi_control(&self) -> bool {
        if self.index >= 3 {
            &self.bytes[0..2] == &[0x1b, 0x5b]
        } else {
            false
        }
    }

    pub fn is_full(&self) -> bool {
        self.index >= 4
    }
}
