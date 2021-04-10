//! A module for the commands portion of shi.
//!
//! This module includes all command-related functionality and interfaces for using shi.

use crate::Result;

// TODO: We should be re-exporting these _from_ the command module. They should be submodules
// underneath the command module.
pub mod echo;
pub mod exit;
pub mod help;
pub mod helptree;
pub mod history;

pub use echo::*;
pub use exit::*;
pub use help::*;
pub use helptree::*;
pub use history::*;

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

/// Command represents all and any command that should exist in shi. It represents a clear
/// bifurcation: a command is either a `Leaf` or a `Parent` command.
///
/// This refers to tree terminology; a command either has no children (subcommands) or it does.
/// Thus, it is either a Leaf command or a `Parent` command, respectively.
pub enum Command<'a, S> {
    /// A command that has no sub commands. Conforms to `BaseCommand`. Executes, unlike `Parent`.
    Leaf(Box<dyn BaseCommand<State = S> + 'a>),
    // TODO: Do we want to make Parent commands a trait?
    /// A command that has sub commands. A `ParentCommand` does not execute.
    Parent(ParentCommand<'a, S>),
}

impl<'a, S> Command<'a, S> {
    // TODO: We should make this more ergonomic to use... Perhaps add `new_basic_leaf()`? At the
    // very least, we should shorten this to `leaf()`?
    /// Creates a new `Leaf` `Command` from the given command.
    pub fn new_leaf<C>(child_cmd: C) -> Self
    where
        C: BaseCommand<State = S> + 'a,
    {
        Self::Leaf(Box::new(child_cmd))
    }

    /// Creates a new `Parent` `Command` from the given vector of sub commands.
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

    fn validate_args(&self, args: &[String]) -> Result<()> {
        match self {
            Self::Leaf(cmd) => cmd.validate_args(args),
            Self::Parent(parent_cmd) => parent_cmd.validate_args(args),
        }
    }

    fn execute(&self, state: &mut Self::State, args: &[String]) -> Result<String> {
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

/// BaseCommand is the lower-level command trait. It covers many of the behaviors one would expect
/// from a shell command, e.g., a name (`name()`) or execution (`execute()`).
///
/// It is generic over a `State`, `S`, expected to be bound to its containing `Shell`.
///
/// As it may aid in understanding: builtins are in fact `BaseCommand`'s, where `State = Shell<T>`.
pub trait BaseCommand {
    /// The State of the command. Expected to be bound to a containing `Shell`.
    type State;

    /// Returns the name of the command. This is equivalent to how the command would be invoked.
    fn name(&self) -> &str;

    // TODO: This may be better removed and implied to implementors to include in execute()'s body.
    /// Validates the given arguments, returning a `Result<()>` indicating the result of
    /// validation.
    ///
    /// # Arguments
    /// `args` - The arguments to validate.
    fn validate_args(&self, args: &[String]) -> Result<()>;

    // TODO: Execute should probably be returning something better than a Result<String>.
    /// Executes the command.
    ///
    /// # Arguments
    /// `state` - The state to execute with.
    /// `args` - The arguments to the command invocation.
    ///
    /// # Returns
    /// `Result<String>` - The result of the execution of this command. If successful, returns a
    /// String that represents the output of the command.
    fn execute(&self, state: &mut Self::State, args: &[String]) -> Result<String>;

    /// Returns a String representing the help text of this command.
    ///
    /// Expected to be relatively brief.
    fn help(&self) -> String {
        // TODO(may): Need to flesh this out more.
        // Likely, we should return a dedicated Help object that can be formatted.
        format!("'{}'", self.name())
    }
}
