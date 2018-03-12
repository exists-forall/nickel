use std::rc::Rc;

use super::types::*;

pub enum Expr<TName> {
    Var { index: usize },

    AbsType { kind: Kind, body: Rc<Expr<TName>> },
    AbsVar {
        arg_type: Type<TName>,
        body: Rc<Expr<TName>>,
    },

    AppType {
        expr: Rc<Expr<TName>>,
        arg: Type<TName>,
    },
    AppVar {
        expr: Rc<Expr<TName>>,
        arg: Rc<Expr<TName>>,
    },

    LetVar {
        expr: Rc<Expr<TName>>,
        body: Rc<Expr<TName>>,
    },
    LetPair {
        expr: Rc<Expr<TName>>,
        body: Rc<Expr<TName>>,
    },
}
