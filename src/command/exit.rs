use std::marker::PhantomData;

use anyhow::{bail, Result};

use super::Command;
use crate::shell::Shell;

#[derive(Debug)]
pub struct ExitCommand<'a, S> {
    phantom: &'a PhantomData<S>,
}

impl<'a, S> ExitCommand<'a, S> {
    pub fn new() -> ExitCommand<'a, S> {
        ExitCommand {
            phantom: &PhantomData,
        }
    }
}

impl<'a, S> Command for ExitCommand<'a, S> {
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
