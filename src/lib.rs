use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

// TODO: We should probably be using thiserror if this is going to be factored out into a library.
use anyhow::{bail, Result};
use rustyline::{error::ReadlineError, Editor};

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

impl<'a, S> Command for BasicCommand<'a, S> {
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

#[derive(Debug)]
pub struct EchoCommand<'a, S> {
    name: &'a str,
    phantom: PhantomData<S>,
}

impl<'a, S> EchoCommand<'a, S> {
    pub fn new() -> EchoCommand<'a, S> {
        EchoCommand {
            name: "echo",
            phantom: PhantomData,
        }
    }
}

impl<'a, S> Command for EchoCommand<'a, S> {
    type State = S;

    fn name(&self) -> &str {
        self.name
    }

    fn validate_args(&self, args: &Vec<String>) -> Result<()> {
        if args.len() == 0 {
            bail!("'{}' requires a non-zero number of arguments!")
        }

        Ok(())
    }

    fn execute(&self, _: &mut S, args: &Vec<String>) -> Result<String> {
        Ok(format!("ECHO: '{:?}'", args))
    }
}

pub struct HelpCommand<'a, S> {
    phantom: &'a PhantomData<S>,
}

impl<'a, S> HelpCommand<'a, S> {
    pub fn new() -> HelpCommand<'a, S> {
        HelpCommand {
            phantom: &PhantomData,
        }
    }
}

impl<'a, S> Command for HelpCommand<'a, S> {
    type State = Shell<'a, S>;

    fn name(&self) -> &str {
        "help"
    }

    fn validate_args(&self, args: &Vec<String>) -> Result<()> {
        if args.len() != 0 {
            // TODO: We may want to make this actually take arguments, like a command name or
            // command name path.
            bail!("help takes no arguments")
        }

        Ok(())
    }

    fn execute(&self, shell: &mut Shell<S>, _: &Vec<String>) -> Result<String> {
        let mut help_lines: Vec<String> =
            Vec::with_capacity(shell.cmds.len() + shell.builtins.len());
        help_lines.push(String::from("Normal commands:"));
        for cmd in shell.cmds.values() {
            help_lines.push(format!("\t'{}' - {}", cmd.name(), cmd.help()));
        }

        help_lines.push(String::from("Built-in commands:"));
        for builtin in shell.builtins.values() {
            help_lines.push(format!("\t'{}' - {}", builtin.name(), builtin.help()))
        }

        Ok(help_lines.join("\n"))
    }
}

pub struct ExitCommand<'a, S> {
    phantom: &'a PhantomData<S>,
}

impl<'a, S> ExitCommand<'a, S> {
    pub fn new() -> ExitCommand<'a, S> {
        ExitCommand {
            phantom: &PhantomData,
        }
    }
}

impl<'a, S> Command for ExitCommand<'a, S> {
    type State = Shell<'a, S>;

    fn name(&self) -> &str {
        "exit"
    }

    fn validate_args(&self, args: &Vec<String>) -> Result<()> {
        if args.len() != 0 {
            bail!("exit takes no arguments")
        }

        Ok(())
    }

    fn execute(&self, shell: &mut Shell<S>, _: &Vec<String>) -> Result<String> {
        shell.terminate = true;
        Ok(String::from("bye"))
    }
}

pub struct HistoryCommand<'a, S> {
    phantom: &'a PhantomData<S>,
}

impl<'a, S> HistoryCommand<'a, S> {
    pub fn new() -> HistoryCommand<'a, S> {
        HistoryCommand {
            phantom: &PhantomData,
        }
    }
}

impl<'a, S> Command for HistoryCommand<'a, S> {
    type State = Shell<'a, S>;

    fn name(&self) -> &str {
        "history"
    }

    fn validate_args(&self, args: &Vec<String>) -> Result<()> {
        if args.len() != 0 {
            // TODO: We will probably want to take an optional flag for searching.
            // TODO: Maybe an optional flag for num items.
            bail!("history takes no arguments")
        }

        Ok(())
    }

    fn execute(&self, shell: &mut Shell<S>, _: &Vec<String>) -> Result<String> {
        let history_output = shell
            .rl
            .history()
            .iter()
            .map(|h| h.to_string())
            .collect::<Vec<String>>()
            .join("\n\t");
        // Add an extra tab because the first line won't have the join separator attached, and will
        // therefore only have the \n from the print.
        Ok(format!("\t{}", history_output))
    }
}

pub struct Shell<'a, S> {
    prompt: &'a str,
    cmds: HashMap<String, Box<dyn Command<State = S> + 'a>>,
    builtins: Rc<HashMap<String, Box<dyn Command<State = Self> + 'a>>>,
    rl: Editor<()>,
    history_file: Option<&'a str>,
    state: S,
    terminate: bool,
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

#[cfg(test)]
mod test {
    use super::*;

    struct CustomCommand {}

    impl CustomCommand {
        // TODO: We may actually prefer to make this return Box<> to make our API less verbose.
        pub fn new() -> CustomCommand {
            CustomCommand {}
        }
    }

    impl Command for CustomCommand {
        type State = Vec<String>;

        fn name(&self) -> &str {
            "custom"
        }

        fn validate_args(&self, _: &Vec<String>) -> Result<()> {
            Ok(())
        }

        fn execute(&self, state: &mut Vec<String>, args: &Vec<String>) -> Result<String> {
            println!("hehe I am custom! state is: {:?}", state.get(0));
            match args.get(0) {
                Some(arg) => state.push(format!("HIJACKED: '{}'", arg)),
                None => state.push(String::from("HIJACKED!")),
            };
            Ok(String::from("yo"))
        }
    }

    fn fake_main() -> Result<()> {
        let lst: Vec<String> = Vec::new();

        let mut shell = Shell::new_with_state("| ", lst);

        shell.set_history_file("readline_history.txt")?;
        shell.register(EchoCommand::new())?;
        shell.register(BasicCommand::new("remove", |_, _| {
            Ok(String::from("I AM REMOVE CLOSURE!!!"))
        }))?;
        shell.register(BasicCommand::new("list", |the_lst: &mut Vec<String>, _| {
            Ok(format!(
                "Current: [{}]",
                the_lst
                    .iter()
                    .map(|f| format!("{:?}", f))
                    .collect::<Vec<String>>()
                    .join(", ")
            ))
        }))?;
        shell.register(ParentCommand::new(
            "add",
            vec![
                Box::new(BasicCommand::new(
                    "title",
                    |the_lst: &mut Vec<String>, _| {
                        the_lst.push("title".to_owned());
                        Ok(String::from("Added 'title'"))
                    },
                )),
                Box::new(BasicCommand::new("isbn", |the_lst: &mut Vec<String>, _| {
                    the_lst.push("isbn".to_owned());
                    Ok(String::from("Added 'isbn'"))
                })),
            ],
        ))?;
        shell.register(CustomCommand::new())?;

        shell.run()?;

        Ok(())
    }

    #[test]
    fn it_works() {
        match fake_main() {
            Ok(_) => println!("YAY!"),
            Err(err) => {
                println!("ERR: {:?}", err);
                assert_eq!(0, 1);
            }
        }
    }
}
