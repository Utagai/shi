use std::fmt::Debug;
use std::marker::PhantomData;

use super::BaseCommand;
use crate::error::ShiError;
use crate::Result;

#[derive(Debug)]
/// EchoCommand is likely not very useful. It is here mostly for letting users scaffold their
/// command hierarchies without needing to actually implement or come up with the actual commands
/// they'd like to have eventually.
///
/// As the name suggests, this command simply echos back whatever arguments it receives.
pub struct EchoCommand<S> {
    phantom: PhantomData<S>,
}

impl<S> Default for EchoCommand<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> EchoCommand<S> {
    /// Creates a new EchoCommand.
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

    fn validate_args(&self, args: &[String]) -> Result<()> {
        if args.is_empty() {
            return Err(ShiError::NoArgs);
        }

        Ok(())
    }

    fn execute(&self, _: &mut S, args: &[String]) -> Result<String> {
        // TODO: We should probably not expose the data type here, and instead return a joined
        // string.
        Ok(format!("ECHO: '{:?}'", args))
    }
}
