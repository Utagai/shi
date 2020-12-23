use std::collections::HashMap;
use std::rc::Rc;

// TODO: We should probably be using thiserror if this is going to be factored out into a library.
use anyhow::{bail, Result};
use rustyline::{error::ReadlineError, Editor};

use crate::command::{
    builtin::{ExitCommand, HelpCommand, HistoryCommand},
    Command,
};

pub struct Shell<'a, S> {
    prompt: &'a str,
    pub(crate) cmds: HashMap<String, Box<dyn Command<State = S> + 'a>>,
    pub(crate) builtins: Rc<HashMap<String, Box<dyn Command<State = Self> + 'a>>>,
    pub(crate) rl: Editor<()>,
    history_file: Option<&'a str>,
    state: S,
    pub(crate) terminate: bool,
}

impl<'a, S> Shell<'a, S> {
    fn build_builtins() -> HashMap<String, Box<dyn Command<State = Shell<'a, S>> + 'a>>
    where
        S: 'a,
    {
        // Wow so many RANGLEs.
        // Anyways, we could just make the HashMap directly, but I find this easier to add elements
        // to.
        let builtins_vec: Vec<Box<dyn Command<State = Shell<S>>>> = vec![
            Box::new(HelpCommand::new()),
            Box::new(ExitCommand::new()),
            Box::new(HistoryCommand::new()),
        ];
        let mut builtins: HashMap<String, Box<dyn Command<State = Shell<S>>>> =
            HashMap::with_capacity(builtins_vec.len());
        for builtin in builtins_vec {
            builtins.insert(builtin.name().to_owned(), builtin);
        }

        return builtins;
    }

    // TODO: Apparently, this doesn't help rustc infer the type, even though we hardcoded what the
    // type of the shell is in this case.
    pub fn new(prompt: &'a str) -> Shell<()> {
        Shell {
            prompt,
            rl: Editor::<()>::new(),
            cmds: HashMap::new(),
            builtins: Rc::new(Shell::build_builtins()),
            history_file: None,
            state: (),
            terminate: false,
        }
    }

    pub fn new_with_state(prompt: &'a str, state: S) -> Shell<S>
    where
        S: 'a,
    {
        Shell {
            prompt,
            rl: Editor::<()>::new(),
            cmds: HashMap::new(),
            builtins: Rc::new(Shell::build_builtins()),
            history_file: None,
            state,
            terminate: false,
        }
    }

    pub fn register<T>(&mut self, cmd: T) -> Result<()>
    where
        T: 'a + Command<State = S>,
    {
        if self.cmds.contains_key(cmd.name()) {
            bail!("command '{}' already registered", cmd.name())
        }

        self.cmds.insert(cmd.name().to_owned(), Box::new(cmd));

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

    pub fn run(&mut self) -> Result<()> {
        while !self.terminate {
            let input = self.rl.readline(self.prompt);

            match input {
                Ok(line) => {
                    self.rl.add_history_entry(line.as_str());
                    let mut splits = line.split(' ');
                    let potential_cmd = match splits.nth(0) {
                        Some(cmd) => cmd,
                        None => {
                            println!("empty!");
                            continue;
                        }
                    };
                    let args: Vec<String> = splits.map(|s| s.to_owned()).collect();
                    match self.cmds.get::<str>(&potential_cmd) {
                        Some(cmd) => {
                            if let Err(err) = cmd.validate_args(&args) {
                                println!("{:?}", err);
                                continue;
                            }
                            println!("{}", cmd.execute(&mut self.state, &args)?);
                        }
                        None => {
                            // Fallback to builtins. Then error if we got nothing.
                            let builtins_rc = self.builtins.clone();
                            match builtins_rc.get::<str>(&potential_cmd) {
                                Some(builtin) => {
                                    if let Err(err) = builtin.validate_args(&args) {
                                        println!("{:?}", err);
                                        continue;
                                    }
                                    println!("{}", builtin.execute(self, &args)?);
                                }
                                None => println!("Unrecognized command: '{}'", potential_cmd),
                            }
                        }
                    };
                }
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
