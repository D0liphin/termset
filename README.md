# Termset

The idea behind termset, is to provide a barebones interface for interacting with the terminal. 
It is not in any way supposed to be a complete TUI library, but is intended to be used as an 
alternative backend to something like ncurses.

I have no intention of making this compatible with anything other than Linux, however, I believe
this should actually work just fine on Windows right now, though I do not care.

### Desired Functionality

I'm checking these off but they're hardly done really...

- [x] Easily clear the terminal, storing the previous contents
- [ ] Acquire information about the terminal reactively
    - [ ] terminal size
    - [ ] mouse cursor location (as terminal coordinates)
- [x] Easily switch to a raw input mode, allowing the user to 
    - [x] write a buffer to a specific location on the terminal
    - [x] control the cursor (and keep track of its location)
- [x] Easily restore the terminal to its previous state (`Termset::restore(self)`)

This is (pretty much) an exhaustive list. I intend to make a more complex TUI library on top of 
this, but I know a lot of people want to do that kind of thing on their own, so I'm hoping this
can act as an easy-to-learn backend for people who want to make their own TUI (for fun).

```rs
fn prepare_terminal() -> (tsc::Termset, tsc::TsStdout) {
    // turn off all those pesky defaults!
    let mut termset = tsc::Termset::new().unwrap();
    termset.disable_lflag(tsc::ECHO | tsc::ICANON | tsc::ISIG);
    termset.update(None);

    // locally buffered handle to stdout
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

    // reads chars and esc seqs from stdin
    let mut tr = tsc::TokenReader::new();
    loop {
        let token = tr.next();
        rtc_println!("token = {:?}", token); // using rtc to print on another terminal

        if token == tsc::Token::Char("\u{3}") {
            // break if we get Ctrl-C SIGINT
            break;
        }

        match token {
            tsc::Token::Char(c) => match c {
                // basic backspace
                "\u{7f}" => {
                    stdout
                        .exec([&StdoutEscSeq::MoveLeft(1)])
                        .write_str(" ")
                        .exec([&StdoutEscSeq::MoveLeft(1)]);
                }
                // or just write the codepoint
                _ => {
                    stdout.write_str(c);
                }
            },
            // just forward the esc seq to stdout
            tsc::Token::Esc(seq) => {
                stdout.exec([&seq.as_stdout_esc_seq().unwrap()]);
            }
        }
        
        // syscall
        stdout.flush();
    }

    restore_terminal(termset, stdout);
}

```
