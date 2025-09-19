use shi::shell::Shell;
use shi::{cmd, parent};

use anyhow::Result;

fn main() -> Result<()> {
    // Instantiate a shell, with the prompt '| '.
    let mut shell = Shell::new("| ")?;

    // Register a basic leaf (no subcommands) command. It is called dog, and it prints out "woof"
    // when executed.
    shell.register(cmd!("dog", |_, _| { Ok(String::from("woof")) }))?;

    // Register a parent command. This means it has subcommands. It is called "felid", and expects
    // either the "panther" or "felinae" subcommands.
    shell.register(parent!(
        "felid",
        cmd!("panther", |_, _| {
            Ok(String::from("generic panther sound"))
        }),
        parent!(
            "felinae",
            cmd!("domestic-cat", |_, _| { Ok(String::from("meow")) }),
            cmd!("dangerous-tiger", |_, _| { Ok(String::from("roar")) }),
        )
    ))?;

    // Start the shell run loop. This will make the program begin its input-output loop.
    shell.run()?;

    Ok(())
}
