use types::*;

#[derive(Clone, Debug)]
struct Scope {
    type_count: usize,
    var_count: usize,
}

#[derive(Clone, Debug)]
pub struct Context<Name> {
    type_kinds: Vec<Kind>,
    var_types: Vec<Type<Name>>,
    scopes: Vec<Scope>,
}

impl<Name: Clone> Context<Name> {
    pub fn new() -> Self {
        Context {
            type_kinds: Vec::new(),
            var_types: Vec::new(),
            scopes: Vec::new(),
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope {
            type_count: self.type_kinds.len(),
            var_count: self.var_types.len(),
        })
    }

    pub fn pop_sope(&mut self) {
        let scope = self.scopes.pop().expect("Stack underflow");
        self.type_kinds.truncate(scope.type_count);
        self.var_types.truncate(scope.var_count);
    }

    pub fn type_kind(&self, index: usize) -> &Kind {
        &self.type_kinds[index]
    }

    pub fn var_type(&self, index: usize) -> &Type<Name> {
        &self.var_types[index]
    }

    pub fn add_type_kind(&mut self, kind: Kind) {
        self.type_kinds.push(kind);
    }

    pub fn add_var_type(&mut self, ty: Type<Name>) {
        self.var_types.push(ty);
    }
}
