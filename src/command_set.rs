use std::collections::{hash_map::Iter, HashMap};

use crate::command::{BaseCommand, Command};

/// A wrapper data structure that offers several basic container methods, specifically for
/// Commands.
pub struct CommandSet<'a, S> {
    cmds: HashMap<String, Box<Command<'a, S>>>,
}

impl<'a, S> CommandSet<'a, S> {
    /// Creates a new, empty `CommandSet`.
    pub fn new() -> Self {
        CommandSet {
            cmds: HashMap::new(),
        }
    }

    /// Creates a new `CommandSet` from the given `Vector` of `Command`'s.
    pub fn new_from_vec(cmds: Vec<Command<'a, S>>) -> Self {
        let mut cmd_set = CommandSet::new();
        for cmd in cmds {
            cmd_set.add(cmd);
        }

        cmd_set
    }
}

impl<'a, S> CommandSet<'a, S> {
    /// Retrieves the command, if one exists, for the given name.
    ///
    /// # Arguments
    /// `name` - The name of the command to retrieve.
    ///
    /// # Returns
    /// `Option<&Box<Command>>` - The command with the name requested, or None if it was not found.
    pub fn get(&self, name: &str) -> Option<&Box<Command<'a, S>>> {
        self.cmds.get(name)
    }

    /// Adds the given command to the set.
    ///
    /// # Arguments
    /// `cmd` - The command to add to this set.
    pub fn add(&mut self, cmd: Command<'a, S>) {
        self.cmds.insert(cmd.name().to_owned(), Box::new(cmd));
    }

    /// Tests for existence of a `Command` with the given `name`.
    ///
    /// # Arguments
    /// `name` - The name to look for in this `CommandSet`.
    ///
    /// # Returns
    /// `bool` - Whether or not a `Command` with the given `name` exists in this set.
    pub fn contains(&self, name: &str) -> bool {
        self.cmds.contains_key(name)
    }

    /// Returns the length of this `CommandSet`.
    ///
    /// # Returns
    /// `usize` - The length of this `CommandSet`.
    pub fn len(&self) -> usize {
        self.cmds.len()
    }

    /// Retrievse the command names of this command set.
    /// Note that this only includes the names at the topmost/root level, it does not potentially
    /// recurse into parent commands and flatten the hierarchy
    ///
    /// # Returns
    /// `Vec<String>` - The top-level `Command` names.
    pub fn names(&self) -> Vec<String> {
        let mut names_vec: Vec<String> = self.iter().map(|cmd| cmd.name().to_string()).collect();
        // Since we are really just a map under the hood, we have no guaranteed ordering. This
        // helps this method be deterministic.
        names_vec.sort();
        return names_vec;
    }

    /// Produces an iterator over this set.
    ///
    /// # Returns
    /// `CommandSetIterator` - An iterator over this `CommandSet`.
    pub fn iter(&self) -> CommandSetIterator<S> {
        CommandSetIterator {
            iter: self.cmds.iter(),
        }
    }
}

/// An iterator for `CommandSet`'s.
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

// Since the CommandSet is just a wrapper around a HashMap, we don't really care too much about
// making very exhaustive or precise tests.
#[cfg(test)]
mod test {
    use super::*;

    use crate::command::{BaseCommand, Command};
    use crate::Result;

    struct EmptyCommand<'a> {
        name: &'a str,
    }

    impl<'a> EmptyCommand<'a> {
        fn new(name: &'a str) -> EmptyCommand<'a> {
            EmptyCommand { name }
        }
    }

    impl<'a> BaseCommand for EmptyCommand<'a> {
        type State = ();

        fn name(&self) -> &str {
            self.name
        }

        #[cfg(not(tarpaulin_include))]
        fn validate_args(&self, _: &Vec<String>) -> Result<()> {
            Ok(())
        }

        #[cfg(not(tarpaulin_include))]
        fn execute(&self, _: &mut Self::State, _: &Vec<String>) -> Result<String> {
            Ok(String::from(""))
        }
    }

    #[test]
    fn get() {
        let cmd_set = CommandSet::new_from_vec(vec![
            Command::new_leaf(EmptyCommand::new("a")),
            Command::new_leaf(EmptyCommand::new("b")),
            Command::new_leaf(EmptyCommand::new("c")),
        ]);

        // First of all, this should exist.
        assert!(cmd_set.get("b").is_some());

        // And this should be the 'b' command.
        assert_eq!(cmd_set.get("b").unwrap().name(), "b");
    }

    #[test]
    fn add() {
        let mut cmd_set = CommandSet::new();

        // Should not exist yet.
        assert!(cmd_set.get("a").is_none());

        cmd_set.add(Command::new_leaf(EmptyCommand::new("a")));

        // And now it should exist, so we should have that command we just added.
        assert_eq!(cmd_set.get("a").unwrap().name(), "a");
    }

    #[test]
    fn contains() {
        let cmd_set = CommandSet::new_from_vec(vec![Command::new_leaf(EmptyCommand::new("b"))]);

        // Test that an element that exists is properly detected.
        assert!(cmd_set.contains("b"));
        // Test that an element that DOESN'T exist is properly (not?) detected.
        assert!(!cmd_set.contains("I DONT EXIST"));
    }

    #[test]
    fn len() {
        let mut cmd_set = CommandSet::new();

        // Empty, so len() == 0:
        assert_eq!(cmd_set.len(), 0);

        // Adding one guy should mean our len is now 1...
        cmd_set.add(Command::new_leaf(EmptyCommand::new("a")));

        // So expect 1...
        assert_eq!(cmd_set.len(), 1);

        // Add some new stuff...
        cmd_set.add(Command::new_leaf(EmptyCommand::new("b")));
        cmd_set.add(Command::new_leaf(EmptyCommand::new("c")));

        // So expect 3 now cause 1 + 2 = 3...
        assert_eq!(cmd_set.len(), 3);
    }

    #[test]
    fn iter() {
        let cmd_set = CommandSet::new_from_vec(vec![
            Command::new_leaf(EmptyCommand::new("a")),
            Command::new_leaf(EmptyCommand::new("b")),
            Command::new_leaf(EmptyCommand::new("c")),
        ]);

        // We should expect to find 1 of a, b and c.
        let mut num_a = 0;
        let mut num_b = 0;
        let mut num_c = 0;

        for cmd in &cmd_set {
            match cmd.name() {
                "a" => num_a += 1,
                "b" => num_b += 1,
                "c" => num_c += 1,
                _ => panic!("unexpected command name from iteration"),
            }
        }

        assert_eq!(num_a, 1);
        assert_eq!(num_b, 1);
        assert_eq!(num_c, 1);
    }

    #[test]
    fn names() {
        let cmd_set = CommandSet::new_from_vec(vec![
            Command::new_leaf(EmptyCommand::new("a")),
            Command::new_leaf(EmptyCommand::new("b")),
            Command::new_leaf(EmptyCommand::new("c")),
        ]);

        let names = cmd_set.names();

        assert_eq!(vec!["a", "b", "c"], names);
    }
}
