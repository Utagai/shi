use std::borrow::Cow::{self, Borrowed, Owned};
use std::path::Path;

use anyhow::{bail, Result};

use rustyline::completion::{Completer, Pair};
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{self, MatchingBracketValidator, Validator};
use rustyline::{Config, Context, Editor};
use rustyline_derive::Helper;

use colored::*;

pub struct Readline {
    rl: Editor<ExecHelper>,
}

impl Readline {
    pub fn new() -> Readline {
        let config = Config::builder()
            .completion_type(rustyline::CompletionType::List)
            .build();
        let mut rl = Editor::with_config(config);
        rl.set_helper(Some(ExecHelper::new()));
        Readline { rl }
    }

    pub fn load_history<P: AsRef<Path> + ?Sized>(&mut self, path: &P) -> Result<()> {
        self.rl.load_history(path)?;
        Ok(())
    }

    pub fn save_history<P: AsRef<Path> + ?Sized>(&mut self, path: &P) -> Result<()> {
        self.rl.save_history(path)?;
        Ok(())
    }

    pub fn add_history_entry<S: AsRef<str> + Into<String>>(&mut self, line: S) -> bool {
        self.rl.add_history_entry(line)
    }

    pub fn readline(&mut self, prompt: &str) -> rustyline::Result<String> {
        self.rl.readline(prompt)
    }

    pub fn history(&self) -> &rustyline::history::History {
        self.rl.history()
    }
}

#[derive(Helper)]
pub struct ExecHelper {
    completer: ExecCompleter,
    highlighter: MatchingBracketHighlighter,
    validator: MatchingBracketValidator,
    hinter: HistoryHinter,
    colored_prompt: String,
}

impl ExecHelper {
    fn new() -> ExecHelper {
        ExecHelper {
            completer: ExecCompleter::new(),
            highlighter: MatchingBracketHighlighter::new(),
            validator: MatchingBracketValidator::new(),
            hinter: HistoryHinter {},
            colored_prompt: "| ".to_string(),
        }
    }
}

impl Completer for ExecHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        println!("COMPLETE()");
        self.completer.complete(line, pos)
    }
}

impl Hinter for ExecHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        let hint = self.hinter.hint(line, pos, ctx);
        // println!("Hint: {:?}", hint);
        hint
    }
}

impl Highlighter for ExecHelper {
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

impl Validator for ExecHelper {
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

struct ExecCompleter {}

impl ExecCompleter {
    fn new() -> ExecCompleter {
        ExecCompleter {}
    }

    fn complete(&self, line: &str, pos: usize) -> rustyline::Result<(usize, Vec<Pair>)> {
        Ok((0, Vec::new()))
    }
}

impl Default for ExecCompleter {
    fn default() -> Self {
        Self::new()
    }
}
