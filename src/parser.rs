use crate::command::{BaseCommand, Command};
use crate::command_set::CommandSet;
use crate::shell::Shell;
use crate::tokenizer::{DefaultTokenizer, Tokenizer};

pub struct Parser {
    tokenizer: DefaultTokenizer,
}

#[derive(Debug, PartialEq)]
pub enum CommandType {
    Builtin,
    Custom,
    Unknown,
}

#[derive(Debug, PartialEq)]
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
                        // NOTE Since i < len, .get(i..) will never panic.
                        remaining: tokens.get(i..).unwrap().to_vec(),
                        cmd_type: if i == 0 {
                            // If this is the first lookup, then obviously we have no idea what the
                            // type is.
                            CommandType::Unknown
                        } else {
                            cmd_type
                        },
                        complete: false,
                    };
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
                        // NOTE Since i < len, .get(i+1..) will never panic.
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
            remaining: tokens.to_vec(),
            cmd_type: CommandType::Unknown,
            complete: false,
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

        return cmd_outcome;
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

    #[cfg(test)]
    use pretty_assertions::assert_eq;

    use std::marker::PhantomData;

    #[derive(Debug)]
    struct ParseTestCommand<'a, S> {
        name: &'a str,
        phantom: PhantomData<S>,
    }

    impl<'a, S> ParseTestCommand<'a, S> {
        fn new(name: &str) -> ParseTestCommand<S> {
            ParseTestCommand {
                name,
                phantom: PhantomData,
            }
        }
    }

    impl<'a, S> BaseCommand for ParseTestCommand<'a, S> {
        type State = S;

        fn name(&self) -> &str {
            self.name
        }

        fn validate_args(&self, _: &Vec<String>) -> Result<()> {
            Ok(())
        }

        fn execute(&self, _: &mut S, _: &Vec<String>) -> Result<String> {
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
                            "qux-c",
                            vec![
                                Command::new_child(ParseTestCommand::new("quux-c")),
                                Command::new_child(ParseTestCommand::new("corge-c")),
                            ],
                        ),
                    ],
                ),
                Command::new_child(ParseTestCommand::new("grault-c")),
                Command::new_child(ParseTestCommand::new("conflict-tie")),
                Command::new_child(ParseTestCommand::new(
                    "conflict-builtin-longer-match-but-still-loses",
                )),
                Command::new_parent(
                    "conflict-custom-wins",
                    vec![Command::new_child(ParseTestCommand::new("child"))],
                ),
            ]),
            CommandSet::new_from_vec(vec![
                Command::new_parent(
                    "foo-b",
                    vec![Command::new_child(ParseTestCommand::new("bar-b"))],
                ),
                Command::new_child(ParseTestCommand::new("conflict-tie")),
                Command::new_child(ParseTestCommand::new("conflict-custom-wins")),
                Command::new_parent(
                    "conflict-builtin-longer-match-but-still-loses",
                    vec![Command::new_child(ParseTestCommand::new("child"))],
                ),
            ]),
        )
    }

    #[test]
    fn nesting() {
        let cmds = make_parser_cmds();

        assert_eq!(
            Parser::new().parse("foo-c bar-c he", &cmds.0, &cmds.1),
            Outcome {
                cmd_path: vec!["foo-c", "bar-c"],
                remaining: vec!["he"],
                cmd_type: CommandType::Custom,
                complete: true,
            }
        );
    }

    #[test]
    fn nesting_no_args() {
        let cmds = make_parser_cmds();

        assert_eq!(
            Parser::new().parse("foo-c bar-c", &cmds.0, &cmds.1),
            Outcome {
                cmd_path: vec!["foo-c", "bar-c"],
                remaining: vec![],
                cmd_type: CommandType::Custom,
                complete: true,
            }
        );
    }

    #[test]
    fn builtin_nesting() {
        let cmds = make_parser_cmds();

        assert_eq!(
            Parser::new().parse("foo-b bar-b he", &cmds.0, &cmds.1),
            Outcome {
                cmd_path: vec!["foo-b", "bar-b"],
                remaining: vec!["he"],
                cmd_type: CommandType::Builtin,
                complete: true,
            }
        );
    }

    #[test]
    fn empty() {
        let cmds = make_parser_cmds();

        assert_eq!(
            Parser::new().parse("", &cmds.0, &cmds.1),
            Outcome {
                cmd_path: vec![],
                remaining: vec![],
                cmd_type: CommandType::Unknown,
                complete: false,
            }
        );
    }

    #[test]
    fn invalid_subcmd() {
        let cmds = make_parser_cmds();

        assert_eq!(
            Parser::new().parse("foo-c he", &cmds.0, &cmds.1),
            Outcome {
                cmd_path: vec!["foo-c"],
                remaining: vec!["he"],
                cmd_type: CommandType::Custom,
                complete: false,
            }
        );
    }

    #[test]
    fn no_nesting() {
        let cmds = make_parser_cmds();

        assert_eq!(
            Parser::new().parse("grault-c la la", &cmds.0, &cmds.1),
            Outcome {
                cmd_path: vec!["grault-c"],
                remaining: vec!["la", "la"],
                cmd_type: CommandType::Custom,
                complete: true,
            }
        );
    }

    #[test]
    fn no_args_no_nesting() {
        let cmds = make_parser_cmds();

        assert_eq!(
            Parser::new().parse("grault-c", &cmds.0, &cmds.1),
            Outcome {
                cmd_path: vec!["grault-c"],
                remaining: vec![],
                cmd_type: CommandType::Custom,
                complete: true,
            }
        );
    }

    #[test]
    fn cmd_has_args_that_match_other_cmds() {
        let cmds = make_parser_cmds();

        assert_eq!(
            Parser::new().parse("grault-c foo-c bar-c", &cmds.0, &cmds.1),
            Outcome {
                cmd_path: vec!["grault-c"],
                // Although these match other command names, since they come after grault, we
                // expect them to be treated as basic arguments.
                remaining: vec!["foo-c", "bar-c"],
                cmd_type: CommandType::Custom,
                complete: true,
            }
        );
    }

    #[test]
    fn nonexistent_cmd() {
        let cmds = make_parser_cmds();

        assert_eq!(
            Parser::new().parse("notacmd", &cmds.0, &cmds.1),
            Outcome {
                cmd_path: vec![],
                remaining: vec!["notacmd"],
                cmd_type: CommandType::Unknown,
                complete: false,
            }
        );
    }

    #[test]
    fn args_with_nonexistent_cmd() {
        let cmds = make_parser_cmds();

        assert_eq!(
            Parser::new().parse("notacmd la la", &cmds.0, &cmds.1),
            Outcome {
                cmd_path: vec![],
                remaining: vec!["notacmd", "la", "la"],
                cmd_type: CommandType::Unknown,
                complete: false,
            }
        );
    }

    #[test]
    fn thee_levels_deep() {
        let cmds = make_parser_cmds();

        assert_eq!(
            Parser::new().parse("foo-c qux-c quux-c la la", &cmds.0, &cmds.1),
            Outcome {
                cmd_path: vec!["foo-c", "qux-c", "quux-c"],
                remaining: vec!["la", "la"],
                cmd_type: CommandType::Custom,
                complete: true,
            }
        );
    }

    #[test]
    fn perfect_tie_custom_wins_tie_breaker() {
        let cmds = make_parser_cmds();

        assert_eq!(
            Parser::new().parse("conflict-tie ha ha", &cmds.0, &cmds.1),
            Outcome {
                cmd_path: vec!["conflict-tie"],
                remaining: vec!["ha", "ha"],
                cmd_type: CommandType::Custom,
                complete: true,
            }
        );
    }

    #[test]
    fn conflict_but_builtin_has_longer_match() {
        let cmds = make_parser_cmds();

        assert_eq!(
            Parser::new().parse(
                "conflict-builtin-longer-match-but-still-loses child ha",
                &cmds.0,
                &cmds.1
            ),
            Outcome {
                cmd_path: vec!["conflict-builtin-longer-match-but-still-loses"],
                remaining: vec!["child", "ha"],
                cmd_type: CommandType::Custom,
                complete: true,
            }
        );
    }

    #[test]
    fn conflict_but_custom_has_longer_match() {
        let cmds = make_parser_cmds();

        assert_eq!(
            Parser::new().parse("conflict-custom-wins child ha", &cmds.0, &cmds.1),
            Outcome {
                cmd_path: vec!["conflict-custom-wins", "child"],
                remaining: vec!["ha"],
                cmd_type: CommandType::Custom,
                complete: true,
            }
        );
    }
}
