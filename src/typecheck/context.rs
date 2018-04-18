use std::ops::Range;

use types::*;

#[derive(Clone, Debug)]
struct Scope {
    type_count: usize,
    var_count: usize,
}

#[derive(Clone, Copy, Debug)]
pub enum Usage {
    Moved,
    Unmoved,
}

#[derive(Clone, Debug)]
struct TypeBinding<Name> {
    name: Name,
}

#[derive(Clone, Debug)]
pub struct Annot<Name> {
    pub phase: Phase,
    pub ty: Type<Name>,
}

#[derive(Clone, Debug)]
struct Var<Name> {
    name: Name,
    annot: Annot<Name>,
    usage: Usage,
}

#[derive(Clone, Debug)]
pub struct Context<Name> {
    types: Vec<TypeBinding<Name>>,
    vars: Vec<Var<Name>>,
    scopes: Vec<Scope>,
}

impl<Name: Clone> Context<Name> {
    pub fn new() -> Self {
        Context {
            types: Vec::new(),
            vars: Vec::new(),
            scopes: Vec::new(),
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope {
            type_count: self.types.len(),
            var_count: self.vars.len(),
        })
    }

    pub fn pop_scope(&mut self) {
        let scope = self.scopes.pop().expect("Stack underflow");
        self.types.truncate(scope.type_count);
        self.vars.truncate(scope.var_count);
    }

    pub fn curr_scope_vars(&self) -> Range<usize> {
        if let Some(last_scope) = self.scopes.last() {
            last_scope.var_count..self.vars.len()
        } else {
            0..self.vars.len()
        }
    }

    pub fn type_index_count(&self) -> usize {
        self.types.len()
    }

    pub fn var_index_count(&self) -> usize {
        self.vars.len()
    }

    pub fn type_name(&self, index: usize) -> &Name {
        &self.types[index].name
    }

    pub fn var_name(&self, index: usize) -> &Name {
        &self.vars[index].name
    }

    pub fn var_type(&self, index: usize) -> &Type<Name> {
        &self.vars[index].annot.ty
    }

    pub fn var_phase(&self, index: usize) -> Phase {
        self.vars[index].annot.phase
    }

    pub fn var_usage(&self, index: usize) -> Usage {
        self.vars[index].usage
    }

    pub fn move_var(&mut self, index: usize) -> Result<(), ()> {
        match self.vars[index].usage {
            Usage::Unmoved => {
                self.vars[index].usage = Usage::Moved;
                Ok(())
            }
            Usage::Moved => Err(()),
        }
    }

    pub fn add_type(&mut self, name: Name) {
        self.types.push(TypeBinding { name });
    }

    pub fn add_var_unmoved(&mut self, name: Name, annot: Annot<Name>) {
        self.vars.push(Var {
            name,
            annot,
            usage: Usage::Unmoved,
        });
    }
}
