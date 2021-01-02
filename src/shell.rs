use std::rc::Rc;

// TODO: We should probably be using thiserror if this is going to be factored out into a library.
use anyhow::{bail, Result};
use rustyline::error::ReadlineError;

use crate::command::{
    builtin::{ExitCommand, HelpCommand, HelpTreeCommand, HistoryCommand},
    BaseCommand, Command,
};
use crate::command_set::CommandSet;
use crate::parser::Parser;
use crate::readline::Readline;

pub struct Shell<'a, S> {
    prompt: &'a str,
    pub(crate) cmds: CommandSet<'a, S>,
    pub(crate) builtins: Rc<CommandSet<'a, Self>>,
    pub(crate) rl: Readline,
    pub(crate) parser: Parser,
    history_file: Option<&'a str>,
    state: S,
    pub(crate) terminate: bool,
}

impl<'a> Shell<'a, ()> {
    pub fn new(prompt: &'a str) -> Shell<()> {
        Shell {
            prompt,
            rl: Readline::new(),
            parser: Parser::new(),
            cmds: CommandSet::new(),
            builtins: Rc::new(Shell::build_builtins()),
            history_file: None,
            state: (),
            terminate: false,
        }
    }
}

impl<'a, S> Shell<'a, S> {
    fn build_builtins() -> CommandSet<'a, Shell<'a, S>>
    where
        S: 'a,
    {
        let mut builtins: CommandSet<'a, Shell<'a, S>> = CommandSet::new();
        builtins.add(Command::new_leaf(HelpCommand::new()));
        builtins.add(Command::new_leaf(HelpTreeCommand::new()));
        builtins.add(Command::new_leaf(ExitCommand::new()));
        builtins.add(Command::new_leaf(HistoryCommand::new()));

        return builtins;
    }

    pub fn new_with_state(prompt: &'a str, state: S) -> Shell<S>
    where
        S: 'a,
    {
        Shell {
            prompt,
            rl: Readline::new(),
            parser: Parser::new(),
            cmds: CommandSet::new(),
            builtins: Rc::new(Shell::build_builtins()),
            history_file: None,
            state,
            terminate: false,
        }
    }

    pub fn register(&mut self, cmd: Command<'a, S>) -> Result<()> {
        if self.cmds.contains(cmd.name()) {
            bail!("command '{}' already registered", cmd.name())
        }

        self.cmds.add(cmd);

        Ok(())
    }

    pub fn set_history_file(&mut self, history_file: &'a str) -> Result<()> {
        self.rl.load_history(history_file)?;
        self.history_file = Some(history_file);
        Ok(())
    }

    pub fn save_history(&mut self) -> Result<()> {
        if let Some(history_file) = self.history_file {
            self.rl.save_history(history_file)?;
        }
        Ok(())
    }

    pub fn eval(&mut self, line: &str) -> Result<String> {
        self.rl.add_history_entry(line);
        let mut splits = line.split(' ');
        let potential_cmd = match splits.nth(0) {
            Some(cmd) => cmd,
            None => {
                println!("empty!");
                return Ok(String::from(""));
            }
        };
        let args: Vec<String> = splits.map(|s| s.to_owned()).collect();
        match self.cmds.get(potential_cmd) {
            Some(cmd) => {
                cmd.validate_args(&args)?;
                return cmd.execute(&mut self.state, &args);
            }
            None => {
                // Fallback to builtins. Then error if we got nothing.
                let builtins_rc = self.builtins.clone();
                match builtins_rc.get(potential_cmd) {
                    Some(builtin) => {
                        builtin.validate_args(&args)?;
                        return builtin.execute(self, &args);
                    }
                    None => println!("Unrecognized command: '{}'", potential_cmd),
                }
            }
        };
        Ok(String::from(""))
    }

    pub fn run(&mut self) -> Result<()> {
        while !self.terminate {
            let input = self.rl.readline(self.prompt);

            match input {
                Ok(line) => match self.eval(&line) {
                    Ok(output) => println!("{}", output),
                    Err(err) => {
                        let outcome = self.parser.parse(&line, &self.cmds, &self.builtins);
                        if !outcome.complete {
                            println!("{}", outcome.error_msg(Some(&format!("{:?}", err))));
                        } else {
                            println!("{:?}", err)
                        }
                    }
                },
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(ReadlineError::Eof) => {
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
