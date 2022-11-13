use std::io::{Read, Write};
mod tsc {
    pub use termset::termset_core::*;
}

fn main() {
    let mut termset = tsc::Termset::new();
    termset.disable_lflag(tsc::ECHO | tsc::ICANON);
    termset.update(None);
    
    loop {
        if let Some(byte) = tsc::stdin_read_byte() {
            if byte.is_ascii_control() {
                print!("<ctrl>");
            } else {
                print!("{}", byte as char);
            }
            let _ = std::io::stdout().lock().flush();
        } else {
            break;
        }
    }

    termset.restore();
}
