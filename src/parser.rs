use crate::command::Command;
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
    pub remaining: Vec<&'a str>,
    cmd_type: CommandType,
    pub possibilities: Vec<String>,
    pub complete: bool,
}

impl<'a> Outcome<'a> {
    pub fn error_msg(&self) -> String {
        if self.complete {
            return String::from("");
        }

        let mut msg = String::new();

        if self.cmd_path.is_empty() && self.remaining.is_empty() {
            msg += "Empty string could not be parsed as a command.";
        } else if self.cmd_path.is_empty() {
            if let Some(first_remaining_word) = self.remaining.get(0) {
                msg.push_str(&format!(
                    "'{}' is not a recognized command.",
                    first_remaining_word
                ));
            } else {
                unreachable!("remaining unparsed tokens cannot be empty at this point")
            }
        } else {
            msg += "Failed to parse fully:\n";
            msg += "\n";

            let valid_prefix = self.cmd_path.join(" ");
            let invalid_suffix = self.remaining.join(" ");
            msg += "\t    (spaces trimmed)\n";
            if self.remaining.is_empty() {
                msg += &format!("\t => '{}  '\n", valid_prefix);
                msg += &format!("\t     {}^\n", " ".repeat(valid_prefix.len() + 1));
            } else {
                msg += &format!("\t => '{} {}'\n", valid_prefix, invalid_suffix);
                msg += &format!("\t     {}^\n", " ".repeat(valid_prefix.len() + 1));
            }
            msg += "expected a valid subcommand\n";
            msg += "instead, got: ";
            if let Some(first_remaining_word) = self.remaining.get(0) {
                msg += &format!("'{}';\n", first_remaining_word);
            } else {
                msg += "nothing;\n"
            }

            msg += "\n";
            msg.push_str(&format!(
                "Run '{} help' for more info on the command.",
                valid_prefix
            ));
        }

        if !self.possibilities.is_empty() {
            msg += "\n\n";
            msg.push_str(&format!(
                "\t => expected one of {}.\n",
                self.possibilities
                    .iter()
                    .map(|s| format!("'{}'", s))
                    .collect::<Vec<String>>()
                    .join(" or ")
            ))
        }

        msg += "\n";
        msg += "Run 'helptree' for more info on the entire command tree.\n";

        msg
    }
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
                        possibilities: current_set.names(),
                        complete: false,
                    };
                }
            };

            // At this point, we have successfully found the token in the set.
            // Now, if this command has children, we want to go deeper into the set.
            // If it is a leaf command, and has we're actually done and can return the current
            // cmd_path and remaining tokens and complete.
            match &**looked_up_cmd {
                Command::Leaf(_) => {
                    // This is a leaf command, so we are actually simply done.
                    return Outcome {
                        cmd_path,
                        // NOTE Since i < len, .get(i+1..) will never panic.
                        remaining: tokens.get(i + 1..).unwrap().to_vec(),
                        cmd_type,
                        possibilities: Vec::new(),
                        complete: true,
                    };
                }
                Command::Parent(cmd) => {
                    current_set = cmd.sub_commands();
                }
            }
        }

        // We will basically only arrive here if the number of tokens is zero.
        Outcome {
            cmd_path,
            remaining: Vec::new(), // If we get here, we are out of tokens anyways.
            cmd_type: if tokens.is_empty() {
                CommandType::Unknown
            } else {
                cmd_type
            },
            possibilities: current_set.names(),
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

#[cfg(test)]
mod test {
    use super::*;

    use crate::command::BaseCommand;

    use anyhow::Result;

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

        #[cfg(not(tarpaulin_include))]
        fn validate_args(&self, _: &Vec<String>) -> Result<()> {
            Ok(())
        }

        #[cfg(not(tarpaulin_include))]
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
                        Command::new_leaf(ParseTestCommand::new("bar-c")),
                        Command::new_leaf(ParseTestCommand::new("baz-c")),
                        Command::new_parent(
                            "qux-c",
                            vec![
                                Command::new_leaf(ParseTestCommand::new("quux-c")),
                                Command::new_leaf(ParseTestCommand::new("corge-c")),
                            ],
                        ),
                    ],
                ),
                Command::new_leaf(ParseTestCommand::new("grault-c")),
                Command::new_leaf(ParseTestCommand::new("conflict-tie")),
                Command::new_leaf(ParseTestCommand::new(
                    "conflict-builtin-longer-match-but-still-loses",
                )),
                Command::new_parent(
                    "conflict-custom-wins",
                    vec![Command::new_leaf(ParseTestCommand::new("child"))],
                ),
            ]),
            CommandSet::new_from_vec(vec![
                Command::new_parent(
                    "foo-b",
                    vec![Command::new_leaf(ParseTestCommand::new("bar-b"))],
                ),
                Command::new_leaf(ParseTestCommand::new("conflict-tie")),
                Command::new_leaf(ParseTestCommand::new("conflict-custom-wins")),
                Command::new_parent(
                    "conflict-builtin-longer-match-but-still-loses",
                    vec![Command::new_leaf(ParseTestCommand::new("child"))],
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
                possibilities: Vec::new(),
                complete: true,
            }
        );
    }

    #[test]
    fn no_nesting_no_args() {
        let cmds = make_parser_cmds();

        assert_eq!(
            Parser::new().parse("foo-c bar-c", &cmds.0, &cmds.1),
            Outcome {
                cmd_path: vec!["foo-c", "bar-c"],
                remaining: vec![],
                cmd_type: CommandType::Custom,
                possibilities: Vec::new(),
                complete: true,
            }
        );
    }

    #[test]
    fn end_with_no_args_but_is_parent() {
        let cmds = make_parser_cmds();

        assert_eq!(
            Parser::new().parse("foo-c qux-c", &cmds.0, &cmds.1),
            Outcome {
                cmd_path: vec!["foo-c", "qux-c"],
                remaining: vec![],
                cmd_type: CommandType::Custom,
                possibilities: vec![String::from("corge-c"), String::from("quux-c")],
                complete: false,
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
                possibilities: Vec::new(),
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
                possibilities: vec![
                    String::from("conflict-builtin-longer-match-but-still-loses"),
                    String::from("conflict-custom-wins"),
                    String::from("conflict-tie"),
                    String::from("foo-c"),
                    String::from("grault-c"),
                ],
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
                possibilities: vec![
                    String::from("bar-c"),
                    String::from("baz-c"),
                    String::from("qux-c"),
                ],
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
                possibilities: Vec::new(),
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
                possibilities: Vec::new(),
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
                possibilities: Vec::new(),
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
                possibilities: vec![
                    String::from("conflict-builtin-longer-match-but-still-loses"),
                    String::from("conflict-custom-wins"),
                    String::from("conflict-tie"),
                    String::from("foo-c"),
                    String::from("grault-c"),
                ],
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
                possibilities: vec![
                    String::from("conflict-builtin-longer-match-but-still-loses"),
                    String::from("conflict-custom-wins"),
                    String::from("conflict-tie"),
                    String::from("foo-c"),
                    String::from("grault-c"),
                ],
                complete: false,
            }
        );
    }

    #[test]
    fn three_levels_deep() {
        let cmds = make_parser_cmds();

        assert_eq!(
            Parser::new().parse("foo-c qux-c quux-c la la", &cmds.0, &cmds.1),
            Outcome {
                cmd_path: vec!["foo-c", "qux-c", "quux-c"],
                remaining: vec!["la", "la"],
                cmd_type: CommandType::Custom,
                possibilities: Vec::new(),
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
                possibilities: Vec::new(),
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
                possibilities: Vec::new(),
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
                possibilities: Vec::new(),
                complete: true,
            }
        );
    }

    mod outcome {
        use super::{CommandType, Outcome};

        use pretty_assertions::assert_eq;

        #[test]
        fn outcome_error_msg() {
            let outcome = Outcome {
                cmd_path: vec!["foo", "bar"],
                remaining: vec!["la", "la"],
                cmd_type: CommandType::Custom,
                possibilities: Vec::new(),
                complete: false,
            };

            assert_eq!(
                outcome.error_msg(),
                vec![
                    "Failed to parse fully:\n",
                    "\n",
                    "\t    (spaces trimmed)\n",
                    "\t => 'foo bar la la'\n",
                    "\t             ^\n",
                    "expected a valid subcommand\n",
                    "instead, got: 'la';\n",
                    "\n",
                    "Run 'foo bar help' for more info on the command.\n",
                    "Run 'helptree' for more info on the entire command tree.\n",
                ]
                .join(""),
            );
        }

        #[test]
        fn empty_remaining_in_outcome() {
            let outcome = Outcome {
                cmd_path: vec!["foo", "bar"],
                remaining: vec![],
                cmd_type: CommandType::Custom,
                possibilities: Vec::new(),
                complete: false,
            };

            assert_eq!(
                outcome.error_msg(),
                vec![
                    "Failed to parse fully:\n",
                    "\n",
                    "\t    (spaces trimmed)\n",
                    "\t => 'foo bar  '\n",
                    "\t             ^\n",
                    "expected a valid subcommand\n",
                    "instead, got: nothing;\n",
                    "\n",
                    "Run 'foo bar help' for more info on the command.\n",
                    "Run 'helptree' for more info on the entire command tree.\n",
                ]
                .join(""),
            );
        }

        #[test]
        fn empty() {
            let outcome = Outcome {
                cmd_path: vec![],
                remaining: vec![],
                cmd_type: CommandType::Custom,
                possibilities: vec![
                    String::from("conflict-tie"),
                    String::from("conflict-builtin-longer-match-but-still-loses"),
                    String::from("conflict-custom-wins"),
                    String::from("foo-c"),
                    String::from("grault-c"),
                ],
                complete: false,
            };

            assert_eq!(
                outcome.error_msg(),
                vec![
                    "Empty string could not be parsed as a command.\n",
                    "\n",
                    "\n",
                    "\t => expected one of 'conflict-tie' or 'conflict-builtin-longer-match-but-still-loses' or 'conflict-custom-wins' or 'foo-c' or 'grault-c'.",
                    "\n",
                    "\n",
                    "Run 'helptree' for more info on the entire command tree.\n",
                ]
                .join(""),
            );
        }

        #[test]
        fn unrecognized_first_cmd() {
            let outcome = Outcome {
                cmd_path: vec![],
                remaining: vec!["notfound", "la"],
                cmd_type: CommandType::Custom,
                possibilities: Vec::new(),
                complete: false,
            };

            assert_eq!(
                outcome.error_msg(),
                vec![
                    "'notfound' is not a recognized command.\n",
                    "\n",
                    "Run 'helptree' for more info on the entire command tree.\n",
                ]
                .join(""),
            );
        }

        #[test]
        fn error_msg_is_blank_for_complete_parse() {
            let outcome = Outcome {
                cmd_path: vec![],
                remaining: vec![],
                cmd_type: CommandType::Custom,
                possibilities: Vec::new(),
                complete: true,
            };

            assert_eq!(outcome.error_msg(), String::from(""));
        }
    }
}
