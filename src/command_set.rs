use std::collections::{hash_map::Iter, HashMap};

use crate::command::{BaseCommand, Command};

/// A wrapper data structure that offers retrieval, insertion, contains and len methods, specifically
/// for Commands.
pub struct CommandSet<'a, S> {
    cmds: HashMap<String, Box<Command<'a, S>>>,
}

impl<'a, S> CommandSet<'a, S> {
    pub fn new() -> Self {
        CommandSet {
            cmds: HashMap::new(),
        }
    }

    pub fn new_from_vec(cmds: Vec<Command<'a, S>>) -> Self {
        let mut cmd_set = CommandSet::new();
        for cmd in cmds {
            cmd_set.add(cmd);
        }

        cmd_set
    }
}

impl<'a, S> CommandSet<'a, S> {
    pub fn get(&self, name: &str) -> Option<&Box<Command<'a, S>>> {
        self.cmds.get(name)
    }

    pub fn add(&mut self, cmd: Command<'a, S>) {
        self.cmds.insert(cmd.name().to_owned(), Box::new(cmd));
    }

    pub fn contains(&self, name: &str) -> bool {
        self.cmds.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.cmds.len()
    }

    pub fn iter(&self) -> CommandSetIterator<S> {
        CommandSetIterator {
            iter: self.cmds.iter(),
        }
    }
}

pub struct CommandSetIterator<'a, S> {
    iter: Iter<'a, String, Box<Command<'a, S>>>,
}

impl<'a, S> Iterator for CommandSetIterator<'a, S> {
    type Item = &'a Box<Command<'a, S>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(_, v)| v)
    }
}

impl<'a, S: 'a> IntoIterator for &'a CommandSet<'a, S> {
    type Item = &'a Box<Command<'a, S>>;
    type IntoIter = CommandSetIterator<'a, S>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
