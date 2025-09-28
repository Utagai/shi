use shi::command::EchoCommand;
use shi::leaf;
use shi::shell::Shell;

use anyhow::Result;

fn main() -> Result<()> {
    // Instantiate a shell, with the prompt '| '.
    let mut shell = Shell::new("| ")?;

    shell.register(leaf!(EchoCommand::new()))?;

    // Start the shell run loop. This will make the program begin its input-output loop.
    shell.run()?;

    Ok(())
}
