use crate::command::{Command, Completion};
use crate::command_set::CommandSet;
use crate::error::ShiError;
use crate::shell::Shell;
use crate::tokenizer::{DefaultTokenizer, Tokenization, Tokenizer};

/// A parser that parses input lines into `Command` invocations.
pub struct Parser {
    tokenizer: DefaultTokenizer,
}

#[derive(Debug, PartialEq)]
/// CommandType represents part of a parse result. A parse attempt for a command will result in a
/// decision on whether a given input line represents a `Builtin` command, a `Custom` command, or
/// `Unknown`, in the case of an unsuccessful or incomplete parse.
pub enum CommandType {
    Builtin,
    Custom,
    Unknown,
}

#[derive(Debug, PartialEq)]
/// Outcome is the final result of a parse attempt. It includes various useful bits of information
/// from the parse:
///
/// * `cmd_path` - The components of the command invocation. In particular, it shows the chain of
///   ancestry in terms of `Parent` commands and the eventual `Leaf` command.
/// * `remaining` - The remaining components of the string. In the case of a successful parse, this
///   represents the arguments passed to the command. In the case of unsuccessful or incomplete
///   parses, this represents the part of the input string that was not able to be parsed.
/// * `cmd_type` - The type of the command. See `CommandType`.
/// * `possibilities` - Includes the potential candidates that the parser is expecting to see
///   following the input line.
/// * `complete` - A flag denoting whether we had a successful and complete parse.
pub struct Outcome<'a> {
    pub cmd_path: Vec<&'a str>,
    pub remaining: Vec<&'a str>,
    pub cmd_type: CommandType,
    pub possibilities: Vec<String>,
    pub leaf_completion: Option<Completion>,
    pub complete: bool,
}

impl<'a> Outcome<'a> {
    pub fn error(&self) -> Option<ShiError> {
        if !self.complete {
            Some(ShiError::ParseError {
                msg: self.error_msg(),
                cmd_path: self.cmd_path.iter().map(|s| s.to_string()).collect(),
                remaining: self.remaining.iter().map(|s| s.to_string()).collect(),
                possibilities: self.possibilities.clone(),
            })
        } else {
            None
        }
    }

    /// Prints an error message for the `Outcome`. Of course, if the `Outcome` was complete, the
    /// error message is empty.
    pub fn error_msg(&self) -> String {
        // TODO: We should split apart this function.

        // If we parsed successfully, we obviously shouldn't produce an error message.
        if self.complete {
            return String::from("");
        }

        // This will be our String buffer.
        let mut msg = String::new();

        if self.cmd_path.is_empty() && self.remaining.is_empty() {
            // In this case, we must have found an empty string, which is obviously not parseable
            // as a command.
            msg += "Empty string could not be parsed as a command.";
        } else if self.cmd_path.is_empty() {
            // If the `cmd_path` is empty, this implies we immediately failed the parse, and it was
            // not at least partially complete. This then implies that the first element of the
            // remaining components must be the thing we failed to parse as a recognized command.
            if let Some(first_remaining_word) = self.remaining.first() {
                msg.push_str(&format!(
                    "'{}' is not a recognized command.",
                    first_remaining_word
                ));
            } else {
                // This should not be possible. If the remaining tokens were empty, then the prior
                // if case should have caught it.
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
            } else {
                msg += &format!("\t => '{} {}'\n", valid_prefix, invalid_suffix);
            }
            msg += &format!("\t     {}^\n", " ".repeat(valid_prefix.len() + 1));

            msg += "expected a valid subcommand\n";
            msg += "instead, got: ";
            if let Some(first_remaining_word) = self.remaining.first() {
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
    /// Constructs a new Parser.
    pub fn new() -> Parser {
        Parser {
            tokenizer: DefaultTokenizer::new(vec!['\'', '"']),
        }
    }

    /// Parses a given Vector of tokens into a parse `Outcome`.
    ///
    /// # Arguments
    /// `tokens` - The tokens produced from an input line.
    /// `cmd_type` - The type of command contained in `set`. See `CommandType`.
    /// `set` - The available commands to parse into.
    ///
    /// # Returns
    /// `Outcome` - The parse outcome, given the arguments.
    fn parse_tokens_with_set<'a, T>(
        &self,
        tokenization: &Tokenization<'a>,
        cmd_type: CommandType,
        set: &CommandSet<T>,
    ) -> Outcome<'a> {
        let mut cmd_path: Vec<&str> = Vec::new();
        let mut current_set = set;
        for (i, token) in tokenization.tokens.iter().enumerate() {
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
                        remaining: tokenization.tokens.get(i..).unwrap().to_vec(),
                        cmd_type: if i == 0 {
                            // If this is the first lookup, then obviously we have no idea what the
                            // type is.
                            CommandType::Unknown
                        } else {
                            cmd_type
                        },
                        possibilities: current_set.names(),
                        leaf_completion: None,
                        complete: false,
                    };
                }
            };

            // At this point, we have successfully found the token in the set.
            // Now, if this command has children, we want to go deeper into the set.
            // If it is a leaf command, and has we're actually done and can return the current
            // cmd_path and remaining tokens and complete.
            match &**looked_up_cmd {
                Command::Leaf(cmd) => {
                    // This is a leaf command, so we are actually almost done.
                    // Leaf commands themselves, can, given their arguments, attempt a local
                    // autocompletion. Let's give that a shot and then finish.
                    return Outcome {
                        cmd_path,
                        // NOTE Since i < len, .get(i+1..) will never panic.
                        remaining: tokenization.tokens.get(i + 1..).unwrap().to_vec(),
                        cmd_type,
                        possibilities: Vec::new(),
                        leaf_completion: Some(cmd.autocomplete(
                            tokenization.tokens.get(i + 1..).unwrap().to_vec(),
                            tokenization.trailing_space,
                        )),
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
            cmd_type: if tokenization.tokens.is_empty() {
                CommandType::Unknown
            } else {
                cmd_type
            },
            possibilities: current_set.names(),
            leaf_completion: None,
            complete: false,
        }
    }

    /// Parses a given Vector of tokens into a parse `Outcome`.
    ///
    /// # Arguments
    /// `tokens` - The tokens produced from an input line.
    /// `cmds` - The available custom commands to parse into.
    /// `builtins` - The available builtins to parse into.
    ///
    /// # Returns
    /// `Outcome` - The parse outcome, given the arguments.
    fn parse_tokens<'a, S>(
        &self,
        tokenization: &Tokenization<'a>,
        cmds: &CommandSet<S>,
        builtins: &CommandSet<Shell<S>>,
    ) -> Outcome<'a> {
        let cmd_outcome = self.parse_tokens_with_set(tokenization, CommandType::Custom, cmds);
        if cmd_outcome.complete {
            return cmd_outcome;
        }

        let builtin_outcome =
            self.parse_tokens_with_set(tokenization, CommandType::Builtin, builtins);
        if builtin_outcome.complete {
            return builtin_outcome;
        }

        cmd_outcome
    }

    /// Parses the given information into a parse `Outcome`.
    ///
    /// # Arguments
    /// `line` - The input line.
    /// `cmds` - The available custom commands to parse into.
    /// `builtins` - The available builtins to parse into.
    ///
    /// # Returns
    /// `Outcome` - The parse outcome, given the arguments.
    pub fn parse<'a, S>(
        &self,
        line: &'a str,
        cmds: &CommandSet<S>,
        builtins: &CommandSet<Shell<S>>,
    ) -> Outcome<'a> {
        let tokenization = self.tokenizer.tokenize(line);
        self.parse_tokens(&tokenization, cmds, builtins)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    use crate::command::BaseCommand;
    use crate::Result;

    use pretty_assertions::assert_eq;

    use std::marker::PhantomData;

    // TODO: We should not need to stutter and call it 'ParseTestCommand'. Just call it
    // 'TestCommand'. It's obviously for parser tests.
    #[derive(Debug)]
    struct ParseTestCommand<'a, S> {
        name: &'a str,
        autocompletions: Vec<&'a str>,
        phantom: PhantomData<S>,
    }

    impl<'a, S> ParseTestCommand<'a, S> {
        fn new(name: &'a str) -> ParseTestCommand<'a, S> {
            ParseTestCommand {
                name,
                autocompletions: Vec::new(),
                phantom: PhantomData,
            }
        }

        fn new_with_completions(
            name: &'a str,
            completions: Vec<&'a str>,
        ) -> ParseTestCommand<'a, S> {
            ParseTestCommand {
                name,
                autocompletions: completions,
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
        fn validate_args(&self, _: &[String]) -> Result<()> {
            Ok(())
        }

        fn autocomplete(&self, args: Vec<&str>, _: bool) -> Completion {
            // If we don't have any autocompletions set, then just short-circuit out.
            if self.autocompletions.is_empty() {
                return Completion::Nothing;
            }

            match args.last() {
                Some(last) => {
                    if self.autocompletions.iter().filter(|s| s == &last).count() > 0 {
                        // If the last argument is in our autocompletions, then we're good, nothing
                        // more to complete.
                        Completion::Nothing
                    } else {
                        let prefix_matches: Vec<String> = self
                            .autocompletions
                            .iter()
                            .filter(|s| s.starts_with(last))
                            .map(|s| s.to_string())
                            .collect();

                        if prefix_matches.is_empty() {
                            // If nothing matched, then we have no completions.
                            return Completion::Nothing;
                        }
                        // If not, then perhaps it is a prefix of an autocompletion. Let's give
                        // back some partial arg completions if so!
                        Completion::PartialArgCompletion(prefix_matches)
                    }
                }
                None => Completion::Possibilities(
                    self.autocompletions.iter().map(|s| s.to_string()).collect(),
                ),
            }
        }

        #[cfg(not(tarpaulin_include))]
        fn execute(&self, _: &mut S, _: &[String]) -> Result<String> {
            Ok(String::from(""))
        }
    }

    pub fn make_parser_cmds<'a>() -> (CommandSet<'a, ()>, CommandSet<'a, Shell<'a, ()>>) {
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
                    vec![Command::new_leaf(ParseTestCommand::new_with_completions(
                        "bar-b",
                        vec!["ho", "he", "bum"],
                    ))],
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
                leaf_completion: Some(Completion::Nothing),
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
                leaf_completion: Some(Completion::Nothing),
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
                possibilities: vec![String::from("quux-c"), String::from("corge-c")],
                leaf_completion: None,
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
                leaf_completion: Some(Completion::Nothing),
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
                    String::from("foo-c"),
                    String::from("grault-c"),
                    String::from("conflict-tie"),
                    String::from("conflict-builtin-longer-match-but-still-loses"),
                    String::from("conflict-custom-wins"),
                ],
                leaf_completion: None,
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
                leaf_completion: None,
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
                leaf_completion: Some(Completion::Nothing),
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
                leaf_completion: Some(Completion::Nothing),
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
                leaf_completion: Some(Completion::Nothing),
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
                    String::from("foo-c"),
                    String::from("grault-c"),
                    String::from("conflict-tie"),
                    String::from("conflict-builtin-longer-match-but-still-loses"),
                    String::from("conflict-custom-wins"),
                ],
                leaf_completion: None,
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
                    String::from("foo-c"),
                    String::from("grault-c"),
                    String::from("conflict-tie"),
                    String::from("conflict-builtin-longer-match-but-still-loses"),
                    String::from("conflict-custom-wins"),
                ],
                leaf_completion: None,
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
                leaf_completion: Some(Completion::Nothing),
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
                leaf_completion: Some(Completion::Nothing),
                complete: true,
            }
        );
    }

    #[test]
    fn conflict_but_builtin_has_longer_match() {
        // We are testing that custom commands have a higher precedence. Although this command
        // exists identically in the builtin set, the custom variant is chosen.
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
                leaf_completion: Some(Completion::Nothing),
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
                leaf_completion: Some(Completion::Nothing),
                complete: true,
            }
        );
    }

    #[test]
    fn cmd_level_partial_autocompletion_multiple_choices() {
        let cmds = make_parser_cmds();

        assert_eq!(
            Parser::new().parse("foo-b bar-b h", &cmds.0, &cmds.1),
            Outcome {
                cmd_path: vec!["foo-b", "bar-b"],
                remaining: vec!["h"],
                cmd_type: CommandType::Builtin,
                possibilities: Vec::new(),
                leaf_completion: Some(Completion::PartialArgCompletion(vec![
                    String::from("ho"),
                    String::from("he")
                ])),
                complete: true,
            }
        );
    }

    #[test]
    fn cmd_level_partial_autocompletion_single_choice() {
        let cmds = make_parser_cmds();

        assert_eq!(
            Parser::new().parse("foo-b bar-b b", &cmds.0, &cmds.1),
            Outcome {
                cmd_path: vec!["foo-b", "bar-b"],
                remaining: vec!["b"],
                cmd_type: CommandType::Builtin,
                possibilities: Vec::new(),
                leaf_completion: Some(Completion::PartialArgCompletion(vec![String::from("bum"),])),
                complete: true,
            }
        );
    }

    #[test]
    fn cmd_level_completion_all_options() {
        let cmds = make_parser_cmds();

        assert_eq!(
            Parser::new().parse("foo-b bar-b", &cmds.0, &cmds.1),
            Outcome {
                cmd_path: vec!["foo-b", "bar-b"],
                remaining: vec![],
                cmd_type: CommandType::Builtin,
                possibilities: Vec::new(),
                leaf_completion: Some(Completion::Possibilities(vec![
                    String::from("ho"),
                    String::from("he"),
                    String::from("bum"),
                ])),
                complete: true,
            }
        );
    }

    #[test]
    fn cmd_level_completion_no_matches() {
        let cmds = make_parser_cmds();

        assert_eq!(
            Parser::new().parse("foo-b bar-b z", &cmds.0, &cmds.1),
            Outcome {
                cmd_path: vec!["foo-b", "bar-b"],
                remaining: vec!["z"],
                cmd_type: CommandType::Builtin,
                possibilities: Vec::new(),
                leaf_completion: Some(Completion::Nothing),
                complete: true,
            }
        );
    }

    #[test]
    fn cmd_level_completion_already_complete() {
        let cmds = make_parser_cmds();

        assert_eq!(
            Parser::new().parse("foo-b bar-b bum", &cmds.0, &cmds.1),
            Outcome {
                cmd_path: vec!["foo-b", "bar-b"],
                remaining: vec!["bum"],
                cmd_type: CommandType::Builtin,
                possibilities: Vec::new(),
                leaf_completion: Some(Completion::Nothing),
                complete: true,
            }
        );
    }

    mod outcome {
        use super::{CommandType, Completion, Outcome};

        use pretty_assertions::assert_eq;

        #[test]
        fn outcome_error_msg() {
            let outcome = Outcome {
                cmd_path: vec!["foo", "bar"],
                remaining: vec!["la", "la"],
                cmd_type: CommandType::Custom,
                possibilities: Vec::new(),
                leaf_completion: None,
                complete: false,
            };

            assert_eq!(
                outcome.error_msg(),
                [
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
                leaf_completion: None,
                complete: false,
            };

            assert_eq!(
                outcome.error_msg(),
                [
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
                leaf_completion: None,
                complete: false,
            };

            assert_eq!(
                outcome.error_msg(),
                [
                    "Empty string could not be parsed as a command.\n",
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
                leaf_completion: None,
                complete: false,
            };

            assert_eq!(
                outcome.error_msg(),
                [
                    "'notfound' is not a recognized command.\n",
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
                leaf_completion: Some(Completion::Nothing),
                complete: true,
            };

            assert_eq!(outcome.error_msg(), String::from(""));
        }
    }
}
