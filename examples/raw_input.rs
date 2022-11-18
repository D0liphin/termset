use std::io::{Read, Write};

use nix::sys::ptrace::interrupt;
mod tsc {
    pub use termset::core::*;
}

fn main() {
    let mut termset = tsc::Termset::new();
    termset.disable_lflag(tsc::ECHO | tsc::ICANON);
    termset.update(None);

    let mut token_builder = tsc::TokenBuilder::new();
    loop {
        if let Some(byte) = tsc::stdin_read_byte() {
            token_builder.feed(byte);
            if let Some(token) = token_builder.interpret() {
                println!("token = {:?}", token);
                token_builder.clear();
            }
        } else {
            break;
        }
    }

    termset.restore();
}
