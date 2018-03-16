use std::rc::Rc;

use super::types::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValParam<Name> {
    pub name: Name,
    pub ty: Type<Name>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExprDataInner<Name> {
    Var { index: usize },

    CopyVar { index: usize },

    Abs {
        type_params: Rc<Vec<TypeParam<Name>>>,
        val_param: ValParam<Name>,
    },

    App {
        type_params: Rc<Vec<Type<Name>>>,
        val_param: ExprData<Name>,
    },

    Pair {
        left: ExprData<Name>,
        right: ExprData<Name>,
    },

    LetPair {
        names: Rc<Vec<Name>>,
        val: ExprData<Name>,
        body: ExprData<Name>,
    },

    LetExists {
        type_names: Rc<Vec<Name>>,
        val_name: Name,
        body: ExprData<Name>,
    },

    MakeExists {
        params: Rc<Vec<(Name, Type<Name>)>>,
        type_body: Type<Name>,
        body: ExprData<Name>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExprData<Name> {
    inner: Rc<ExprDataInner<Name>>,
}
