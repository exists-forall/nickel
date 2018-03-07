use std::rc::Rc;

use super::types::*;

pub enum Expr {
    Var { index: usize },

    AbsType { kind: Kind, body: Rc<Expr> },
    AbsVar {
        access: FuncAccess,
        arg_type: Type,
        body: Rc<Expr>,
    },

    AppType { expr: Rc<Expr>, arg: Type },
    AppVar { expr: Rc<Expr>, arg: Rc<Expr> },

    LetVar { expr: Rc<Expr>, body: Rc<Expr> },
    LetPair { expr: Rc<Expr>, body: Rc<Expr> },
}
