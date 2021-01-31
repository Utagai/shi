use shi::command::{BasicCommand, Command};
use shi::shell::Shell;

use anyhow::Result;

fn main() -> Result<()> {
    // Instantiate a shell, with the prompt '| '.
    let mut shell = Shell::new("| ");

    // Register a basic leaf (no subcommands) command. It is called dog, and it prints out "woof"
    // when executed.
    shell.register(Command::new_leaf(BasicCommand::new("dog", |_, _| {
        Ok(String::from("woof"))
    })))?;

    // Register a parent command. This means it has subcommands. It is called "felid", and expects
    // either the "panther" or "felinae" subcommands.
    shell.register(Command::new_parent(
        "felid",
        vec![
            Command::new_leaf(BasicCommand::new("panther", |_, _| {
                Ok(String::from("uhh what sound does a panther make"))
            })),
            Command::new_parent(
                "felinae",
                vec![
                    Command::new_leaf(BasicCommand::new("domestic-cat", |_, _| {
                        Ok(String::from("meow"))
                    })),
                    Command::new_leaf(BasicCommand::new("dangerous-tiger", |_, _| {
                        Ok(String::from("rawr"))
                    })),
                ],
            ),
        ],
    ))?;

    // Start the shell run loop. This will make the program begin its input-output loop.
    shell.run()?;

    Ok(())
}
