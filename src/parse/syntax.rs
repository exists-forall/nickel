use std::rc::Rc;

use types::{Phase, Quantifier};
use expr::{Intrinsic, VarUsage};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Ident {
    pub name: Rc<String>,
    pub collision_id: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeParam {
    pub ident: Ident,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Unit,
    Var {
        ident: Ident,
    },
    Quantified {
        quantifier: Quantifier,
        param: TypeParam,
        body: Box<Type>,
    },
    Func {
        arg: Box<Type>,
        arg_phase: Phase,
        ret: Box<Type>,
        ret_phase: Phase,
    },
    Pair {
        left: Box<Type>,
        right: Box<Type>,
    },
    App {
        constructor: Box<Type>,
        param: Box<Type>,
    },
    Equiv {
        orig: Box<Type>,
        dest: Box<Type>,
    },
    Size {
        ty: Box<Type>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Unit,
    Var {
        usage: VarUsage,
        ident: Ident,
    },
    ForAll {
        type_params: Vec<TypeParam>,
        body: Box<Expr>,
    },
    Func {
        arg_name: Ident,
        arg_type: Type,
        arg_phase: Phase,
        body: Box<Expr>,
    },
    Inst {
        receiver: Box<Expr>,
        type_params: Vec<Type>,
    },
    App {
        callee: Box<Expr>,
        arg: Box<Expr>,
    },
    Pair {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Let {
        names: Vec<Ident>,
        val: Box<Expr>,
        body: Box<Expr>,
    },
    LetExists {
        type_names: Vec<Ident>,
        val_name: Ident,
        val: Box<Expr>,
        body: Box<Expr>,
    },
    MakeExists {
        params: Vec<(Ident, Type)>,
        type_body: Type,
        body: Box<Expr>,
    },
    Cast {
        param: TypeParam,
        type_body: Type,
        equivalence: Box<Expr>,
        body: Box<Expr>,
    },
    Intrinsic {
        intrinsic: Intrinsic,
    },
}
