use std::rc::Rc;

use anyhow::Result;

use super::BaseCommand;

pub struct BasicCommand<'a, S> {
    name: &'a str,
    exec: Rc<dyn Fn(&mut S, &Vec<String>) -> Result<String>>,
}

impl<'a, S> BasicCommand<'a, S> {
    // TODO: We may actually prefer to make this return Box<> to make our API less verbose.
    pub fn new<F>(name: &'a str, exec: F) -> BasicCommand<'a, S>
    where
        F: Fn(&mut S, &Vec<String>) -> Result<String> + 'static,
    {
        BasicCommand {
            name,
            exec: Rc::new(exec),
        }
    }
}

impl<'a, S> BaseCommand for BasicCommand<'a, S> {
    type State = S;

    fn name(&self) -> &str {
        self.name
    }

    fn validate_args(&self, _: &Vec<String>) -> Result<()> {
        Ok(())
    }

    fn execute(&self, state: &mut S, args: &Vec<String>) -> Result<String> {
        (self.exec)(state, args)
    }
}
