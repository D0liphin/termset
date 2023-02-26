/// Indicates an issue when trying to create a Termset instance.
/// You can safely ignore the error variant for most applications on a result with
/// this return type, as it is effectively guaranteed not to happen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TermsetCreationError {
    /// Case is thrown if for some reason your `STDIN_FILENO` is not a valid
    /// file descriptor... No idea why that would happen!
    BadFileDescriptor,
    /// Case is thrown if for some reason your `STDIN_FILENO` is not a terminal 
    /// file descriptor... No idea why that would happen either!
    NotATerminal,
}

