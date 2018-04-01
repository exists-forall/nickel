use types::Kind;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Ident {
    pub name: String,
    pub collision_id: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeParam {
    pub ident: Ident,
    pub kind: Kind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Unit,
    Var { ident: Ident },
    Exists { param: TypeParam, body: Box<Type> },
    Func {
        params: Vec<TypeParam>,
        arg: Box<Type>,
        ret: Box<Type>,
    },
    Pair { left: Box<Type>, right: Box<Type> },
    App {
        constructor: Box<Type>,
        param: Box<Type>,
    },
}
