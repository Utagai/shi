use shi::command::{Command, EchoCommand};
use shi::shell::Shell;

use anyhow::Result;

fn main() -> Result<()> {
    // Instantiate a shell, with the prompt '| '.
    let mut shell = Shell::new("| ");

    shell.register(Command::new_leaf(EchoCommand::new()))?;

    // Start the shell run loop. This will make the program begin its input-output loop.
    shell.run()?;

    Ok(())
}
