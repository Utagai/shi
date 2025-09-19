use shi::shell::Shell;
use shi::{cmd, parent};

use anyhow::Result;

fn main() -> Result<()> {
    println!("configure cli functions");
    let mut shell = Shell::new("| ")?;
    shell.register(parent!(
        "server",
        cmd!("listen", "Start listening on the given port", |_, args| {
            println!("hello world start: {:?}", args);
            Ok("start".to_string())
        }),
        cmd!("unlisten", "stop listening", |_, args| {
            println!("hello world stop: {:?}", args);
            Ok("stop".to_string())
        })
    ))?;
    shell.run()?;
    Ok(())
}
