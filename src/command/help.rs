use std::marker::PhantomData;

use crate::command::{BaseCommand, Command};
use crate::command_set::CommandSet;
use crate::error::ShiError;
use crate::parser::CommandType;
use crate::shell::Shell;
use crate::Result;

#[derive(Debug)]
/// HelpCommand is a command for printing out a listing of all available commands and builtins.
///
/// It displays two separated sections, one for custom commands and one for builtins.
/// It assumes that all commands it prints have meaningful implementations of Help(), as it
/// includes it in the output.
pub struct HelpCommand<'a, S> {
    // TODO: Not sure if we need this crap.
    _phantom: &'a PhantomData<S>,
}

impl<'a, S> Default for HelpCommand<'a, S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, S> HelpCommand<'a, S> {
    /// Creates a new HelpCommand.
    pub fn new() -> HelpCommand<'a, S> {
        HelpCommand {
            _phantom: &PhantomData,
        }
    }

    fn execute_no_args(&self, shell: &mut Shell<S>) -> String {
        // We expect there to be one line per command, +2 commands for headers of the two sections.
        let mut help_lines: Vec<String> =
            Vec::with_capacity(shell.cmds.borrow().len() + shell.builtins.len() + 2);
        help_lines.push(String::from("Normal commands:"));
        for cmd in shell.cmds.borrow().iter() {
            help_lines.push(format!("\t'{}' - {}", cmd.name(), cmd.help()));
        }

        help_lines.push(String::from("Built-in commands:"));
        for builtin in shell.builtins.iter() {
            help_lines.push(format!("\t'{}' - {}", builtin.name(), builtin.help()))
        }

        help_lines.join("\n")
    }

    fn help_breakdown<T>(
        &self,
        cmd_path: Vec<&str>,
        invocation_args: Vec<&str>,
        cmds: &CommandSet<T>,
    ) -> Result<String> {
        // We expect cmd_path.len() number of lines, one per segment, with potential for an extra
        // line for the command args. Let's request the maximum.
        let mut lines = Vec::with_capacity(cmd_path.len() + 1);
        let mut current_cmds = cmds;
        for (indent, segment) in cmd_path.iter().enumerate() {
            match current_cmds.get(segment) {
                Some(cmd) => {
                    let cmd_name = cmd.name();
                    let help_msg = cmd.help();
                    lines.push(format!(
                        "{}└─ {} - {}",
                        "   ".repeat(indent), // Use two spaces since we have 2 pipe-characters & a space.
                        cmd_name,
                        help_msg
                    ));
                    match &**cmd {
                        Command::Parent(parent) => current_cmds = parent.sub_commands(),
                        Command::Leaf(_) => {
                            let mut called_with_msg =
                                format!("Called with args: [{}]", invocation_args.join(", "));
                            if invocation_args.is_empty() {
                                called_with_msg = String::from("Called with no args")
                            }
                            lines.push(format!(
                                "{}└─ {}",
                                // Use two spaces since we have 2 pipe-characters & a space.
                                "   ".repeat(indent + 1),
                                called_with_msg,
                            ));
                        }
                    };
                }
                None => {
                    return Err(ShiError::UnrecognizedCommand {
                        got: segment.to_string(),
                    })
                }
            }
        }

        Ok(lines.join("\n"))
    }

    fn execute_with_args(&self, shell: &mut Shell<S>, args: &[String]) -> Result<String> {
        let invocation = args.join(" ");
        let outcome = shell.parse(&invocation);

        // Now that we've parsed the args as a command invocation, we can offer a detailed help
        // break down for the command path:
        match outcome.cmd_type {
            CommandType::Custom => {
                self.help_breakdown(outcome.cmd_path, outcome.remaining, &shell.cmds.borrow())
            }
            CommandType::Builtin => {
                self.help_breakdown(outcome.cmd_path, outcome.remaining, &shell.builtins)
            }
            CommandType::Unknown => Err(outcome
                .error()
                .expect("unknown command type, but could not produce error")),
        }
    }
}

impl<'a, S> BaseCommand for HelpCommand<'a, S> {
    type State = Shell<'a, S>;

    fn name(&self) -> &str {
        "help"
    }

    fn validate_args(&self, _: &[String]) -> Result<()> {
        Ok(())
    }

    fn execute(&self, shell: &mut Shell<S>, args: &[String]) -> Result<String> {
        if args.is_empty() {
            Ok(self.execute_no_args(shell))
        } else {
            self.execute_with_args(shell, args)
        }
    }

    fn help(&self) -> String {
        String::from("Prints help info for root commands or explains a given command invocation")
    }
}

#[cfg(test)]
mod test {
    use super::HelpCommand;
    use crate::command::BaseCommand;
    use crate::shell::Shell;
    use crate::Result;
    use crate::{leaf, parent};
    use pretty_assertions::assert_eq;
    use std::marker::PhantomData;

    #[derive(Debug)]
    struct TestCommand<'a, S> {
        name: &'a str,
        help: &'a str,
        phantom: PhantomData<S>,
    }

    impl<'a, S> TestCommand<'a, S> {
        fn new(name: &'a str, help: &'a str) -> TestCommand<'a, S> {
            TestCommand {
                name,
                help,
                phantom: PhantomData,
            }
        }
    }

    impl<'a, S> BaseCommand for TestCommand<'a, S> {
        type State = S;

        fn name(&self) -> &str {
            self.name
        }

        fn validate_args(&self, _: &[String]) -> Result<()> {
            Ok(())
        }

        fn execute(&self, _: &mut S, _: &[String]) -> Result<String> {
            Ok(String::from(""))
        }

        fn help(&self) -> String {
            self.help.to_string()
        }
    }

    fn run_help_test(args: Vec<String>, expected: String) -> Result<()> {
        // TODO: Do we really need to make a shell to test this? Is this a code-smell?
        let mut shell = Shell::new("")?;
        shell.register(leaf!(TestCommand::new("leaf", "1")))?;
        shell.register(parent!(
            "foo",
            "2",
            leaf!(TestCommand::new("bar", "2.1")),
            leaf!(TestCommand::new("baz", "2.2")),
            parent!(
                "qux",
                "2.3",
                leaf!(TestCommand::new("quuz", "2.3.1")),
                leaf!(TestCommand::new("corge", "2.3.2")),
            ),
            leaf!(TestCommand::new("quux", "2.4")),
        ))?;

        verify_help_output(&mut shell, args, expected);

        Ok(())
    }

    fn run_help_test_no_cmds(args: Vec<String>, expected: String) -> Result<()> {
        // TODO: Do we really need to make a shell to test this? Is this a code-smell?
        let mut shell = Shell::new("")?;

        verify_help_output(&mut shell, args, expected);

        Ok(())
    }

    fn verify_help_output(shell: &mut Shell<()>, args: Vec<String>, expected: String) {
        let help_cmd = HelpCommand::new();
        match help_cmd.execute(shell, &args) {
            Ok(help_output) => {
                println!("{}", help_output);
                assert_eq!(help_output, expected);
            }
            Err(err) => {
                assert_eq!(format!("{}", err), expected)
            }
        };
    }

    #[test]
    fn help_with_no_args_gives_list() -> Result<()> {
        run_help_test(
            vec![],
            String::from(
                "\
        Normal commands:\n\t\
            \'leaf\' - 1\n\t\
            \'foo\' - 2\n\
        Built-in commands:\n\t\
            \'help\' - Prints help info for root commands or explains a given command invocation\n\t\
            \'helptree\' - Prints a tree depiction of all commands in this shell\n\t\
            \'exit\' - Exits the shell session\n\t\
            \'history\' - Prints the history of commands",
            ),
        )
    }

    #[test]
    fn help_with_no_args_and_no_cmds() {
        run_help_test_no_cmds(
            vec![],
            String::from(
                "\
                Normal commands:\n\
                Built-in commands:\n\t\
                    \'help\' - Prints help info for root commands or explains a given command invocation\n\t\
                    \'helptree\' - Prints a tree depiction of all commands in this shell\n\t\
                    \'exit\' - Exits the shell session\n\t\
                    \'history\' - Prints the history of commands\
            "),
        ).expect("Failed to run test for help with no cmds")
    }

    // NOTE: In some of the tests below, we can't use escaped multi-line strings because the escape
    // removes the spacing that creates the tree-like structure.
    #[test]
    fn help_on_root_leaf_cmd() -> Result<()> {
        run_help_test(
            vec![String::from("leaf")],
            String::from("└─ leaf - 1\n   └─ Called with no args"),
        )
    }

    #[test]
    fn help_on_root_parent_cmd() -> Result<()> {
        run_help_test(vec![String::from("foo")], String::from("└─ foo - 2"))
    }

    #[test]
    fn help_on_depth_2() -> Result<()> {
        run_help_test(
            vec![String::from("foo"), String::from("bar")],
            String::from("└─ foo - 2\n   └─ bar - 2.1\n      └─ Called with no args"),
        )
    }

    #[test]
    fn help_on_depth_3() -> Result<()> {
        run_help_test(
            vec![
                String::from("foo"),
                String::from("qux"),
                String::from("quuz"),
            ],
            String::from(
                "└─ foo - 2\n   └─ qux - 2.3\n      └─ quuz - 2.3.1\n         └─ Called with no args",
            ),
        )
    }

    #[test]
    fn help_on_depth_2_with_1_leaf_arg() -> Result<()> {
        run_help_test(
            vec![
                String::from("foo"),
                String::from("bar"),
                String::from("hello"),
            ],
            String::from("└─ foo - 2\n   └─ bar - 2.1\n      └─ Called with args: [hello]"),
        )
    }

    #[test]
    fn help_on_depth_2_with_2_leaf_args() -> Result<()> {
        run_help_test(
            vec![
                String::from("foo"),
                String::from("bar"),
                String::from("hello"),
                String::from("world"),
            ],
            String::from("└─ foo - 2\n   └─ bar - 2.1\n      └─ Called with args: [hello, world]"),
        )
    }

    #[test]
    fn invalid_command_invocation() -> Result<()> {
        run_help_test(
            vec![String::from("DNE")],
            r#"command failed to parse: 'DNE' is not a recognized command.
                        @
                        @	 => expected one of 'leaf' or 'foo'.
                        @
                        Run 'helptree' for more info on the entire command tree.
                        @"#
            .replace("@", "")
            .replace("                        ", ""),
        )
    }
}
