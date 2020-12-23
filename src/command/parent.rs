use std::collections::HashMap;

use anyhow::{bail, Result};

use super::Command;

pub struct ParentCommand<'a, S> {
    name: &'a str,
    sub_cmds: HashMap<String, Box<dyn Command<State = S>>>,
}

impl<'a, S> ParentCommand<'a, S> {
    pub fn new(name: &'a str, sub_cmds: Vec<Box<dyn Command<State = S>>>) -> ParentCommand<'a, S> {
        let mut hm = HashMap::new();
        for sub_cmd in sub_cmds {
            hm.insert(sub_cmd.name().to_owned(), sub_cmd);
        }
        ParentCommand { name, sub_cmds: hm }
    }

    fn get_sub_cmd_for_args(&self, args: &Vec<String>) -> Result<&Box<dyn Command<State = S>>> {
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
}

impl<'a, S> Command for ParentCommand<'a, S> {
    type State = S;

    fn name(&self) -> &str {
        self.name
    }

    fn sub_commands(&self) -> Vec<&dyn Command<State = S>> {
        self.sub_cmds.values().map(|v| v.as_ref()).collect()
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
