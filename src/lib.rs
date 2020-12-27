pub mod command;
mod command_set;
mod parser;
mod readline;
pub mod shell;
mod tokenizer;

#[cfg(test)]
mod test {
    use super::command::example::EchoCommand;
    use super::command::*;
    use super::shell::*;
    use anyhow::Result;

    struct CustomCommand {}

    impl CustomCommand {
        // TODO: We may actually prefer to make this return Box<> to make our API less verbose.
        pub fn new() -> CustomCommand {
            CustomCommand {}
        }
    }

    impl BaseCommand for CustomCommand {
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
        shell.register(Command::new_child(EchoCommand::new()))?;
        shell.register(Command::new_child(BasicCommand::new("remove", |_, _| {
            Ok(String::from("I AM REMOVE CLOSURE!!!"))
        })))?;
        shell.register(Command::new_child(BasicCommand::new(
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
                Command::new_child(BasicCommand::new(
                    "title",
                    |the_lst: &mut Vec<String>, _| {
                        the_lst.push("title".to_owned());
                        Ok(String::from("Added 'title'"))
                    },
                )),
                Command::new_parent(
                    "isbn",
                    vec![
                        Command::new_child(BasicCommand::new(
                            "eu",
                            |the_lst: &mut Vec<String>, _| {
                                the_lst.push("eu".to_owned());
                                Ok(String::from("Added 'eu'"))
                            },
                        )),
                        Command::new_child(BasicCommand::new(
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
        shell.register(Command::new_child(CustomCommand::new()))?;

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
