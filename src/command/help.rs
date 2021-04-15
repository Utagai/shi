use std::marker::PhantomData;

use crate::command::{BaseCommand, Command};
use crate::command_set::CommandSet;
use crate::error::ShiError;
use crate::parser::CommandType;
use crate::shell::Shell;
use crate::Result;

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

impl<'a, S> Default for HelpCommand<'a, S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, S> HelpCommand<'a, S> {
    /// Creates a new HelpCommand.
    pub fn new() -> HelpCommand<'a, S> {
        HelpCommand {
            phantom: &PhantomData,
        }
    }

    fn execute_no_args(&self, shell: &mut Shell<S>) -> Result<String> {
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

    fn help_breakdown<T>(&self, cmd_path: Vec<&str>, cmds: &CommandSet<T>) -> Result<String> {
        let mut indent = 0;
        let mut lines = Vec::new();
        let mut current_cmds = cmds;
        for segment in cmd_path {
            match current_cmds.get(segment) {
                Some(cmd) => {
                    let cmd_name = cmd.name();
                    let help_msg = cmd.help();
                    lines.push(format!(
                        "{}└─ {} - {}",
                        "   ".repeat(indent), // Use two spaces since we have 2 pipe-characters & a space.
                        cmd_name,
                        help_msg
                    ));
                    match &**cmd {
                        Command::Parent(parent) => current_cmds = parent.sub_commands(),
                        Command::Leaf(_) => break,
                    };
                }
                None => {
                    return Err(ShiError::UnrecognizedCommand {
                        got: segment.to_string(),
                    })
                }
            }
            indent += 1;
        }

        Ok(lines.join("\n"))
    }

    fn execute_with_args(&self, shell: &mut Shell<S>, args: &[String]) -> Result<String> {
        let invocation = args.join(" ");
        let outcome = shell.parse(&invocation);

        // Now that we've parsed the args as a command invocation, we can offer a detailed help
        // break down for the command path:
        return match outcome.cmd_type {
            CommandType::Custom => self.help_breakdown(outcome.cmd_path, &shell.cmds.borrow()),
            CommandType::Builtin => self.help_breakdown(outcome.cmd_path, &shell.builtins),
            CommandType::Unknown => Err(outcome
                .error()
                .expect("unknown command type, but could not produce error")),
        };
    }
}

impl<'a, S> BaseCommand for HelpCommand<'a, S> {
    type State = Shell<'a, S>;

    fn name(&self) -> &str {
        "help"
    }

    fn validate_args(&self, _: &[String]) -> Result<()> {
        Ok(())
    }

    fn execute(&self, shell: &mut Shell<S>, args: &[String]) -> Result<String> {
        if args.is_empty() {
            self.execute_no_args(shell)
        } else {
            self.execute_with_args(shell, args)
        }
    }
}
