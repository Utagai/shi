use std::marker::PhantomData;

use anyhow::{bail, Result};

use super::Command;
use crate::shell::Shell;

#[derive(Debug)]
pub struct HistoryCommand<'a, S> {
    phantom: &'a PhantomData<S>,
}

impl<'a, S> HistoryCommand<'a, S> {
    pub fn new() -> HistoryCommand<'a, S> {
        HistoryCommand {
            phantom: &PhantomData,
        }
    }
}

impl<'a, S> Command for HistoryCommand<'a, S> {
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
