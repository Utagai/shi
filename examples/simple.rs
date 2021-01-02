use shi::command::{BasicCommand, Command};
use shi::shell::Shell;

use anyhow::Result;

fn main() -> Result<()> {
    let mut shell = Shell::new("| ");

    shell.register(Command::new_child(BasicCommand::new("dog", |_, _| {
        Ok(String::from("woof"))
    })))?;
    shell.register(Command::new_parent(
        "felid",
        vec![
            Command::new_child(BasicCommand::new("panther", |_, _| {
                Ok(String::from("uhh what sound does a panther make"))
            })),
            Command::new_parent(
                "felinae",
                vec![
                    Command::new_child(BasicCommand::new("domestic-cat", |_, _| {
                        Ok(String::from("meow"))
                    })),
                    Command::new_child(BasicCommand::new("tiger", |_, _| Ok(String::from("rawr")))),
                ],
            ),
        ],
    ))?;

    shell.run()?;

    Ok(())
}
