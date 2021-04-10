use std::marker::PhantomData;

use super::{BaseCommand, Command};
use crate::command_set::CommandSet;
use crate::error::ShiError;
use crate::shell::Shell;
use crate::Result;

#[derive(Debug)]
/// HelpTreeCommand prints out a prettified and more complete version of the HelpCommand. It prints
/// out a tree visualization of all the commands in its shell. In particular, it shows the
/// hierarchies of commands. The output of this command is inspired by the `tree` program.
///
/// An example of what this command may produce is:
/// ```plaintext
/// Normal commands
/// ├── dog
/// └── felid
///     ├── panther
///     ├── felinae
///     │    ├── dangerous-tiger
///     │    └── domestic-cat
///     └── felinae2
///          ├── dangerous-tiger
///          └── domestic-cat
///
///
/// Builtins
/// ├── helptree
/// ├── exit
/// ├── help
/// └── history
/// ```
pub struct HelpTreeCommand<'a, S> {
    // TODO: Not sure if we need this crap.
    phantom: &'a PhantomData<S>,
}

#[derive(Clone)]
/// A helper struct that records the context needed to determine how to correctly indent a line for
/// the helptree visualization. It includes two pieces of relevant information:
///
/// * Am I the last command of my level?
/// * Of all my ancestors, were _they_ the last command of _their_ level?
///
/// These two pieces of information allow us to correctly determine spacing and connectors needed
/// to produce the tree.
///
/// IndentContexts are produced by either _indenting_ them to a new level of recursion in the tree,
/// OR, by traversing to the next element in the same level. It's methods, `indent` and `with_last`
/// correspond to these two cases respectively. In other words, a tree can either get deeper or
/// wider, respectively.
struct IndentContext {
    last: bool,
    // This is a mouthful, but the idea is that if(parent_lastness_chain[i]) implies that parent_i was
    // the last item in the level it belonged too. This is necessary to know when we need to figure
    // out if we should continue a verticle pipe.
    parent_lastness_chain: Vec<bool>,
}

impl IndentContext {
    /// Produces a new IndentContext for the next indentation level (or, perhaps more accurately,
    /// next level of the tree, or, next recursion).
    fn indent(&self, last: bool) -> Self {
        // We don't want future IndentContexts to hold references to prior IndentContexts' parent
        // chains, since they should be different.
        // There may be a way to avoid the copy and hold onto slices of a larger chain, but I do
        // not think the addition in complexity is worth the negligible performance gain (if any).
        let mut parent_chain_copy = self.parent_lastness_chain.to_vec();
        parent_chain_copy.push(last);
        IndentContext {
            last,
            parent_lastness_chain: parent_chain_copy,
        }
    }

    /// Produces a new IndentContext, but does not indent it and therefore maintains the current
    /// level of the tree. Thus, it keeps the `parent_lastness_chain` the same. However, since a
    /// new IndentContext for a given level could be the _last_ element of that level, it takes an
    /// argument for denoting that.
    fn with_last(&self, new_last: bool) -> Self {
        IndentContext {
            last: new_last,
            parent_lastness_chain: self.parent_lastness_chain.to_vec(),
        }
    }
}

impl<'a, S> Default for HelpTreeCommand<'a, S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, S> HelpTreeCommand<'a, S> {
    /// Creates a new HelpTreeCommand.
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
    /// Adds the given name under the given indentation context to the given vector of strings,
    /// maintaining the appearance of a tree.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of where in the tree we are adding lines to.
    /// * `lines` - The lines of the helptree visualization. It is added to, and includes the
    /// entire tree by the end of this function.
    /// * `name` - The name of a command to add.
    fn add_name_to_lines(&self, ctx: &IndentContext, lines: &mut Vec<String>, name: &str) {
        // This is not to be confused with `lines`. Think of this as the columns; merging the
        // elements in this vector gives you a line, to be added to `lines`.
        let mut line_elems: Vec<&str> = Vec::new();

        // For each of the parents in our chain, if they were last, then we only want a space
        // because then their pipe is an elbow connector.
        //   └─ Foo
        //   │  └─ SubFoo <--- WRONG!
        // Instead we want:
        //   └─ Foo
        //      └─ SubFoo <--- RIGHT!
        // However, if they were _NOT_ last, then we want a vertical pipe, since their connector is
        // a 3-way connector. So we'd want that continuation.
        //   ├─ Foo
        //   │  └─ SubFoo <--- RIGHT!
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
        // NOTE: This is for the _current_ command. So this is not to be confused with what we are
        // doing above with the `parent_lastness_chain`.
        if ctx.last {
            line_elems.push("└");
        } else {
            line_elems.push("├");
        }

        // Write two horizontal pipes to lead to our name, with a space for separation...
        let dash_name = format!("── {}", name);
        line_elems.push(&dash_name);

        lines.push(line_elems.join(""))
    }

    /// Adds the lines of the helptree visualization.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context of where in the tree we are adding lines to.
    /// * `lines` - The lines of the helptree visualization. It is added to, and includes the
    /// entire tree by the end of this function.
    /// * `cmds` - The set of Commands for which to create and add lines of the helptree
    /// visualization.
    fn add_tree_lines_for_children<T>(
        &self,
        ctx: &IndentContext,
        lines: &mut Vec<String>,
        cmds: &CommandSet<T>,
    ) {
        for (i, cmd) in cmds.iter().enumerate() {
            let last = i == cmds.len() - 1;

            // Because we may recurse, we'll be going into a deeper level whose lines should come
            // _after_, so add the current command's line to the vector now.
            self.add_name_to_lines(&ctx.with_last(last), lines, cmd.name());

            match &**cmd {
                Command::Leaf(_) => continue, // We can't recurse in this case.
                Command::Parent(parent_cmd) => {
                    // We need to recurse another level for our children.
                    self.add_tree_lines_for_children(
                        &ctx.indent(last),
                        lines,
                        parent_cmd.sub_commands(),
                    );
                }
            }
        }
    }

    /// Produces the helptree representation of the given Shell's commands via a `Vec<String>`.
    ///
    /// # Arguments
    ///
    /// * `shell` - The shell for which to produce the helptree.
    fn to_lines(&self, shell: &Shell<'a, S>) -> Vec<String> {
        // We tackle the initial two subtrees separately, since they have slightly differing types.
        //  1: The normal commands (state = S).
        //  2: The builtins (state = Shell<S>).
        //
        //  Since they are different types, we need to invoke `add_tree_lines_for_children()`
        //  separately for each, and combine the resulting help lines.

        // Start with an initial context with the lastness chain being empty.
        // Of course, `last` should also be false, which we ensure with `.with_last(false)` in the
        // invocations to `add_tree_lines_for_children()` below.
        let ctx = IndentContext {
            last: false,
            parent_lastness_chain: Vec::new(),
        };

        let mut lines: Vec<String> = Vec::new();
        lines.push(String::from("Normal commands"));
        self.add_tree_lines_for_children(&ctx.with_last(false), &mut lines, &shell.cmds.borrow());

        lines.push(String::from("\n"));

        lines.push(String::from("Builtins"));
        self.add_tree_lines_for_children(&ctx.with_last(false), &mut lines, &shell.builtins);

        lines
    }
}

impl<'a, S> BaseCommand for HelpTreeCommand<'a, S> {
    type State = Shell<'a, S>;

    fn name(&self) -> &str {
        "helptree"
    }

    fn validate_args(&self, args: &[String]) -> Result<()> {
        if !args.is_empty() {
            // TODO: We may want to make this actually take arguments, like a command name or
            // command name path.
            return Err(ShiError::ExtraArgs { got: args.to_vec() });
        }

        Ok(())
    }

    fn execute(&self, shell: &mut Shell<'a, S>, _: &[String]) -> Result<String> {
        let help_lines = self.to_lines(shell);

        Ok(help_lines.join("\n"))
    }
}
