use std::fmt::Debug;
use std::marker::PhantomData;

use anyhow::{bail, Result};

use super::BaseCommand;

#[derive(Debug)]
pub struct EchoCommand<S> {
    phantom: PhantomData<S>,
}

impl<S> EchoCommand<S> {
    pub fn new() -> EchoCommand<S> {
        EchoCommand {
            phantom: PhantomData,
        }
    }
}

impl<S> BaseCommand for EchoCommand<S> {
    type State = S;

    fn name(&self) -> &str {
        "echo"
    }

    fn validate_args(&self, args: &Vec<String>) -> Result<()> {
        if args.len() == 0 {
            bail!("'{}' requires a non-zero number of arguments!")
        }

        Ok(())
    }

    fn execute(&self, _: &mut S, args: &Vec<String>) -> Result<String> {
        Ok(format!("ECHO: '{:?}'", args))
    }
}
