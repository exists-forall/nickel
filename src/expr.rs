use std::rc::Rc;

use super::types::*;

pub enum Expr<Free> {
    Var { index: usize },

    AbsType { kind: Kind, body: Rc<Expr<Free>> },
    AbsVar {
        access: FuncAccess,
        arg_type: Type<Free>,
        body: Rc<Expr<Free>>,
    },

    AppType {
        expr: Rc<Expr<Free>>,
        arg: Type<Free>,
    },
    AppVar {
        expr: Rc<Expr<Free>>,
        arg: Rc<Expr<Free>>,
    },

    LetVar {
        expr: Rc<Expr<Free>>,
        body: Rc<Expr<Free>>,
    },
    LetPair {
        expr: Rc<Expr<Free>>,
        body: Rc<Expr<Free>>,
    },
}
