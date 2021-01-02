use anyhow::Result;

pub mod echo;
pub mod exit;
pub mod help;
pub mod helptree;
pub mod history;

pub mod example {
    pub use super::echo::EchoCommand;
}

pub(crate) mod builtin {
    pub use super::exit::ExitCommand;
    pub use super::help::HelpCommand;
    pub use super::helptree::HelpTreeCommand;
    pub use super::history::HistoryCommand;
}

pub mod parent;
pub use parent::ParentCommand;

pub mod basic;
pub use basic::BasicCommand;

pub enum Command<'a, S> {
    // TODO: This should be called Leaf.
    Leaf(Box<dyn BaseCommand<State = S> + 'a>),
    // TODO: Do we want to make Parent commands a trait?
    Parent(ParentCommand<'a, S>),
}

impl<'a, S> Command<'a, S> {
    pub fn new_leaf<C>(child_cmd: C) -> Self
    where
        C: BaseCommand<State = S> + 'a,
    {
        Self::Leaf(Box::new(child_cmd))
    }

    pub fn new_parent(name: &'a str, sub_cmds: Vec<Command<'a, S>>) -> Self {
        Self::Parent(ParentCommand::new(name, sub_cmds))
    }
}

impl<'a, S> BaseCommand for Command<'a, S> {
    type State = S;

    fn name(&self) -> &str {
        match self {
            Self::Leaf(cmd) => cmd.name(),
            Self::Parent(parent_cmd) => parent_cmd.name(),
        }
    }

    fn validate_args(&self, args: &Vec<String>) -> Result<()> {
        match self {
            Self::Leaf(cmd) => cmd.validate_args(args),
            Self::Parent(parent_cmd) => parent_cmd.validate_args(args),
        }
    }

    fn execute(&self, state: &mut Self::State, args: &Vec<String>) -> Result<String> {
        match self {
            Self::Leaf(cmd) => cmd.execute(state, args),
            Self::Parent(parent_cmd) => parent_cmd.execute(state, args),
        }
    }

    fn help(&self) -> String {
        match self {
            Self::Leaf(cmd) => cmd.help(),
            Self::Parent(parent_cmd) => parent_cmd.help(),
        }
    }
}

pub trait BaseCommand {
    type State;

    fn name(&self) -> &str;
    // TODO: This may be better removed and implied to implementors to include in execute()'s body.
    fn validate_args(&self, args: &Vec<String>) -> Result<()>;
    // TODO: Execute should probably be returning something better than a Result<String>.
    fn execute(&self, state: &mut Self::State, args: &Vec<String>) -> Result<String>;
    fn help(&self) -> String {
        // TODO(may): Need to flesh this out more.
        // Likely, we should return a dedicated Help object that can be formatted.
        format!("'{}'", self.name())
    }
}
