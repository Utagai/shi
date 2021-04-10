//! A crate for building shell interfaces in Rust.
//!
//! See the README.md and examples for more information.

use std::result;

pub mod command;
mod command_set;
pub mod error;
mod parser;
mod readline;
pub mod shell;
mod tokenizer;

pub type Result<T> = result::Result<T, error::ShiError>;

#[cfg(test)]
mod test {
    use super::command::example::EchoCommand;
    use super::command::*;
    use super::shell::*;
    use super::Result;

    struct CustomCommand {}

    impl CustomCommand {
        // TODO: We may actually prefer to make this return Box<> to make our API less verbose.
        #[allow(dead_code)]
        pub fn new() -> CustomCommand {
            CustomCommand {}
        }
    }

    impl BaseCommand for CustomCommand {
        type State = Vec<String>;

        fn name(&self) -> &str {
            "custom"
        }

        fn validate_args(&self, _: &[String]) -> Result<()> {
            Ok(())
        }

        fn execute(&self, state: &mut Vec<String>, args: &[String]) -> Result<String> {
            println!("hehe I am custom! state is: {:?}", state.get(0));
            match args.get(0) {
                Some(arg) => state.push(format!("HIJACKED: '{}'", arg)),
                None => state.push(String::from("HIJACKED!")),
            };
            Ok(String::from("yo"))
        }
    }

    #[allow(dead_code)]
    fn fake_main() -> Result<()> {
        let lst: Vec<String> = Vec::new();

        let mut shell = Shell::new_with_state("| ", lst);

        shell.set_and_load_history_file("readline_history.txt")?;
        shell.register(Command::new_leaf(EchoCommand::new()))?;
        shell.register(Command::new_leaf(BasicCommand::new("remove", |_, _| {
            Ok(String::from("I AM REMOVE CLOSURE!!!"))
        })))?;
        shell.register(Command::new_leaf(BasicCommand::new(
            "list",
            |the_lst: &mut Vec<String>, _| {
                Ok(format!(
                    "Current: [{}]",
                    the_lst
                        .iter()
                        .map(|f| format!("{:?}", f))
                        .collect::<Vec<String>>()
                        .join(", ")
                ))
            },
        )))?;
        shell.register(Command::new_parent(
            "add",
            vec![
                Command::new_leaf(BasicCommand::new(
                    "title",
                    |the_lst: &mut Vec<String>, _| {
                        the_lst.push("title".to_owned());
                        Ok(String::from("Added 'title'"))
                    },
                )),
                Command::new_parent(
                    "isbn",
                    vec![
                        Command::new_leaf(BasicCommand::new(
                            "eu",
                            |the_lst: &mut Vec<String>, _| {
                                the_lst.push("eu".to_owned());
                                Ok(String::from("Added 'eu'"))
                            },
                        )),
                        Command::new_leaf(BasicCommand::new(
                            "us",
                            |the_lst: &mut Vec<String>, _| {
                                the_lst.push("us".to_owned());
                                Ok(String::from("Added 'us'"))
                            },
                        )),
                    ],
                ),
            ],
        ))?;
        shell.register(Command::new_leaf(CustomCommand::new()))?;

        // shell.run()?;

        Ok(())
    }
}
