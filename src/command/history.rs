use std::marker::PhantomData;

use super::BaseCommand;
use crate::error::ShiError;
use crate::shell::Shell;
use crate::Result;

#[derive(Debug)]
/// HistoryCommand emits a listing of the command history.
///
/// What this command produces may only be the commands executed in the current session, or it may
/// also include prior sessions. This is dependent on how the containing Shell was configured.
///
/// Repeated, subsequent command invocations are a single entry in the history.
pub struct HistoryCommand<'a, S> {
    _phantom: &'a PhantomData<S>,
}

impl<'a, S> Default for HistoryCommand<'a, S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, S> HistoryCommand<'a, S> {
    /// Creates a new HistoryCommand.
    pub fn new() -> HistoryCommand<'a, S> {
        HistoryCommand {
            _phantom: &PhantomData,
        }
    }
}

impl<'a, S> BaseCommand for HistoryCommand<'a, S> {
    type State = Shell<'a, S>;

    fn name(&self) -> &str {
        "history"
    }

    fn validate_args(&self, args: &[String]) -> Result<()> {
        if !args.is_empty() {
            // TODO: We will probably want to take an optional flag for searching.
            // TODO: Maybe an optional flag for num items.
            return Err(ShiError::ExtraArgs { got: args.to_vec() });
        }

        Ok(())
    }

    fn execute(&self, shell: &mut Shell<S>, _: &[String]) -> Result<String> {
        // A bit of a mouthful. We grab the underlying history of the shell, collect its elements
        // as strings in a vector, then join them with a newline + tab.

        let history = shell.rl.history();

        let history_elements = {
            let mut mut_history_elements = vec![];

            for i in 0..history.len() {
                if let Some(elem) = history.get(i, rustyline::history::SearchDirection::Forward)? {
                    mut_history_elements.push(elem.entry.to_string());
                }
            }

            mut_history_elements
        };

        let history_output = history_elements.join("\n\t");

        // Add an extra tab because the first line won't have the join separator attached, and will
        // therefore only have the \n from the print.
        Ok(format!("\t{}", history_output))
    }

    fn help(&self) -> String {
        String::from("Prints the history of commands")
    }
}
