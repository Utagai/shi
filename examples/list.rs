use shi::shell::Shell;
use shi::{cmd, parent};

use anyhow::Result;

fn main() -> Result<()> {
    let lst: Vec<String> = Vec::new();

    let mut shell = Shell::new_with_state("| ", lst)?;

    shell.register(cmd!(
        "pop",
        "pops the last item from the list",
        |the_lst: &mut Vec<String>, _| {
            let _ = the_lst.pop();
            Ok(String::from("popped last item"))
        }
    ))?;
    shell.register(cmd!(
        "list",
        "shows the list",
        |the_lst: &mut Vec<String>, _| {
            Ok(format!(
                "Current: [{}]",
                the_lst
                    .iter()
                    .map(|f| format!("{:?}", f))
                    .collect::<Vec<String>>()
                    .join(", ")
            ))
        }
    ))?;
    shell.register(parent!(
        "add",
        "adds something to the list",
        cmd!(
            "title",
            "adds 'title' to the list",
            |the_lst: &mut Vec<String>, _| {
                the_lst.push("title".to_owned());
                Ok(String::from("Added 'title'"))
            },
        ),
        parent!(
            "isbn",
            "adds a country to the list",
            cmd!(
                "eu",
                "adds 'eu' to the list",
                |the_lst: &mut Vec<String>, _| {
                    the_lst.push("eu".to_owned());
                    Ok(String::from("Added 'eu'"))
                },
            ),
            cmd!(
                "us",
                "adds 'us' to the list",
                |the_lst: &mut Vec<String>, _| {
                    the_lst.push("us".to_owned());
                    Ok(String::from("Added 'us'"))
                }
            ),
        ),
    ))?;

    shell.run()?;

    Ok(())
}
