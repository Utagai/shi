use anyhow::{bail, Result};

use super::{BaseCommand, Command};
use crate::command_set::CommandSet;

pub struct ParentCommand<'a, S> {
    name: &'a str,
    sub_cmds: CommandSet<'a, S>,
}

impl<'a, S> ParentCommand<'a, S> {
    pub fn new(name: &'a str, sub_cmds: Vec<Command<'a, S>>) -> ParentCommand<'a, S> {
        let mut command_set = CommandSet::new();
        for sub_cmd in sub_cmds {
            command_set.add(sub_cmd);
        }
        ParentCommand {
            name,
            sub_cmds: command_set,
        }
    }

    fn get_sub_cmd_for_args(&self, args: &Vec<String>) -> Result<&Box<Command<S>>> {
        let first_arg = match args.get(0) {
            Some(arg) => arg,
            None => bail!(
                "expected one of {:?}, got nothing",
                self.sub_commands()
                    .iter()
                    .map(|cmd| cmd.name())
                    .collect::<Vec<&str>>()
            ),
        };

        match self.sub_cmds.get(first_arg) {
            Some(cmd) => Ok(cmd),
            None => bail!(
                "expected one of {:?}, got {}",
                self.sub_commands()
                    .iter()
                    .map(|cmd| cmd.name())
                    .collect::<Vec<&str>>(),
                first_arg
            ),
        }
    }

    pub fn sub_commands(&self) -> &CommandSet<S> {
        &self.sub_cmds
    }
}

impl<'a, S> BaseCommand for ParentCommand<'a, S> {
    type State = S;

    fn name(&self) -> &str {
        self.name
    }

    fn validate_args(&self, args: &Vec<String>) -> Result<()> {
        if self.sub_commands().len() == 0 && args.len() == 0 {
            return Ok(());
        } else if self.sub_commands().len() == 0 {
            bail!("no sub commands expected, but got {:?}", args)
        }

        // This will error if we do not find the command, but we don't actually care about the
        // particular command we find here.
        self.get_sub_cmd_for_args(args)?;

        Ok(())
    }

    fn execute(&self, state: &mut S, args: &Vec<String>) -> Result<String> {
        let sub_cmd = self.get_sub_cmd_for_args(args)?;

        sub_cmd.execute(state, &args[1..].to_vec())
    }
}
