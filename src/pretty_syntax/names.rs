use std::rc::Rc;
use std::collections::HashMap;
use std::fmt::Write;

use parse::lex::{quote_name, valid_name};

#[derive(Clone, Debug)]
struct Scope {
    index_count: usize,
    old_counts: HashMap<Rc<String>, usize>,
}

#[derive(Clone, Debug)]
pub struct Names {
    names: Vec<Rc<String>>,
    counts: HashMap<Rc<String>, usize>,
    scopes: Vec<Scope>,
}

impl Names {
    pub fn new() -> Self {
        Names {
            names: Vec::new(),
            counts: HashMap::new(),
            scopes: Vec::new(),
        }
    }

    pub fn index_count(&self) -> usize {
        self.names.len()
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope {
            index_count: self.names.len(),
            old_counts: HashMap::new(),
        });
    }

    pub fn pop_scope(&mut self) {
        let mut scope = self.scopes.pop().expect("Stack underflow");
        self.names.truncate(scope.index_count);
        for (name, old_count) in scope.old_counts.drain() {
            if old_count == 0 {
                self.counts.remove(&name);
            } else {
                self.counts.insert(name, old_count);
            }
        }
    }

    pub fn add_name(&mut self, name: Rc<String>) -> Rc<String> {
        if let Some(curr_count) = self.counts.get(&name).cloned() {
            if let Some(last_scope) = self.scopes.last_mut() {
                if !last_scope.old_counts.contains_key(&name) {
                    last_scope.old_counts.insert(name.clone(), curr_count);
                }
            }
            self.counts.insert(name.clone(), curr_count + 1);
            let new_name = Rc::new({
                let mut owned: String = if valid_name(&name) {
                    (*name).clone()
                } else {
                    quote_name(&name)
                };
                write!(&mut owned, "#{}", curr_count).unwrap();
                owned
            });
            self.names.push(new_name.clone());
            new_name
        } else {
            if let Some(last_scope) = self.scopes.last_mut() {
                last_scope.old_counts.insert(name.clone(), 0);
            }
            self.counts.insert(name.clone(), 1);
            let display_name = if valid_name(&name) {
                name
            } else {
                Rc::new(quote_name(&name))
            };
            self.names.push(display_name.clone());
            display_name
        }
    }

    pub fn get_name(&self, index: usize) -> Rc<String> {
        self.names[index].clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add_simple() {
        let mut names = Names::new();
        assert_eq!(names.index_count(), 0);

        assert_eq!(*names.add_name(Rc::new("foo".to_owned())), "foo");
        assert_eq!(*names.get_name(0), "foo");
        assert_eq!(names.index_count(), 1);

        assert_eq!(*names.add_name(Rc::new("bar".to_owned())), "bar");
        assert_eq!(*names.get_name(0), "foo");
        assert_eq!(*names.get_name(1), "bar");
        assert_eq!(names.index_count(), 2);
    }

    #[test]
    fn scoped() {
        let mut names = Names::new();

        assert_eq!(*names.add_name(Rc::new("foo".to_owned())), "foo");

        names.push_scope();
        assert_eq!(*names.get_name(0), "foo");
        assert_eq!(*names.add_name(Rc::new("bar".to_owned())), "bar");
        assert_eq!(*names.get_name(1), "bar");
        names.pop_scope();

        assert_eq!(*names.get_name(0), "foo");

        assert_eq!(*names.add_name(Rc::new("baz".to_owned())), "baz");
        assert_eq!(*names.get_name(0), "foo");
        assert_eq!(*names.get_name(1), "baz");
    }

    #[test]
    fn collision() {
        let mut names = Names::new();

        assert_eq!(*names.add_name(Rc::new("foo".to_owned())), "foo");
        assert_eq!(*names.add_name(Rc::new("foo".to_owned())), "foo#1");
        assert_eq!(*names.add_name(Rc::new("bar".to_owned())), "bar");

        assert_eq!(*names.get_name(0), "foo");
        assert_eq!(*names.get_name(1), "foo#1");
        assert_eq!(*names.get_name(2), "bar");

        assert_eq!(*names.add_name(Rc::new("foo".to_owned())), "foo#2");
        assert_eq!(*names.get_name(0), "foo");
        assert_eq!(*names.get_name(1), "foo#1");
        assert_eq!(*names.get_name(2), "bar");
        assert_eq!(*names.get_name(3), "foo#2");
    }

    #[test]
    fn scoped_collision() {
        let mut names = Names::new();

        assert_eq!(*names.add_name(Rc::new("foo".to_owned())), "foo");
        assert_eq!(*names.add_name(Rc::new("foo".to_owned())), "foo#1");

        names.push_scope();
        assert_eq!(*names.add_name(Rc::new("bar".to_owned())), "bar");
        assert_eq!(*names.add_name(Rc::new("foo".to_owned())), "foo#2");
        assert_eq!(*names.get_name(0), "foo");
        assert_eq!(*names.get_name(1), "foo#1");
        assert_eq!(*names.get_name(2), "bar");
        assert_eq!(*names.get_name(3), "foo#2");
        names.pop_scope();

        assert_eq!(*names.add_name(Rc::new("baz".to_owned())), "baz");
        assert_eq!(*names.add_name(Rc::new("foo".to_owned())), "foo#2");
        assert_eq!(*names.add_name(Rc::new("bar".to_owned())), "bar");
        assert_eq!(*names.get_name(0), "foo");
        assert_eq!(*names.get_name(1), "foo#1");
        assert_eq!(*names.get_name(2), "baz");
        assert_eq!(*names.get_name(3), "foo#2");
        assert_eq!(*names.get_name(4), "bar");
    }

    #[test]
    fn quoted() {
        let mut names = Names::new();

        assert_eq!(*names.add_name(Rc::new("".to_owned())), "``");
        assert_eq!(*names.add_name(Rc::new("".to_owned())), "``#1");
        assert_eq!(*names.add_name(Rc::new("forall".to_owned())), "`forall`");
        assert_eq!(
            *names.add_name(Rc::new("hello world".to_owned())),
            "`hello world`"
        );
        assert_eq!(
            *names.add_name(Rc::new("\\".to_owned())),
            "`\\\\`".to_owned()
        );
        assert_eq!(
            *names.add_name(Rc::new("Hello \\ world `".to_owned())),
            "`Hello \\\\ world \\``"
        );
        assert_eq!(*names.add_name(Rc::new("forall".to_owned())), "`forall`#1");

        assert_eq!(*names.add_name(Rc::new("equiv".to_owned())), "`equiv`");

        assert_eq!(*names.add_name(Rc::new("cast".to_owned())), "`cast`");

        assert_eq!(*names.get_name(0), "``");
        assert_eq!(*names.get_name(1), "``#1");
        assert_eq!(*names.get_name(2), "`forall`");
        assert_eq!(*names.get_name(3), "`hello world`");
        assert_eq!(*names.get_name(4), "`\\\\`");
        assert_eq!(*names.get_name(5), "`Hello \\\\ world \\``");
        assert_eq!(*names.get_name(6), "`forall`#1");
        assert_eq!(*names.get_name(7), "`equiv`");
        assert_eq!(*names.get_name(8), "`cast`");
    }
}
