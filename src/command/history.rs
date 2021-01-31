use std::marker::PhantomData;

use anyhow::{bail, Result};

use super::BaseCommand;
use crate::shell::Shell;

#[derive(Debug)]
/// HistoryCommand emits a listing of the command history.
///
/// What this command produces may only be the commands executed in the current session, or it may
/// also include prior sessions. This is dependent on how the containing Shell was configured.
///
/// Repeated, subsequent command invocations are a single entry in the history.
pub struct HistoryCommand<'a, S> {
    phantom: &'a PhantomData<S>,
}

impl<'a, S> HistoryCommand<'a, S> {
    /// Creates a new HistoryCommand.
    pub fn new() -> HistoryCommand<'a, S> {
        HistoryCommand {
            phantom: &PhantomData,
        }
    }
}

impl<'a, S> BaseCommand for HistoryCommand<'a, S> {
    type State = Shell<'a, S>;

    fn name(&self) -> &str {
        "history"
    }

    fn validate_args(&self, args: &Vec<String>) -> Result<()> {
        if args.len() != 0 {
            // TODO: We will probably want to take an optional flag for searching.
            // TODO: Maybe an optional flag for num items.
            bail!("history takes no arguments")
        }

        Ok(())
    }

    fn execute(&self, shell: &mut Shell<S>, _: &Vec<String>) -> Result<String> {
        // A bit of a mouthful. We grab the underlying history of the shell, iterate it, map the
        // elements to strings, then collection them into a vector of Strings before we join them
        // with a newline + tab.
        let history_output = shell
            .rl
            .history()
            .iter()
            .map(|h| h.to_string())
            .collect::<Vec<String>>()
            .join("\n\t");

        // Add an extra tab because the first line won't have the join separator attached, and will
        // therefore only have the \n from the print.
        Ok(format!("\t{}", history_output))
    }
}
