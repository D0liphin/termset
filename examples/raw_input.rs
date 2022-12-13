// use std::io::{Read, Write};
use rtc::rtc_println;

mod tsc {
    pub use termset::core::*;
}

fn main() {
    rtc_println!("---");

    let mut termset = tsc::Termset::new();
    // termset.restore_on_sigint();
    termset.disable_lflag(tsc::ECHO | tsc::ICANON);
    termset.update(None);

    let mut token_builder = tsc::TokenBuilder::new();
    loop {
        if let Some(byte) = tsc::stdin_read_byte() {
            token_builder.feed(byte);
            if let Some(token) = token_builder.interpret() {
                let _ = rtc::REMOTE.send(&[token_builder.bytes()]);
                token_builder.clear();
            }
        } else {
            rtc_println!("error!");
        }
    }

    termset.restore();
}
