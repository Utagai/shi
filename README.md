<p align="center"><img src="./rsrc/banner/shi.png"></img></p>

# shi
![Rust](https://github.com/Utagai/shi/workflows/Rust/badge.svg)

shi is a library for crafting ***sh***ell ***i***nterfaces in Rust.

**WIP**.

## Example
This is a pretty simple example. It uses no state, and has only one level of nesting. The actual Rust code for this can be found in `./examples/simple.rs`.
```rust
use shi::command::{BasicCommand, Command};
use shi::shell::Shell;

use anyhow::Result;

fn main() -> Result<()> {
    let mut shell = Shell::new("| ");

    shell.register(Command::new_leaf(BasicCommand::new("dog", |_, _| {
        Ok(String::from("woof"))
    })))?;
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
                    Command::new_leaf(BasicCommand::new("tiger", |_, _| Ok(String::from("rawr")))),
                ],
            ),
        ],
    ))?;

    shell.run()?;

    Ok(())
}
```
Here's some output from the above snippet:
```
| help
Normal commands:
        'dog' - 'dog'
        'felid' - 'felid'
Built-in commands:
        'history' - 'history'
        'exit' - 'exit'
        'helptree' - 'helptree'
        'help' - 'help'
| helptree
Normal commands
├── dog
└── felid
    ├── panther
    └── felinae
        ├── tiger
        └── domestic-cat


Builtins
├── history
├── exit
├── helptree
└── help
| dog
woof
| felid panther
uhh what sound does a panther make
| felid DNE
Failed to parse fully:

            (spaces trimmed)
         => 'felid DNE'
                   ^
expected a valid subcommand
instead, got: 'DNE';

         => expected one of ["panther", "felinae"], got DNE

Run 'felid help' for more info on the command.
Run 'helptree' for more info on the entire command tree.

| felid felinae domestic-cat
meow
| exit
bye
```

## Contributing
This is my first Rust crate, so I welcome any and all feedback from those more experienced.
There is no process for this though. Just open an issue or a PR. :slightly_smiling_face:
