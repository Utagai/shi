//! A crate for building shell interfaces in Rust.
//!
//! See the README.md and examples for more information.

use std::result;

pub mod command;
mod command_set;
pub mod error;
mod parser;
mod readline;
pub mod shell;
mod tokenizer;

pub type Result<T> = result::Result<T, error::ShiError>;

/// Creates a parent command that has child subcommands underneath it.
#[macro_export]
macro_rules! parent {
    ( $name:expr, $help:literal, $( $x:expr ),* $(,)? ) => {
        {
            $crate::command::Command::Parent(
                $crate::command::ParentCommand::new_with_help(
                    $name,
                    $help,
                    vec![
                    $(
                        $x,
                    )*
                    ],
                )
            )
        }
    };
    ( $name:expr, $( $x:expr ),* $(,)? ) => {
        {
            $crate::command::Command::new_parent(
                $name,
                vec![
                $(
                    $x,
                )*
                ],
            )
        }
    };
}

/// Creates a leaf command from a given Command.
#[macro_export]
macro_rules! leaf {
    ( $cmd:expr ) => {
        $crate::command::Command::new_leaf($cmd)
    };
}

/// Creates a leaf command from the given name and closure.
#[macro_export]
macro_rules! cmd {
    ( $name:expr, $exec:expr ) => {
        $crate::leaf!($crate::command::BasicCommand::new($name, $exec))
    };
    ( $name:expr, $help:literal, $exec:expr ) => {
        $crate::leaf!($crate::command::BasicCommand::new_with_help(
            $name, $help, $exec
        ))
    };
}
