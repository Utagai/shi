use std::marker::PhantomData;

use anyhow::{bail, Result};

use super::BaseCommand;
use crate::shell::Shell;

// TODO: This should be private.
#[derive(Debug)]
/// ExitCommand is a command that triggers a termination of the shell.
pub struct ExitCommand<'a, S> {
    phantom: &'a PhantomData<S>,
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

    fn validate_args(&self, args: &Vec<String>) -> Result<()> {
        if args.len() != 0 {
            bail!("exit takes no arguments")
        }

        Ok(())
    }

    fn execute(&self, shell: &mut Shell<S>, _: &Vec<String>) -> Result<String> {
        shell.terminate = true;
        Ok(String::from("bye"))
    }
}
