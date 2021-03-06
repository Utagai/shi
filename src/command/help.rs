use std::marker::PhantomData;

use anyhow::{bail, Result};

use super::BaseCommand;
use crate::shell::Shell;

#[derive(Debug)]
/// HelpCommand is a command for printing out a listing of all available commands and builtins.
///
/// It displays two separated sections, one for custom commands and one for builtins.
/// It assumes that all commands it prints have meaningful implementations of Help(), as it
/// includes it in the output.
pub struct HelpCommand<'a, S> {
    // TODO: Not sure if we need this crap.
    phantom: &'a PhantomData<S>,
}

impl<'a, S> HelpCommand<'a, S> {
    /// Creates a new HelpCommand.
    pub fn new() -> HelpCommand<'a, S> {
        HelpCommand {
            phantom: &PhantomData,
        }
    }
}

impl<'a, S> BaseCommand for HelpCommand<'a, S> {
    type State = Shell<'a, S>;

    fn name(&self) -> &str {
        "help"
    }

    fn validate_args(&self, args: &Vec<String>) -> Result<()> {
        if args.len() != 0 {
            // TODO: We may want to make this actually take arguments, like a command name or
            // command name path.
            bail!("help takes no arguments")
        }

        Ok(())
    }

    fn execute(&self, shell: &mut Shell<S>, _: &Vec<String>) -> Result<String> {
        // We expect there to be one line per command, +2 commands for headers of the two sections.
        let mut help_lines: Vec<String> =
            Vec::with_capacity(shell.cmds.borrow().len() + shell.builtins.len() + 2);
        help_lines.push(String::from("Normal commands:"));
        for cmd in shell.cmds.borrow().iter() {
            help_lines.push(format!("\t'{}' - {}", cmd.name(), cmd.help()));
        }

        help_lines.push(String::from("Built-in commands:"));
        for builtin in shell.builtins.iter() {
            help_lines.push(format!("\t'{}' - {}", builtin.name(), builtin.help()))
        }

        Ok(help_lines.join("\n"))
    }
}
