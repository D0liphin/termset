use std::io::{self, Write};

use libc::{c_void, STDIN_FILENO};

use super::StdoutEscSeq;

/// Wrap this in a `io::BufWriter` to get a faster handle to stdout, but without your
/// write buffer being synchronised (which is literally fine... like who cares?)
pub struct UnsafeStdout;

impl io::Write for UnsafeStdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let bytes_written =
            unsafe { libc::write(STDIN_FILENO, buf.as_ptr() as *const c_void, buf.len()) };

        if bytes_written == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(bytes_written as usize)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Wrapper around Stdout -- uses ```UnsafeStdout``` internally, so is buffered thread-locally.
/// All commands do not return io errors, as these are basically always ignored and just look
/// gross. when you have to `let _ =` or `.unwrap()`
pub struct TsStdout {
    buf_writer: io::BufWriter<UnsafeStdout>,
}

impl TsStdout {
    pub fn new() -> Self {
        Self {
            buf_writer: io::BufWriter::new(UnsafeStdout),
        }
    }

    /// execute a serious of commands (escape sequences) in order
    pub fn exec<'a, T>(&mut self, commands: T) -> &mut Self
    where
        T: IntoIterator<Item = &'a StdoutEscSeq>,
    {
        for command in commands.into_iter() {
            let _ = self.buf_writer.write_all(&command.as_bytes());
        }
        self
    }

    /// write a string to stdout
    pub fn write_str(&mut self, s: &str) -> &mut Self {
        let _ = self.buf_writer.write_all(s.as_bytes());
        self
    }

    pub fn flush(&mut self) -> &mut Self {
        let _ = self.buf_writer.flush();
        self
    }    
}
