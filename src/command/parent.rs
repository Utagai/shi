use super::{BaseCommand, Command};
use crate::command_set::CommandSet;
use crate::error::ShiError;
use crate::Result;

/// ParentCommand represents a command with subcommands. It has a name, but it does not execute
/// anything itself. It dispatches to the appropriate child command, if one exists.
pub struct ParentCommand<'a, S> {
    name: &'a str,
    help: &'a str,
    sub_cmds: CommandSet<'a, S>,
}

impl<'a, S> ParentCommand<'a, S> {
    /// Creates a new ParentCommand.
    ///
    /// # Arguments
    /// `name` - The name of this command.
    /// `sub_cmds` - The subcommands or children of the `ParentCommand` to be created.
    pub fn new(name: &'a str, sub_cmds: Vec<Command<'a, S>>) -> ParentCommand<'a, S> {
        let mut command_set = CommandSet::new();
        for sub_cmd in sub_cmds {
            command_set.add(sub_cmd);
        }
        ParentCommand {
            name,
            help: "",
            sub_cmds: command_set,
        }
    }

    /// Creates a new ParentCommand with the given help message.
    ///
    /// # Arguments
    /// `name` - The name of this command.
    /// `sub_cmds` - The subcommands or children of the `ParentCommand` to be created.
    pub fn new_with_help(
        name: &'a str,
        help: &'a str,
        sub_cmds: Vec<Command<'a, S>>,
    ) -> ParentCommand<'a, S> {
        let mut command_set = CommandSet::new();
        for sub_cmd in sub_cmds {
            command_set.add(sub_cmd);
        }
        ParentCommand {
            name,
            help,
            sub_cmds: command_set,
        }
    }

    /// Retrieves the subcommand that corresponds to the arguments. The arguments passed to the
    /// ParentCommand are expected to be some non-zero length chain of subcommands, the first
    /// element of which should exist in this `ParentCommand` as a subcommand.
    ///
    /// # Arguments
    /// `args` - The arguments that this command was invoked with.
    fn get_sub_cmd_for_args(&self, args: &[String]) -> Result<&Command<S>> {
        let first_arg = match args.get(0) {
            Some(arg) => arg,
            None => return Err(ShiError::NoArgs),
        };

        match self.sub_cmds.get(first_arg) {
            Some(cmd) => Ok(cmd),
            None => {
                return Err(ShiError::InvalidSubCommand {
                    got: first_arg.to_string(),
                    expected: self
                        .sub_commands()
                        .iter()
                        .map(|cmd| cmd.name().to_string())
                        .collect::<Vec<String>>(),
                })
            }
        }
    }

    /// Returns a `CommandSet` of the child commands under this `ParentCommand`.
    pub fn sub_commands(&self) -> &CommandSet<S> {
        &self.sub_cmds
    }
}

impl<'a, S> BaseCommand for ParentCommand<'a, S> {
    type State = S;

    fn name(&self) -> &str {
        self.name
    }

    fn validate_args(&self, args: &[String]) -> Result<()> {
        if let Some(first_arg) = args.first() {
            // If args given...
            if self.sub_commands().len() == 0 {
                // But we expect no args...
                return Err(ShiError::InvalidSubCommand {
                    got: first_arg.clone(),
                    expected: args.to_vec(),
                });
            } else {
                // If we expect args...
                // This will error if we do not find the command, but we don't actually care about the
                // particular command we find here.
                self.get_sub_cmd_for_args(args)?;
            }
        } else {
            // If no args given...
            if self.sub_commands().len() != 0 {
                // But we expect args...
                return Err(ShiError::NoArgs);
            }
        }

        Ok(())
    }

    fn execute(&self, state: &mut S, args: &[String]) -> Result<String> {
        let sub_cmd = self.get_sub_cmd_for_args(args)?;

        sub_cmd.execute(state, &args[1..].to_vec())
    }

    fn help(&self) -> String {
        self.help.to_string()
    }
}
