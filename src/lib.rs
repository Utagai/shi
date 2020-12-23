pub mod command;
pub mod shell;

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
                Box::new(ParentCommand::new(
                    "isbn",
                    vec![
                        Box::new(BasicCommand::new("eu", |the_lst: &mut Vec<String>, _| {
                            the_lst.push("eu".to_owned());
                            Ok(String::from("Added 'eu'"))
                        })),
                        Box::new(BasicCommand::new("us", |the_lst: &mut Vec<String>, _| {
                            the_lst.push("us".to_owned());
                            Ok(String::from("Added 'us'"))
                        })),
                    ],
                )),
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
