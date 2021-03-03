<p align="center"><img src="./rsrc/banner/shi.png"></img></p>

# shi
![Rust](https://github.com/Utagai/shi/workflows/Rust/badge.svg)

shi is a library for crafting ***sh***ell ***i***nterfaces in Rust.

**WIP**.

## What's Left
Currently, `shi` is actually usable. The majority of remaining work involves quality of life and UX improvements, both for the shell interface itself, as well as the API. Currently, I'm using it in two personal projects of mine, which I'll be sure to link in this README when they are ready. Here's a (not comprehensive) list of things I would like to still do for `shi`:

* Upload to [crates.io](https://crates.io/) and linking to [docs.rs](https://docs.rs/).
* Implement multi-line input (via `\`).
* Add support for flags (named arguments) to commands.
* Flesh out the `help` of commands in `shi`.
* Switch to using `thiserror` instead of `anyhow`, since this is not a binary.

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
                    Command::new_leaf(BasicCommand::new("dangerous-tiger", |_, _| Ok(String::from("rawr")))),
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
        ├── dangerous-tiger
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

Run 'felid help' for more info on the command.

         => expected one of 'felinae' or 'panther'.

Run 'helptree' for more info on the entire command tree.

| felid felinae domestic-cat
meow
| exit
bye
```

## Contributing
This is my first Rust crate, so I welcome any and all feedback, even from fellow newbies.
There is no process for this though. Just open an issue or a PR. :slightly_smiling_face:
