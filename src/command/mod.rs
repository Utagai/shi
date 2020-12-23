use anyhow::Result;

pub mod echo;
pub mod exit;
pub mod help;
pub mod history;

pub mod example {
    pub use super::echo::EchoCommand;
}

pub(crate) mod builtin {
    pub use super::exit::ExitCommand;
    pub use super::help::HelpCommand;
    pub use super::history::HistoryCommand;
}

pub mod parent;
pub use parent::ParentCommand;

pub mod basic;
pub use basic::BasicCommand;

pub trait Command {
    type State;

    fn name(&self) -> &str;
    // TODO: This may be better removed and implied to implementors to include in execute()'s body.
    fn validate_args(&self, args: &Vec<String>) -> Result<()>;
    // TODO: This probably shouldn't have a default impl.
    fn sub_commands(&self) -> Vec<&dyn Command<State = Self::State>> {
        Vec::new()
    }
    // TODO: Execute should probably be returning something better than a Result<String>.
    fn execute(&self, state: &mut Self::State, args: &Vec<String>) -> Result<String>;
    fn help(&self) -> String {
        // TODO(may): Need to flesh this out more.
        // Likely, we should return a dedicated Help object that can be formatted.
        format!("'{}'", self.name())
    }
}
