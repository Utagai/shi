use std::collections::{hash_map::Iter, HashMap};

use crate::command::Command;

/// A wrapper data structure that offers retrieval, insertion, contains and len methods, specifically
/// for Commands.
pub struct CommandSet<'a, S> {
    cmds: HashMap<String, Box<dyn Command<State = S> + 'a>>,
}

impl<'a, S> CommandSet<'a, S> {
    pub fn new() -> Self {
        CommandSet {
            cmds: HashMap::new(),
        }
    }
}

impl<'a, S> CommandSet<'a, S> {
    pub fn get(&self, name: &str) -> Option<&Box<dyn Command<State = S> + 'a>> {
        self.cmds.get(name)
    }

    pub fn add<C>(&mut self, cmd: C)
    where
        C: Command<State = S> + 'a,
    {
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
    iter: Iter<'a, String, Box<dyn Command<State = S> + 'a>>,
}

impl<'a, S> Iterator for CommandSetIterator<'a, S> {
    type Item = &'a Box<dyn Command<State = S> + 'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(_, v)| v)
    }
}

impl<'a, S: 'a> IntoIterator for &'a CommandSet<'a, S> {
    type Item = &'a Box<dyn Command<State = S> + 'a>;
    type IntoIter = CommandSetIterator<'a, S>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
