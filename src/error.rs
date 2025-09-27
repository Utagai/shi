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
    #[error("unrecognized command: '{got}'")]
    UnrecognizedCommand { got: String },
    #[error("command already registered: {cmd}")]
    AlreadyRegistered { cmd: String },
    #[error("command failed to parse: {msg}")]
    ParseError {
        msg: String,
        possibilities: Vec<String>,
        cmd_path: Vec<String>,
        remaining: Vec<String>,
    },
    #[error("error: {msg}")]
    General { msg: String },
}

impl ShiError {
    pub fn general<S: AsRef<str>>(msg: S) -> ShiError {
        ShiError::General {
            msg: msg.as_ref().to_string(),
        }
    }
}
