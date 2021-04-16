<p align="center"><img src="./rsrc/banner/shi.png"></img></p>

# shi

![Rust](https://github.com/Utagai/shi/workflows/Rust/badge.svg)

shi is a library for crafting <b>_sh_</b>ell <b>_i_</b>nterfaces in Rust.

shi is built on top of the excellent
[`rustyline`](https://github.com/kkawakam/rustyline). It adds abstractions for
supporting commands, parsing them and supporting dynamic autocompletion with
zero work.

## What's Left

There's a few things here and there that would be nice to do. Most of this is
code clean-up and quality of life improvements, rather than features. For
example, I only recently learned about the [Rust API
Guidelines](https://rust-lang.github.io/api-guidelines/checklist.html). I'm
probably breaking some rules there that are worth fixing.

I'm likely the only person using his crate, so I won't know what other features
or changes would be nice to have for people. Feel free to suggest some (or put
up a PR).

`shi` is used in some of my personal projects, which I'll link here when
they're ready. Otherwise, it's ready to use, though it is not stable.

## Example

This is a pretty simple example. It uses no state, and has only one level of nesting. The actual Rust code for this can be found in `./examples/simple.rs`.

```rust
use shi::shell::Shell;
use shi::{cmd, parent};

use anyhow::Result;

fn main() -> Result<()> {
    let mut shell = Shell::new("| ");

    shell.register(cmd!("dog", |_, _| { Ok(String::from("woof")) }))?;
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
