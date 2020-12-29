use crate::command::{BaseCommand, Command};
use crate::command_set::CommandSet;
use crate::shell::Shell;
use crate::tokenizer::{DefaultTokenizer, Tokenizer};

pub struct Parser {
    tokenizer: DefaultTokenizer,
}

#[derive(Debug)]
pub enum CommandType {
    Builtin,
    Custom,
    Unknown,
}

#[derive(Debug)]
pub struct Outcome<'a> {
    cmd_path: Vec<&'a str>,
    remaining: Vec<&'a str>,
    cmd_type: CommandType,
    complete: bool,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            tokenizer: DefaultTokenizer::new(vec!['\'', '"']),
        }
    }

    fn parse_tokens_with_set<'a, T>(
        &self,
        tokens: &Vec<&'a str>,
        cmd_type: CommandType,
        set: &CommandSet<T>,
    ) -> Outcome<'a> {
        let mut cmd_path: Vec<&str> = Vec::new();
        let mut current_set = set;
        for (i, token) in tokens.iter().enumerate() {
            // Try looking up the token in our set.
            let looked_up_cmd = match current_set.get(token) {
                Some(cmd) => {
                    cmd_path.push(token);
                    cmd
                }
                None => {
                    return Outcome {
                        cmd_path,
                        remaining: tokens.get(i..).unwrap().to_vec(),
                        cmd_type,
                        complete: false,
                    }
                }
            };

            // At this point, we have successfully found the token in the set.
            // Now, if this command has children, we want to go deeper into the set.
            // If it is a leaf command, and has we're actually done and can return the current
            // cmd_path and remaining tokens and complete.
            match &**looked_up_cmd {
                Command::Child(_) => {
                    // This is a leaf command, so we are actually simply done.
                    return Outcome {
                        cmd_path,
                        remaining: tokens.get(i + 1..).unwrap().to_vec(),
                        cmd_type,
                        complete: true,
                    };
                }
                Command::Parent(cmd) => {
                    current_set = cmd.sub_commands();
                }
            }
        }

        Outcome {
            cmd_path: Vec::new(),
            remaining: Vec::new(),
            cmd_type,
            complete: true,
        }
    }

    fn parse_tokens<'a, S>(
        &self,
        tokens: &Vec<&'a str>,
        cmds: &CommandSet<S>,
        builtins: &CommandSet<Shell<S>>,
    ) -> Outcome<'a> {
        let cmd_outcome = self.parse_tokens_with_set(tokens, CommandType::Custom, cmds);
        if cmd_outcome.complete {
            return cmd_outcome;
        }

        let builtin_outcome = self.parse_tokens_with_set(tokens, CommandType::Builtin, builtins);
        if builtin_outcome.complete {
            return builtin_outcome;
        }

        // If neither worked, take the one closest to a match:
        if cmd_outcome.cmd_path.len() < builtin_outcome.cmd_path.len() {
            return builtin_outcome;
        } else {
            return cmd_outcome;
        }
    }

    pub fn parse<'a, S>(
        &self,
        line: &'a str,
        cmds: &CommandSet<S>,
        builtins: &CommandSet<Shell<S>>,
    ) -> Outcome<'a> {
        let tokens = self.tokenizer.tokenize(line);
        self.parse_tokens(&tokens, cmds, builtins)
    }
}

mod test {
    use super::*;

    use anyhow::Result;

    #[derive(Debug)]
    struct ParseTestCommand<'a> {
        name: &'a str,
    }

    impl<'a> ParseTestCommand<'a> {
        // TODO: We may actually prefer to make this return Box<> to make our API less verbose.
        fn new(name: &str) -> ParseTestCommand {
            ParseTestCommand { name }
        }
    }

    impl<'a> BaseCommand for ParseTestCommand<'a> {
        type State = ();

        fn name(&self) -> &str {
            self.name
        }

        fn validate_args(&self, _: &Vec<String>) -> Result<()> {
            Ok(())
        }

        fn execute(&self, _: &mut (), _: &Vec<String>) -> Result<String> {
            Ok(String::from(""))
        }
    }

    fn make_parser_cmds<'a>() -> (CommandSet<'a, ()>, CommandSet<'a, Shell<'a, ()>>) {
        (
            CommandSet::new_from_vec(vec![
                Command::new_parent(
                    "foo-c",
                    vec![
                        Command::new_child(ParseTestCommand::new("bar-c")),
                        Command::new_child(ParseTestCommand::new("baz-c")),
                        Command::new_parent(
                            "qux",
                            vec![
                                Command::new_child(ParseTestCommand::new("quux")),
                                Command::new_child(ParseTestCommand::new("corge")),
                            ],
                        ),
                    ],
                ),
                Command::new_child(ParseTestCommand::new("grault")),
            ]),
            CommandSet::new_from_vec(vec![]),
        )
    }

    #[test]
    fn test() {
        let cmds = make_parser_cmds();
        let p = Parser::new();

        println!(
            "Parse outcome: {:?}",
            p.parse("foo-c bar-c he", &cmds.0, &cmds.1)
        );

        println!("Parse outcome: {:?}", p.parse("foo-c he", &cmds.0, &cmds.1));
        println!(
            "Parse outcome: {:?}",
            p.parse("grault lala", &cmds.0, &cmds.1)
        );
        println!(
            "Parse outcome: {:?}",
            p.parse("grault foo-c bar-c", &cmds.0, &cmds.1)
        );
        println!(
            "Parse outcome: {:?}",
            p.parse("notacmd ha ha", &cmds.0, &cmds.1)
        );
        println!(
            "Parse outcome: {:?}",
            p.parse("foo-c qux quux", &cmds.0, &cmds.1)
        );
        println!(
            "Parse outcome: {:?}",
            p.parse("foo-c qux quux la la la", &cmds.0, &cmds.1)
        );
    }
}
