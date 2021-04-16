use std::rc::Rc;

use super::BaseCommand;
use crate::Result;

/// A BasicCommand is a very simple command type. It has a name, and it has a closure that it
/// executes when it is invoked. The closure takes a state, as determined by its containing shell,
/// and a vector of String arguments.
pub struct BasicCommand<'a, S> {
    name: &'a str,
    help: &'a str,
    exec: Rc<dyn Fn(&mut S, &[String]) -> Result<String>>,
}

impl<'a, S> BasicCommand<'a, S> {
    /// Creates a new BasicCommand with the given name and closure.
    ///
    /// # Arguments
    /// * `name` - The name of the command. This is how users will execute the command.
    /// * `exec` - The closure that will be executed when this command is invoked.
    pub fn new<F>(name: &'a str, exec: F) -> BasicCommand<'a, S>
    where
        F: Fn(&mut S, &[String]) -> Result<String> + 'static,
    {
        BasicCommand {
            name,
            help: "",
            exec: Rc::new(exec),
        }
    }

    /// Creates a new BasicCommand with the given name, closure and help message.
    ///
    /// # Arguments
    /// * `name` - The name of the command. This is how users will execute the command.
    /// * `exec` - The closure that will be executed when this command is invoked.
    /// * `help` - The help message to use.
    pub fn new_with_help<F>(name: &'a str, help: &'a str, exec: F) -> BasicCommand<'a, S>
    where
        F: Fn(&mut S, &[String]) -> Result<String> + 'static,
    {
        BasicCommand {
            name,
            help,
            exec: Rc::new(exec),
        }
    }
}

impl<'a, S> BaseCommand for BasicCommand<'a, S> {
    type State = S;

    fn name(&self) -> &str {
        self.name
    }

    fn validate_args(&self, _: &[String]) -> Result<()> {
        Ok(())
    }

    fn execute(&self, state: &mut S, args: &[String]) -> Result<String> {
        (self.exec)(state, args)
    }

    fn help(&self) -> String {
        self.help.to_string()
    }
}
