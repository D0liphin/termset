use rtc::rtc_println;
use std::{
    io::{self, Stdout, Write},
    thread, time,
};
use termset::core::StdoutEscSeq;

mod tsc {
    pub use termset::core::*;
}

fn prepare_terminal() -> (tsc::Termset, tsc::TsStdout) {
    let mut termset = tsc::Termset::new().unwrap();
    termset.disable_lflag(tsc::ECHO | tsc::ICANON | tsc::ISIG);
    termset.update(None);

    let mut stdout = tsc::TsStdout::new();
    stdout
        .exec([
            &StdoutEscSeq::SaveCursorPosition,
            &StdoutEscSeq::SaveScreen,
            &StdoutEscSeq::EraseEntireScreen,
        ])
        .flush();

    (termset, stdout)
}

fn restore_terminal(termset: tsc::Termset, mut ts_stdout: tsc::TsStdout) {
    ts_stdout
        .exec([
            &StdoutEscSeq::RestorCursorPosition,
            &StdoutEscSeq::RestoreScreen,
        ])
        .flush();
    termset.restore();
}

fn main() {
    let (termset, mut stdout) = prepare_terminal();

    let mut tr = tsc::TokenReader::new();
    loop {
        let token = tr.next();
        rtc_println!("token = {:?}", token);

        if token == tsc::Token::Char("\u{3}") {
            break;
        }

        match token {
            tsc::Token::Char(c) => match c {
                "\u{7f}" => {
                    stdout
                        .exec([&StdoutEscSeq::MoveLeft(1)])
                        .write_str(" ")
                        .exec([&StdoutEscSeq::MoveLeft(1)]);
                }
                _ => {
                    stdout.write_str(c);
                }
            },
            tsc::Token::Esc(seq) => {
                stdout.exec([&seq.as_stdout_esc_seq().unwrap()]);
            }
        }

        stdout.flush();
    }

    restore_terminal(termset, stdout);
}
