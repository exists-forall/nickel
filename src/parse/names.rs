use std::collections::HashMap;

use super::syntax::Ident;

#[derive(Clone, Debug)]
struct Scope {
    added_names: Vec<Ident>,
}

#[derive(Clone, Debug)]
pub struct Names {
    indices: HashMap<Ident, usize>,
    scopes: Vec<Scope>,
}

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
        self.scopes.push(Scope { added_names: Vec::new() })
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
