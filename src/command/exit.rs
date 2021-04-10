use std::marker::PhantomData;

use super::BaseCommand;
use crate::error::ShiError;
use crate::shell::Shell;
use crate::Result;

// TODO: This should be private.
#[derive(Debug)]
/// ExitCommand is a command that triggers a termination of the shell.
pub struct ExitCommand<'a, S> {
    phantom: &'a PhantomData<S>,
}

impl<'a, S> Default for ExitCommand<'a, S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, S> ExitCommand<'a, S> {
    /// Creates a new ExitCommand.
    pub fn new() -> ExitCommand<'a, S> {
        ExitCommand {
            phantom: &PhantomData,
        }
    }
}

impl<'a, S> BaseCommand for ExitCommand<'a, S> {
    type State = Shell<'a, S>;

    fn name(&self) -> &str {
        "exit"
    }

    fn validate_args(&self, args: &[String]) -> Result<()> {
        if !args.is_empty() {
            return Err(ShiError::ExtraArgs { got: args.to_vec() });
        }

        Ok(())
    }

    fn execute(&self, shell: &mut Shell<S>, _: &[String]) -> Result<String> {
        shell.terminate = true;
        Ok(String::from("bye"))
    }
}
