use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ShiError {
    #[error("readline error")]
    ReadlineError(#[from] rustyline::error::ReadlineError),
    #[error("expected a non-zero number of args, got none")]
    NoArgs,
    #[error("expected no args, but got {got:?}")]
    ExtraArgs { got: Vec<String> },
    #[error("invalid sub command, got {got} but expected {expected:?}")]
    InvalidSubCommand { got: String, expected: Vec<String> },
    #[error("unrecognized command: {got}")]
    UnrecognizedCommand { got: String },
    #[error("command already registered: {cmd}")]
    AlreadyRegistered { cmd: String },
}