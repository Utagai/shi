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
use crate::parser::Parser;
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
    pub(crate) cmds: Rc<RefCell<CommandSet<'a, S>>>,
    pub(crate) builtins: Rc<CommandSet<'a, Self>>,
    pub(crate) rl: Readline<'a, S>,
    pub(crate) parser: Parser,
    history_file: Option<&'a str>,
    state: S,
    pub(crate) terminate: bool,
}

impl<'a> Shell<'a, ()> {
    /// Constructs a new shell with the given prompt, and no state.
    ///
    /// # Arguments
    /// `prompt` - The prompt to display to the user.
    pub fn new(prompt: &'a str) -> Shell<()> {
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
    pub fn new_with_state(prompt: &'a str, state: S) -> Shell<S>
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

    /// Eval executes a single loop of the shell's run-loop.
    ///
    /// In other words, it takes a single input line and executes on it; `run()` is a loop over
    /// `eval()`.
    ///
    /// # Arguments
    /// `line` - The line to evaluate.
    pub fn eval(&mut self, line: &str) -> Result<String> {
        self.rl.add_history_entry(line);
        let mut splits = line.split(' ');
        let potential_cmd = match splits.next() {
            Some(cmd) => cmd,
            None => {
                println!("empty!");
                return Ok(String::from(""));
            }
        };
        let args: Vec<String> = splits.map(|s| s.to_owned()).collect();
        if let Some(cmd) = self.cmds.borrow().get(potential_cmd) {
            cmd.validate_args(&args)?;
            return cmd.execute(&mut self.state, &args);
        };

        // Fallback to builtins. Then error if we got nothing.
        let builtins_rc = self.builtins.clone();
        match builtins_rc.get(potential_cmd) {
            Some(builtin) => {
                builtin.validate_args(&args)?;
                builtin.execute(self, &args)
            }
            None => Err(ShiError::UnrecognizedCommand {
                got: potential_cmd.to_string(),
            }),
        }
    }

    /// Executes the shell's run-loop.
    ///
    /// This will run indefinitely until the user exits, otherwise terminates the shell or
    /// process or the shell encounters an error and stops.
    ///
    /// Note that invalid command invocations, e.g., nonexistent commands, are not considered fatal
    /// errors and do _not_ cause a return from this method.
    pub fn run(&mut self) -> Result<()> {
        while !self.terminate {
            let input = self.rl.readline(self.prompt);

            match input {
                Ok(line) => match self.eval(&line) {
                    Ok(output) => println!("{}", output),
                    Err(err) => {
                        let outcome = self
                            .parser
                            .parse(&line, &self.cmds.borrow(), &self.builtins);
                        if !outcome.complete {
                            println!("{}", outcome.error_msg());
                        } else {
                            println!("{:?}", err)
                        }
                    }
                },
                Err(ReadlineError::Interrupted) => {
                    // TODO: We should make this say 'bye' or something.
                    println!("CTRL-C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    // TODO: We should make this say 'bye' or something.
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }

        self.save_history()?;

        Ok(())
    }
}
