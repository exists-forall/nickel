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
struct Var<Name> {
    ty: Type<Name>,
    usage: Usage,
}

#[derive(Clone, Debug)]
pub struct Context<Name> {
    type_kinds: Vec<Kind>,
    vars: Vec<Var<Name>>,
    scopes: Vec<Scope>,
}

impl<Name: Clone> Context<Name> {
    pub fn new() -> Self {
        Context {
            type_kinds: Vec::new(),
            vars: Vec::new(),
            scopes: Vec::new(),
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope {
            type_count: self.type_kinds.len(),
            var_count: self.vars.len(),
        })
    }

    pub fn pop_sope(&mut self) {
        let scope = self.scopes.pop().expect("Stack underflow");
        self.type_kinds.truncate(scope.type_count);
        self.vars.truncate(scope.var_count);
    }

    pub fn curr_scope_vars(&self) -> Range<usize> {
        if let Some(last_scope) = self.scopes.last() {
            last_scope.var_count..self.vars.len()
        } else {
            0..self.vars.len()
        }
    }

    pub fn type_kind(&self, index: usize) -> &Kind {
        &self.type_kinds[index]
    }

    pub fn var_type(&self, index: usize) -> &Type<Name> {
        &self.vars[index].ty
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

    pub fn add_type_kind(&mut self, kind: Kind) {
        self.type_kinds.push(kind);
    }

    pub fn add_var_unmoved(&mut self, ty: Type<Name>) {
        self.vars.push(Var {
            ty,
            usage: Usage::Unmoved,
        });
    }
}
