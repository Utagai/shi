use std::marker::PhantomData;

use anyhow::{bail, Result};

use super::Command;
use crate::shell::Shell;

#[derive(Debug)]
pub struct HelpTreeCommand<'a, S> {
    // TODO: Not sure if we need this crap.
    phantom: &'a PhantomData<S>,
}

#[derive(Clone)]
struct IndentContext {
    last: bool,
    // This is a mouthful, but the idea is that if(parent_lastness_chain[i]) implies that parent_i was
    // the last item in the level it belonged too. This is necessary to know when we need to
    // continue a verticle pipe.
    parent_lastness_chain: Vec<bool>,
}

impl IndentContext {
    fn indent(&self, last: bool) -> Self {
        let mut parent_chain_copy = self.parent_lastness_chain.to_vec();
        parent_chain_copy.push(last);
        IndentContext {
            last,
            parent_lastness_chain: parent_chain_copy,
        }
    }

    fn with_last(&self, new_last: bool) -> Self {
        IndentContext {
            last: new_last,
            parent_lastness_chain: self.parent_lastness_chain.to_vec(),
        }
    }
}

impl<'a, S> HelpTreeCommand<'a, S> {
    pub fn new() -> HelpTreeCommand<'a, S> {
        HelpTreeCommand {
            phantom: &PhantomData,
        }
    }

    // TODO: This works, but it isn't designed in the best way possible. What we should be doing is
    // taking the commands and iterating them and their children into a tree. Then, we should pass
    // the tree of strings (or, whatever type holding the information we want to print) to a
    // function like this, responsible for rendering the tree.
    // Right now, for example, there isn't any way to test this code without creating a shell,
    // which is a code smell.
    fn add_name_to_lines(&self, ctx: &IndentContext, lines: &mut Vec<String>, name: &str) {
        let mut line_elems: Vec<&str> = Vec::new();
        for parent_was_last in &ctx.parent_lastness_chain {
            if *parent_was_last {
                // If the parent was the last in the chain, we don't need to continue its vertical
                // pipe, because it will have a clean elbow cut-off.
                line_elems.push("    ");
            } else {
                line_elems.push("│   ");
            }
        }

        // If we're the last guy, we want a clean elbow cut-off, otherwise, we want a fork.
        if ctx.last {
            line_elems.push("└");
        } else {
            line_elems.push("├");
        }

        // Write two horizontal pipes to lead to our name, with a space for separation..
        let dash_name = format!("── {}", name);
        line_elems.push(&dash_name);

        lines.push(line_elems.join(""))
    }

    fn add_tree_lines_for_children<T>(
        &self,
        ctx: &IndentContext,
        lines: &mut Vec<String>,
        cmds: Vec<&dyn Command<State = T>>,
    ) {
        for (i, cmd) in cmds.iter().enumerate() {
            let last = i == cmds.len() - 1;
            self.add_name_to_lines(&ctx.with_last(last), lines, cmd.name());
            if cmd.sub_commands().len() != 0 {
                // We need to recurse another level for our children.
                self.add_tree_lines_for_children(&ctx.indent(last), lines, cmd.sub_commands());
            }
        }
    }

    fn get_tree_lines(
        &self,
        normal_cmds: &'a Vec<&Box<dyn Command<State = S> + 'a>>,
        builtin_cmds: &'a Vec<&Box<dyn Command<State = Shell<S>> + 'a>>,
    ) -> Vec<String> {
        // First, tackle the initial two subtrees, since they have slightly differing types.
        //  1: The normal commands.
        //  2: The builtins.
        //
        //  Really, we are solving the same problem, but due to the differing types we need to
        //  handle them 'manually' here, and then let recursion handle the rest.

        let ctx = IndentContext {
            last: false,
            parent_lastness_chain: Vec::new(),
        };
        let mut lines: Vec<String> = Vec::new();
        lines.push(String::from("Normal commands"));
        for (i, cmd) in normal_cmds.iter().enumerate() {
            let last = i == normal_cmds.len() - 1;
            self.add_name_to_lines(&ctx.with_last(last), &mut lines, cmd.name());
            if cmd.sub_commands().len() != 0 {
                self.add_tree_lines_for_children(&ctx.indent(last), &mut lines, cmd.sub_commands());
            }
        }

        lines.push(String::from("\n"));

        lines.push(String::from("Builtins"));
        for (i, cmd) in builtin_cmds.iter().enumerate() {
            let last = i == builtin_cmds.len() - 1;
            self.add_name_to_lines(&ctx.with_last(last), &mut lines, cmd.name());
            if cmd.sub_commands().len() != 0 {
                self.add_tree_lines_for_children(&ctx.indent(last), &mut lines, cmd.sub_commands());
            }
        }

        lines
    }

    fn to_lines(&self, shell: &Shell<'a, S>) -> Vec<String> {
        let normal_cmds_vec: Vec<&Box<dyn Command<State = S>>> = shell.cmds.values().collect();
        let builtin_cmds_vec: Vec<&Box<dyn Command<State = Shell<S>>>> =
            shell.builtins.values().collect();
        let help_lines: Vec<String> = self.get_tree_lines(&normal_cmds_vec, &builtin_cmds_vec);

        help_lines
    }
}

impl<'a, S> Command for HelpTreeCommand<'a, S> {
    type State = Shell<'a, S>;

    fn name(&self) -> &str {
        "helptree"
    }

    fn validate_args(&self, args: &Vec<String>) -> Result<()> {
        if args.len() != 0 {
            // TODO: We may want to make this actually take arguments, like a command name or
            // command name path.
            bail!("help takes no arguments")
        }

        Ok(())
    }

    fn execute(&self, shell: &mut Shell<'a, S>, _: &Vec<String>) -> Result<String> {
        let help_lines = self.to_lines(shell);

        Ok(help_lines.join("\n"))
    }
}