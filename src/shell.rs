//! A module for the shell portion of shi.
//!
//! This module includes all shell-related functionality and interfaces for using shi.
//! Namely, it exposes the `Shell` struct, which is the heart of shi. It makes use of `Command`'s
//! to create a shell interface.

use std::cell::RefCell;
use std::rc::Rc;

use rustyline::error::ReadlineError;

use crate::command::{
    builtin::{ExitCommand, HelpCommand, HelpTreeCommand, HistoryCommand},
    BaseCommand, Command,
};
use crate::command_set::CommandSet;
use crate::error::ShiError;
use crate::parser::{CommandType, Outcome, Parser};
use crate::readline::Readline;
use crate::Result;

/// The shell.
///
/// This gives the shell interface for shi. It is constructed and registered with commands.
/// Execution is done through a run-loop of input/output.
pub struct Shell<'a, S> {
    prompt: &'a str,
    // TODO: We likely should NOT be exporting these, even within the crate. Instead, we should add
    // public getters, perhaps?
    // We need Rc<RefCell> because:
    // * We need Rc because Shell is a self-referencing struct, in that the cmds field is referenced
    // by rl, so we need to allocate this at construction time (at runtime, on the heap) and share
    // references. This calls for Rc.
    // * Rc by itself is not mutable however, but we support adding commands to cmds. So we need
    // RefCell.
    pub(crate) cmds: Rc<RefCell<CommandSet<'a, S>>>,
    pub(crate) builtins: Rc<CommandSet<'a, Self>>,
    pub(crate) rl: Readline<'a, S>,
    parser: Parser,
    history_file: Option<&'a str>,
    state: S,
    pub(crate) terminate: bool,
}

impl<'a> Shell<'a, ()> {
    /// Constructs a new shell with the given prompt, and no state.
    ///
    /// # Arguments
    /// `prompt` - The prompt to display to the user.
    pub fn new(prompt: &'a str) -> Shell<'a, ()> {
        let cmds = Rc::new(RefCell::new(CommandSet::new()));
        let builtins = Rc::new(Shell::build_builtins());
        Shell {
            prompt,
            rl: Readline::new(Parser::new(), cmds.clone(), builtins.clone()),
            parser: Parser::new(),
            cmds,
            builtins,
            history_file: None,
            state: (),
            terminate: false,
        }
    }
}

impl<'a, S> Shell<'a, S> {
    /// Constructs the various builtin commands and returns a `CommandSet` of them.
    fn build_builtins() -> CommandSet<'a, Shell<'a, S>>
    where
        S: 'a,
    {
        let mut builtins: CommandSet<'a, Shell<'a, S>> = CommandSet::new();
        builtins.add(Command::new_leaf(HelpCommand::new()));
        builtins.add(Command::new_leaf(HelpTreeCommand::new()));
        builtins.add(Command::new_leaf(ExitCommand::new()));
        builtins.add(Command::new_leaf(HistoryCommand::new()));

        builtins
    }

    /// Constructs a new shell, with the given prompt & state.
    ///
    /// # Arguments
    /// `prompt` - The prompt to display to the user.
    /// `state` - The state that the `Shell` should persist across command invocations.
    pub fn new_with_state(prompt: &'a str, state: S) -> Shell<'a, S>
    where
        S: 'a,
    {
        let cmds = Rc::new(RefCell::new(CommandSet::new()));
        let builtins = Rc::new(Shell::build_builtins());
        Shell {
            prompt,
            rl: Readline::new(Parser::new(), cmds.clone(), builtins.clone()),
            parser: Parser::new(),
            cmds,
            builtins,
            history_file: None,
            state,
            terminate: false,
        }
    }

    /// Registers the given command under this `Shell`.
    ///
    /// # Arguments
    /// `cmd` - The command to register.
    pub fn register(&mut self, cmd: Command<'a, S>) -> Result<()> {
        if self.cmds.borrow().contains(cmd.name()) {
            return Err(ShiError::AlreadyRegistered {
                cmd: cmd.name().to_string(),
            });
        }

        self.cmds.borrow_mut().add(cmd);

        Ok(())
    }

    // TODO: Should we be doing something similar to `rustyline` where we take `P: Path` or
    // whatever it is?
    /// Sets the history file & loads the history from it, if it exists already.
    ///
    /// This is necessary to call if one wishes for their command history to persist across
    /// sessions.
    ///
    /// # Arguments
    /// `history-file` - The path to the history file.
    pub fn set_and_load_history_file(&mut self, history_file: &'a str) -> Result<()> {
        self.rl.load_history(history_file)?;
        self.history_file = Some(history_file);
        Ok(())
    }

    /// Saves the history.
    ///
    /// This is effectively a no-op if no history file has been set.
    ///
    /// This must also be called to actually persist the current session's history. It is necessary
    /// to persist the history if one wishes to see it in future sessions.
    pub fn save_history(&mut self) -> Result<()> {
        if let Some(history_file) = self.history_file {
            self.rl.save_history(history_file)?;
        }
        Ok(())
    }

    pub(crate) fn parse<'b>(&mut self, line: &'b str) -> Outcome<'b> {
        self.parser.parse(line, &self.cmds.borrow(), &self.builtins)
    }

    /// Eval executes a single loop of the shell's run-loop.
    ///
    /// In other words, it takes a single input line and executes on it; `run()` is a loop over
    /// `eval()`.
    ///
    /// # Arguments
    /// `line` - The line to evaluate.
    pub fn eval(&mut self, line: &str) -> Result<String> {
        self.rl.add_history_entry(line);
        let outcome = self.parse(line);

        if !outcome.complete {
            return Err(outcome
                .error()
                .expect("incomplete parse, but failed to produce an error")); // This should never happen.
        }

        match outcome.cmd_type {
            CommandType::Custom => {
                // TODO: This recursive walking through the arguments when we pass this into the
                // ParentCommand is redundant, since we already did that work when we parsed
                // things. We should avoid doing this.
                if let Some(base_cmd_name) = outcome.cmd_path.first() {
                    if let Some(base_cmd) = self.cmds.borrow().get(base_cmd_name) {
                        let args: Vec<String> =
                            line.split(' ').skip(1).map(|s| s.to_string()).collect();
                        base_cmd.validate_args(&args)?;
                        return base_cmd.execute(&mut self.state, &args);
                    }
                }

                Err(ShiError::UnrecognizedCommand {
                    got: line.to_string(),
                })
            }
            CommandType::Builtin => {
                if let Some(base_cmd_name) = outcome.cmd_path.first() {
                    if let Some(base_cmd) = self.builtins.clone().get(base_cmd_name) {
                        let args: Vec<String> =
                            line.split(' ').skip(1).map(|s| s.to_string()).collect();
                        base_cmd.validate_args(&args)?;
                        return base_cmd.execute(self, &args);
                    }
                }

                Err(ShiError::UnrecognizedCommand {
                    got: line.to_string(),
                })
            }
            CommandType::Unknown => Err(outcome
                .error()
                .expect("parsed an Unknown, but failed to produce an error")), // This should never happen.
        }
    }

    /// Executes the shell's run-loop but only once.
    ///
    /// This relies on the caller to call it repeatedly to keep the shell operational.
    ///
    /// Note that this blocks on reading a line from the user.
    ///
    /// Returns
    ///     - Ok(true), update can be called again to continue service
    ///     - Ok(false), service terminated. update should not be called again.
    ///     - Err on any unhandled errors that should terminate the shell. This should result in no
    ///       longer calling update by the caller.
    ///
    pub fn update(&mut self) -> Result<bool> {
        if self.terminate {
            return Ok(false);
        }

        let input = self.rl.readline(self.prompt);

        match input {
            Ok(line) => match self.eval(&line) {
                Ok(output) => println!("{}", output),
                Err(err) => println!("Error: {}", err),
            },
            Err(ReadlineError::Interrupted) => {
                println!("-> CTRL+C; bye.");
                return Ok(false);
            }
            Err(ReadlineError::Eof) => {
                println!("-> CTRL+D; bye.");
                return Ok(false);
            }
            Err(err) => {
                println!("Error: {:?}", err);
                return Err(ShiError::general(err.to_string()));
            }
        }

        Ok(true)
    }

    /// The caller calls this after update routines finish servicing. This performs final computations
    /// before exit.
    pub fn finish(&mut self) -> Result<()> {
        self.save_history()?;

        Ok(())
    }

    /// Executes the shell's run-loop.
    ///
    /// This will run indefinitely until the user exits, otherwise terminates the shell or
    /// process or the shell encounters an error and stops.
    ///
    /// Note that invalid command invocations, e.g., nonexistent commands, are not considered fatal
    /// errors and do _not_ cause a return from this method.
    pub fn run(&mut self) -> Result<()> {
        loop {
            let update_again = self.update()?;

            if !update_again {
                break;
            }
        }

        self.finish()?;

        Ok(())
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    use crate::Result;
    use crate::{cmd, parent};

    use pretty_assertions::assert_eq;

    // TODO: Replace or add more tests that trigger the full codepath of the shell.
    #[test]
    fn issue6() -> Result<()> {
        let mut shell = Shell::new("| ");
        shell.register(parent!(
            "server",
            cmd!("listen", "Start listening on the given port", |_, args| {
                Ok(format!("start: {:?}", args))
            }),
            cmd!("unlisten", "stop listening", |_, args| {
                Ok(format!("stop: {:?}", args))
            })
        ))?;

        let output = shell.eval("server listen")?;
        assert_eq!(output, "start: []");

        let output = shell.eval("server listen foo")?;
        assert_eq!(output, "start: [\"foo\"]");

        let output = shell.eval("server unlisten")?;
        assert_eq!(output, "stop: []");

        let output = shell.eval("server unlisten foo")?;
        assert_eq!(output, "stop: [\"foo\"]");

        Ok(())
    }
}
