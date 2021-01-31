use std::borrow::Cow::{self, Borrowed, Owned};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use anyhow::Result;
use colored::*;

use rustyline::completion::{Completer, Pair};
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{self, MatchingBracketValidator, Validator};
use rustyline::{Config, Context, Editor};
use rustyline_derive::Helper;

use crate::command_set::CommandSet;
use crate::parser::Parser;
use crate::shell::Shell;

/// A wrapper around `rustyline::Editor`.
pub struct Readline<'a, S> {
    rl: Editor<ExecHelper<'a, S>>,
}

impl<'a, S> Readline<'a, S> {
    /// Constructs a new `Readline`.
    pub fn new(
        parser: Parser,
        cmds: Rc<RefCell<CommandSet<'a, S>>>,
        builtins: Rc<CommandSet<'a, Shell<'a, S>>>,
    ) -> Readline<'a, S> {
        let config = Config::builder()
            .completion_type(rustyline::CompletionType::List)
            .build();
        let mut rl = Editor::with_config(config);
        rl.set_helper(Some(ExecHelper::new(parser, cmds, builtins)));
        Readline { rl }
    }

    /// Loads the readline history from the given file.
    ///
    /// # Arguments
    /// `path` - The path to the history file to load history from.
    pub fn load_history<P: AsRef<Path> + ?Sized>(&mut self, path: &P) -> Result<()> {
        self.rl.load_history(path)?;
        Ok(())
    }

    /// Saves the history to the given file.
    ///
    /// # Arguments
    /// `path` - The path at which to save the history.
    pub fn save_history<P: AsRef<Path> + ?Sized>(&mut self, path: &P) -> Result<()> {
        self.rl.save_history(path)?;
        Ok(())
    }

    /// Adds a history entry to the history. This is done in memory. Persistence is achieved via
    /// `save_history()`.
    pub fn add_history_entry<E: AsRef<str> + Into<String>>(&mut self, line: E) -> bool {
        self.rl.add_history_entry(line)
    }

    /// Reads a line via the given prompt.
    ///
    /// # Arguments
    /// `prompt` - The prompt to display to the user.
    pub fn readline(&mut self, prompt: &str) -> rustyline::Result<String> {
        self.rl.readline(prompt)
    }

    /// Returns the readline `History`.
    ///
    /// Repeated, subsequent commands are not duplicated in the history.
    /// Invalid command invocations _are_ included in the history.
    /// May only be the commands executed in the current session, or it may also include prior
    /// sessions. This is dependent on whether `load_history()` was called for prior session
    /// histories.
    ///
    /// # Returns
    /// `rustyline::history::History` - The history of invoked commands.
    pub fn history(&self) -> &rustyline::history::History {
        self.rl.history()
    }
}

#[derive(Helper)]
/// An ExecHelper for supporting various `rustyline` features.
pub struct ExecHelper<'a, S> {
    completer: ExecCompleter<'a, S>,
    highlighter: MatchingBracketHighlighter,
    validator: MatchingBracketValidator,
    hinter: HistoryHinter,
    colored_prompt: String,
}

impl<'a, S> ExecHelper<'a, S> {
    /// Constructs an `ExecHelper`.
    fn new(
        parser: Parser,
        cmds: Rc<RefCell<CommandSet<'a, S>>>,
        builtins: Rc<CommandSet<'a, Shell<'a, S>>>,
    ) -> ExecHelper<'a, S> {
        ExecHelper {
            completer: ExecCompleter::new(parser, cmds, builtins),
            highlighter: MatchingBracketHighlighter::new(),
            validator: MatchingBracketValidator::new(),
            hinter: HistoryHinter {},
            colored_prompt: "| ".to_string(),
        }
    }
}

impl<'a, S> Completer for ExecHelper<'a, S> {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        self.completer.complete(line, pos)
    }
}

impl<'a, S> Hinter for ExecHelper<'a, S> {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        let hint = self.hinter.hint(line, pos, ctx);
        hint
    }
}

impl<'a, S> Highlighter for ExecHelper<'a, S> {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned(hint.black().on_green().to_string())
    }
}

impl<'a, S> Validator for ExecHelper<'a, S> {
    fn validate(
        &self,
        ctx: &mut validate::ValidationContext,
    ) -> rustyline::Result<validate::ValidationResult> {
        self.validator.validate(ctx)
    }

    fn validate_while_typing(&self) -> bool {
        self.validator.validate_while_typing()
    }
}

/// ExecCompleter enables command completion in the shell.
struct ExecCompleter<'a, S> {
    parser: Parser,
    cmds: Rc<RefCell<CommandSet<'a, S>>>,
    builtins: Rc<CommandSet<'a, Shell<'a, S>>>,
}

impl<'a, S> ExecCompleter<'a, S> {
    /// Constructs a new `ExecCompleter`.
    ///
    /// # Arguments
    /// `parser` - The parser to use for command completion.
    /// `cmds` - The custom commands to complete for.
    /// `builtins` - The builtins to complete for.
    fn new(
        parser: Parser,
        cmds: Rc<RefCell<CommandSet<'a, S>>>,
        builtins: Rc<CommandSet<'a, Shell<'a, S>>>,
    ) -> ExecCompleter<'a, S> {
        ExecCompleter {
            parser,
            cmds,
            builtins,
        }
    }

    /// Offers completion candidates for a line.
    ///
    /// Tries to mimic to some degree, the completion behavior in bash shells.
    ///
    /// In particular, this means that `pos` values that are not at the end of the line behave as
    /// if the portion of the line prior to it is the entirety of the line. e.g.:
    ///
    /// ```bash
    /// $ happ|iness
    ///       ^ Assuming this is the cursor position...
    /// $ happinessiness # Is the completion result.
    /// ```
    ///
    /// # Arguments
    /// `line` - The line to try offering completion candidates for.
    /// `pos` - The position of the cursor on that line.
    ///
    /// # Returns
    /// `rustyline::Result<(usize, Vec<Pair>)>` - A result of a position & completion results to
    /// present.
    fn complete(&self, line: &str, pos: usize) -> rustyline::Result<(usize, Vec<Pair>)> {
        // First, let's get the slice of the line leading up to the position, because really,
        // that's what we actually care about when trying to determine the completion.
        let partial = match line.get(..pos) {
            Some(p) => p,
            None => {
                // This shouldn't ever happen, as I believe `pos` should always be within bounds of
                // `line`. However, it doesn't hurt to be safe.
                return Ok((0, Vec::new()));
            }
        };

        // Now, try parsing what the user wants us to complete.
        let outcome = self
            .parser
            .parse(partial, &self.cmds.borrow(), &self.builtins);

        // If the parse was complete, then there is of course nothing to complete. It's...
        // complete.
        if outcome.complete {
            return Ok((pos, vec![]));
        }

        // The outcome includes what the parser would have allowed to have existed in the string.
        // Of these possibilities, some are better matches than others. Let's rank them as such by
        // finding those that share the first of the remaining tokens (or empty string if empty).
        let prefix = if let Some(first_token) = outcome.remaining.first() {
            first_token
        } else {
            // If the remaining is empty and we have an incomplete parse, that implies that the
            // user has thus far entered something valid but there are more subcommands to provide.
            // If the user then tabs to get a completion, it implies that they want to add a
            // subcommand. Before we can do that, we need a space delimiter, so that should be our
            // provided completion if it does not yet exist!
            //
            // ... with one gotcha. If the line is completely empty, we obviously should not expect
            // a space. The start of the line is itself a delimiter of sorts.
            if partial.is_empty() {
                ""
            } else if !partial.ends_with(" ") {
                // As said before, complete this as a space so that the next attempt at tab
                // completion gives the results the user likely actually wanted to see.
                return Ok((
                    pos,
                    vec![Pair {
                        display: String::from(" "),
                        replacement: String::from(" "),
                    }],
                ));
            } else {
                // Otherwise, the user already has the delimiter. So now we should provide any and
                // all subsequent subcommands.
                ""
            }
        };

        // So now, filter out those that have that aforementioned token as a prefix. And once we
        // have that, grab the suffix for completion.
        let candidates = outcome.possibilities.into_iter().filter_map(|poss| {
            if poss.starts_with(prefix) {
                // This really should never fail to get the remaining suffix, since the condition
                // guarantees that the prefix exists... but no harm in being safe if we can.
                poss.get(prefix.len()..).map(|s| s.to_string())
            } else {
                None
            }
        });

        // Finally, map the candidates to `Pair`'s, which is what the Completer interface wants.
        let pairs: Vec<Pair> = candidates
            .map(|candidate| Pair {
                display: candidate.to_string(),
                // Since we set our position of replacement to pos, we can just get away with
                // returning the suffix of the candidate to append from there.
                replacement: candidate,
            })
            .collect();

        Ok((pos, pairs))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod completions {
        use super::*;
        use crate::parser::test::make_parser_cmds;
        use crate::parser::Parser;

        use pretty_assertions::assert_eq;

        fn make_completer<'a>() -> ExecCompleter<'a, ()> {
            let (cmds, builtins) = make_parser_cmds();

            // Wrap these to satisfy the type checker.
            let cmds = Rc::new(RefCell::new(cmds));
            let builtins = Rc::new(builtins);

            ExecCompleter::new(Parser::new(), cmds, builtins)
        }

        fn test_completion<'a>(
            completer: ExecCompleter<'a, ()>,
            line: &str,
            pos: usize,
            expected_pairs: Vec<Pair>,
        ) {
            match completer.complete(line, pos) {
                Ok(cmpl_res) => {
                    let (cmpl_pos, pairs) = cmpl_res;
                    // We should always be returning a position that is the given position.
                    assert_eq!(cmpl_pos, pos, "mismatched positions");

                    assert_eq!(
                        pairs.len(),
                        expected_pairs.len(),
                        "mismatched number of completions"
                    );

                    for (p1, p2) in pairs.iter().zip(expected_pairs.iter()) {
                        assert_eq!(p1.display, p2.display, "non-matching display strings");
                        assert_eq!(
                            p1.replacement, p2.replacement,
                            "non-matching replacement strings"
                        );
                    }
                }
                Err(err) => {
                    panic!(format!("failed to complete '{}': {}", line, err))
                }
            }
        }

        #[test]
        fn simple() {
            let completer = make_completer();

            let line = "grau";

            test_completion(
                completer,
                line,
                line.len(),
                vec![Pair {
                    display: "lt-c".to_string(),
                    replacement: "lt-c".to_string(),
                }],
            )
        }

        #[test]
        fn no_matches() {
            let completer = make_completer();

            let line = "idontexistlol";

            test_completion(completer, line, line.len(), vec![])
        }

        #[test]
        fn multiple_matches() {
            let completer = make_completer();

            let line = "conflict-";

            test_completion(
                completer,
                line,
                line.len(),
                vec![
                    Pair {
                        display: "builtin-longer-match-but-still-loses".to_string(),
                        replacement: "builtin-longer-match-but-still-loses".to_string(),
                    },
                    Pair {
                        display: "custom-wins".to_string(),
                        replacement: "custom-wins".to_string(),
                    },
                    Pair {
                        display: "tie".to_string(),
                        replacement: "tie".to_string(),
                    },
                ],
            )
        }

        #[test]
        fn nested() {
            let completer = make_completer();

            let line = "foo-c qu";

            test_completion(
                completer,
                line,
                line.len(),
                vec![Pair {
                    display: "x-c".to_string(),
                    replacement: "x-c".to_string(),
                }],
            )
        }

        #[test]
        fn already_completed() {
            let completer = make_completer();

            let line = "foo-c qux-c quux-c";

            test_completion(completer, line, line.len(), vec![])
        }

        #[test]
        fn completely_blank_for_last_command() {
            let completer = make_completer();

            let line = "foo-c qux-c ";

            test_completion(
                completer,
                line,
                line.len(),
                vec![
                    Pair {
                        display: "corge-c".to_string(),
                        replacement: "corge-c".to_string(),
                    },
                    Pair {
                        display: "quux-c".to_string(),
                        replacement: "quux-c".to_string(),
                    },
                ],
            )
        }

        #[test]
        fn completion_includes_a_space() {
            let completer = make_completer();

            let line = "foo-c qux-c";

            test_completion(
                completer,
                line,
                line.len(),
                vec![Pair {
                    display: " ".to_string(),
                    replacement: " ".to_string(),
                }],
            )
        }

        #[test]
        fn nothing_typed() {
            let completer = make_completer();

            let line = "";

            test_completion(
                completer,
                line,
                line.len(),
                vec![
                    Pair {
                        display: "conflict-builtin-longer-match-but-still-loses".to_string(),
                        replacement: "conflict-builtin-longer-match-but-still-loses".to_string(),
                    },
                    Pair {
                        display: "conflict-custom-wins".to_string(),
                        replacement: "conflict-custom-wins".to_string(),
                    },
                    Pair {
                        display: "conflict-tie".to_string(),
                        replacement: "conflict-tie".to_string(),
                    },
                    Pair {
                        display: "foo-c".to_string(),
                        replacement: "foo-c".to_string(),
                    },
                    Pair {
                        display: "grault-c".to_string(),
                        replacement: "grault-c".to_string(),
                    },
                ],
            )
        }

        #[test]
        fn non_end_pos() {
            let completer = make_completer();

            let line = "grault-c";

            test_completion(
                completer,
                line, // 'grault-c'
                3,    //     ^
                vec![Pair {
                    display: "ult-c".to_string(),
                    replacement: "ult-c".to_string(),
                }],
            )
        }

        #[test]
        fn nested_non_end_pos() {
            let completer = make_completer();

            let line = "foo-c qux-c quux-c";

            test_completion(
                completer,
                line, // 'foo-c qux-c quux-c'
                8,    //          ^
                vec![Pair {
                    display: "x-c".to_string(),
                    replacement: "x-c".to_string(),
                }],
            )
        }
    }
}
