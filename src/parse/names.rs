use std::collections::HashMap;

use super::syntax::Ident;

// TODO: Determine if this should use `Rc<String>` instead of `String` to avoid unnecessary deep
// clones in to_indices.

#[derive(Clone, Debug)]
struct Scope {
    added_names: Vec<Ident>,
}

#[derive(Clone, Debug)]
pub struct Names {
    indices: HashMap<Ident, usize>,
    scopes: Vec<Scope>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Shadow(Ident),
    NotFound(Ident),
}

impl Names {
    pub fn new() -> Self {
        Names {
            indices: HashMap::new(),
            scopes: Vec::new(),
        }
    }

    pub fn index_count(&self) -> usize {
        self.indices.len()
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope {
            added_names: Vec::new(),
        })
    }

    pub fn pop_scope(&mut self) {
        let scope = self.scopes.pop().expect("Stack underflow");
        for name in &scope.added_names {
            self.indices.remove(name);
        }
    }

    pub fn add_name(&mut self, name: Ident) -> Result<(), Error> {
        let new_index = self.index_count();
        let old_index = self.indices.insert(name.clone(), new_index);

        if old_index.is_some() {
            return Err(Error::Shadow(name));
        }

        if let Some(last_scope) = self.scopes.last_mut() {
            last_scope.added_names.push(name);
        }

        Ok(())
    }

    pub fn get_index(&self, name: &Ident) -> Result<usize, Error> {
        if let Some(&index) = self.indices.get(name) {
            Ok(index)
        } else {
            Err(Error::NotFound(name.clone()))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use test_utils::parse_syntax::*;

    #[test]
    fn add_simple() {
        let mut names = Names::new();
        assert_eq!(names.index_count(), 0);

        assert!(names.add_name(mk_ident("hello")).is_ok());
        assert_eq!(names.index_count(), 1);
        assert_eq!(names.get_index(&mk_ident("hello")), Ok(0));

        assert!(names.add_name(mk_ident("world")).is_ok());
        assert_eq!(names.index_count(), 2);
        assert_eq!(names.get_index(&mk_ident("hello")), Ok(0));
        assert_eq!(names.get_index(&mk_ident("world")), Ok(1));

        assert!(names.add_name(mk_ident_collision("hello", 1)).is_ok());
        assert_eq!(names.index_count(), 3);
        assert_eq!(names.get_index(&mk_ident("hello")), Ok(0));
        assert_eq!(names.get_index(&mk_ident("world")), Ok(1));
        assert_eq!(names.get_index(&mk_ident_collision("hello", 1)), Ok(2));
    }

    #[test]
    fn shadow() {
        let mut names = Names::new();
        assert!(names.add_name(mk_ident("hello")).is_ok());
        assert!(names.add_name(mk_ident("hello")).is_err());
    }

    #[test]
    fn not_found() {
        let mut names = Names::new();
        assert!(names.add_name(mk_ident("hello")).is_ok());
        assert!(names.get_index(&mk_ident("world")).is_err());
        assert!(names.get_index(&mk_ident_collision("hello", 1)).is_err());
    }

    #[test]
    fn scoped() {
        let mut names = Names::new();

        assert!(names.add_name(mk_ident("hello")).is_ok());

        names.push_scope();

        assert_eq!(names.index_count(), 1);
        assert_eq!(names.get_index(&mk_ident("hello")), Ok(0));

        assert!(names.add_name(mk_ident("world")).is_ok());
        assert_eq!(names.index_count(), 2);
        assert_eq!(names.get_index(&mk_ident("hello")), Ok(0));
        assert_eq!(names.get_index(&mk_ident("world")), Ok(1));

        assert!(names.add_name(mk_ident("foo")).is_ok());
        assert_eq!(names.index_count(), 3);
        assert_eq!(names.get_index(&mk_ident("hello")), Ok(0));
        assert_eq!(names.get_index(&mk_ident("world")), Ok(1));
        assert_eq!(names.get_index(&mk_ident("foo")), Ok(2));

        names.pop_scope();

        assert_eq!(names.index_count(), 1);
        assert_eq!(names.get_index(&mk_ident("hello")), Ok(0));
        assert!(names.get_index(&mk_ident("world")).is_err());
        assert!(names.get_index(&mk_ident("foo")).is_err());
    }
}
