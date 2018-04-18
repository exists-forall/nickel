use std::rc::Rc;

use super::types::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VarUsage {
    Move,
    Copy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ExprDataInner<TAnnot, EAnnot, Name> {
    Unit,

    Var { usage: VarUsage, index: usize },

    ForAll {
        type_params: Rc<Vec<TypeParam<Name>>>,
        body: ExprData<TAnnot, EAnnot, Name>,
    },

    Func {
        arg_name: Name,
        arg_type: AnnotType<TAnnot, Name>,
        arg_phase: Phase,
        body: ExprData<TAnnot, EAnnot, Name>,
    },

    Inst {
        receiver: ExprData<TAnnot, EAnnot, Name>,
        type_params: Rc<Vec<AnnotType<TAnnot, Name>>>,
    },

    App {
        callee: ExprData<TAnnot, EAnnot, Name>,
        arg: ExprData<TAnnot, EAnnot, Name>,
    },

    Pair {
        left: ExprData<TAnnot, EAnnot, Name>,
        right: ExprData<TAnnot, EAnnot, Name>,
    },

    Let {
        names: Rc<Vec<Name>>,
        val: ExprData<TAnnot, EAnnot, Name>,
        body: ExprData<TAnnot, EAnnot, Name>,
    },

    LetExists {
        type_names: Rc<Vec<Name>>,
        val_name: Name,
        val: ExprData<TAnnot, EAnnot, Name>,
        body: ExprData<TAnnot, EAnnot, Name>,
    },

    MakeExists {
        params: Rc<Vec<(Name, AnnotType<TAnnot, Name>)>>,
        type_body: AnnotType<TAnnot, Name>,
        body: ExprData<TAnnot, EAnnot, Name>,
    },

    Cast {
        param: TypeParam<Name>,
        type_body: AnnotType<TAnnot, Name>,
        equivalence: ExprData<TAnnot, EAnnot, Name>,
        body: ExprData<TAnnot, EAnnot, Name>,
    },

    ReflEquiv { ty: AnnotType<TAnnot, Name> },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExprData<TAnnot, EAnnot, Name> {
    annot: EAnnot,
    inner: Rc<ExprDataInner<TAnnot, EAnnot, Name>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExprContent<TAnnot, EAnnot, Name> {
    Unit { free_vars: usize, free_types: usize },

    Var {
        usage: VarUsage,
        free_vars: usize,
        free_types: usize,
        index: usize,
    },

    ForAll {
        type_params: Rc<Vec<TypeParam<Name>>>,
        body: AnnotExpr<TAnnot, EAnnot, Name>,
    },

    Func {
        arg_name: Name,
        arg_type: AnnotType<TAnnot, Name>,
        arg_phase: Phase,
        body: AnnotExpr<TAnnot, EAnnot, Name>,
    },

    Inst {
        receiver: AnnotExpr<TAnnot, EAnnot, Name>,
        type_params: Rc<Vec<AnnotType<TAnnot, Name>>>,
    },

    App {
        callee: AnnotExpr<TAnnot, EAnnot, Name>,
        arg: AnnotExpr<TAnnot, EAnnot, Name>,
    },

    Pair {
        left: AnnotExpr<TAnnot, EAnnot, Name>,
        right: AnnotExpr<TAnnot, EAnnot, Name>,
    },

    Let {
        names: Rc<Vec<Name>>,
        val: AnnotExpr<TAnnot, EAnnot, Name>,
        body: AnnotExpr<TAnnot, EAnnot, Name>,
    },

    LetExists {
        type_names: Rc<Vec<Name>>,
        val_name: Name,
        val: AnnotExpr<TAnnot, EAnnot, Name>,
        body: AnnotExpr<TAnnot, EAnnot, Name>,
    },

    MakeExists {
        params: Rc<Vec<(Name, AnnotType<TAnnot, Name>)>>,
        type_body: AnnotType<TAnnot, Name>,
        body: AnnotExpr<TAnnot, EAnnot, Name>,
    },

    Cast {
        param: TypeParam<Name>,
        type_body: AnnotType<TAnnot, Name>,
        equivalence: AnnotExpr<TAnnot, EAnnot, Name>,
        body: AnnotExpr<TAnnot, EAnnot, Name>,
    },

    ReflEquiv {
        free_vars: usize,
        ty: AnnotType<TAnnot, Name>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnnotExpr<TAnnot, EAnnot, Name> {
    free_vars: usize,
    free_types: usize,
    data: ExprData<TAnnot, EAnnot, Name>,
}

pub type Expr<Name> = AnnotExpr<(), (), Name>;

impl<TAnnot: Clone, EAnnot: Clone, Name: Clone> AnnotExpr<TAnnot, EAnnot, Name> {
    pub fn free_vars(&self) -> usize {
        self.free_vars
    }

    pub fn free_types(&self) -> usize {
        self.free_types
    }

    pub fn annot(&self) -> EAnnot {
        self.data.annot.clone()
    }

    pub fn from_content_annot(annot: EAnnot, content: ExprContent<TAnnot, EAnnot, Name>) -> Self {
        match content {
            ExprContent::Unit {
                free_vars,
                free_types,
            } => {
                AnnotExpr {
                    free_vars,
                    free_types,
                    data: ExprData {
                        annot,
                        inner: Rc::new(ExprDataInner::Unit),
                    },
                }
            }

            ExprContent::Var {
                usage,
                free_vars,
                free_types,
                index,
            } => {
                assert!(index < free_vars);
                AnnotExpr {
                    free_vars,
                    free_types,
                    data: ExprData {
                        annot,
                        inner: Rc::new(ExprDataInner::Var { usage, index }),
                    },
                }
            }

            ExprContent::ForAll { type_params, body } => {
                assert!(
                    type_params.len() <= body.free_types,
                    "Must have at least {} free type variables",
                );

                AnnotExpr {
                    free_vars: body.free_vars,
                    free_types: body.free_types - type_params.len(),
                    data: ExprData {
                        annot,
                        inner: Rc::new(ExprDataInner::ForAll {
                            type_params,
                            body: body.data,
                        }),
                    },
                }
            }

            ExprContent::Func {
                arg_name,
                arg_type,
                arg_phase,
                body,
            } => {
                assert_eq!(
                    arg_type.free(),
                    body.free_types,
                    "Free type variables do not match",
                );

                assert!(
                    1 <= body.free_vars,
                    "Must have at least one free term variable",
                );

                AnnotExpr {
                    free_vars: body.free_vars - 1,
                    free_types: body.free_types,
                    data: ExprData {
                        annot,
                        inner: Rc::new(ExprDataInner::Func {
                            arg_name,
                            arg_type,
                            arg_phase,
                            body: body.data,
                        }),
                    },
                }
            }

            ExprContent::Inst {
                receiver,
                type_params,
            } => {
                for param in type_params.iter() {
                    assert_eq!(
                        param.free(),
                        receiver.free_types,
                        "Free type variables do not match"
                    );
                }
                AnnotExpr {
                    free_vars: receiver.free_vars,
                    free_types: receiver.free_types,
                    data: ExprData {
                        annot,
                        inner: Rc::new(ExprDataInner::Inst {
                            receiver: receiver.data,
                            type_params,
                        }),
                    },
                }
            }

            ExprContent::App { callee, arg } => {
                assert_eq!(
                    callee.free_vars,
                    arg.free_vars,
                    "Free term variables do not match"
                );

                assert_eq!(
                    callee.free_types,
                    arg.free_types,
                    "Free type variables do not match",
                );

                AnnotExpr {
                    free_vars: arg.free_vars,
                    free_types: arg.free_types,
                    data: ExprData {
                        annot,
                        inner: Rc::new(ExprDataInner::App {
                            callee: callee.data,
                            arg: arg.data,
                        }),
                    },
                }
            }

            ExprContent::Pair { left, right } => {
                assert_eq!(
                    left.free_vars,
                    right.free_vars,
                    "Free term variables do not match",
                );

                assert_eq!(
                    left.free_types,
                    right.free_types,
                    "Free type variables do not match",
                );

                AnnotExpr {
                    free_vars: left.free_vars,
                    free_types: left.free_types,
                    data: ExprData {
                        annot,
                        inner: Rc::new(ExprDataInner::Pair {
                            left: left.data,
                            right: right.data,
                        }),
                    },
                }
            }

            ExprContent::Let { names, val, body } => {
                assert!(names.len() > 0, "Must bind at least one variable");

                assert_eq!(
                    val.free_types,
                    body.free_types,
                    "Free type variables do not match",
                );

                assert_eq!(
                    val.free_vars + names.len(),
                    body.free_vars,
                    "Free term variables do not match",
                );

                AnnotExpr {
                    free_vars: val.free_vars,
                    free_types: val.free_types,
                    data: ExprData {
                        annot,
                        inner: Rc::new(ExprDataInner::Let {
                            names,
                            val: val.data,
                            body: body.data,
                        }),
                    },
                }
            }

            ExprContent::LetExists {
                type_names,
                val_name,
                val,
                body,
            } => {
                assert!(type_names.len() > 0, "Must bind at least one type");

                assert_eq!(
                    val.free_types + type_names.len(),
                    body.free_types,
                    "Free type variables do not match",
                );

                assert_eq!(
                    val.free_vars + 1,
                    body.free_vars,
                    "Free term variables do not match",
                );

                AnnotExpr {
                    free_vars: val.free_vars,
                    free_types: val.free_types,
                    data: ExprData {
                        annot,
                        inner: Rc::new(ExprDataInner::LetExists {
                            type_names,
                            val_name,
                            val: val.data,
                            body: body.data,
                        }),
                    },
                }
            }

            ExprContent::MakeExists {
                params,
                type_body,
                body,
            } => {
                assert!(params.len() > 0, "Must bind at least one type");

                assert_eq!(
                    body.free_types + params.len(),
                    type_body.free(),
                    "Free type variables do not match",
                );

                for &(_, ref param) in params.iter() {
                    assert_eq!(
                        param.free(),
                        body.free_types,
                        "Free type variables do not match",
                    );
                }

                AnnotExpr {
                    free_vars: body.free_vars,
                    free_types: body.free_types,
                    data: ExprData {
                        annot,
                        inner: Rc::new(ExprDataInner::MakeExists {
                            params,
                            type_body,
                            body: body.data,
                        }),
                    },
                }
            }

            ExprContent::Cast {
                param,
                type_body,
                equivalence,
                body,
            } => {
                assert_eq!(
                    equivalence.free_types,
                    body.free_types,
                    "Free type variables do not match",
                );

                assert_eq!(
                    equivalence.free_vars,
                    body.free_vars,
                    "Free term variables do not match",
                );

                assert_eq!(
                    type_body.free(),
                    body.free_vars + 1,
                    "Free type variables do not match"
                );

                AnnotExpr {
                    free_vars: body.free_vars,
                    free_types: body.free_types,
                    data: ExprData {
                        annot,
                        inner: Rc::new(ExprDataInner::Cast {
                            param,
                            type_body,
                            equivalence: equivalence.data,
                            body: body.data,
                        }),
                    },
                }
            }

            ExprContent::ReflEquiv { free_vars, ty } => {
                AnnotExpr {
                    free_vars,
                    free_types: ty.free(),
                    data: ExprData {
                        annot,
                        inner: Rc::new(ExprDataInner::ReflEquiv { ty }),
                    },
                }
            }
        }
    }

    pub fn to_content(&self) -> ExprContent<TAnnot, EAnnot, Name> {
        match &*self.data.inner {
            &ExprDataInner::Unit => {
                ExprContent::Unit {
                    free_vars: self.free_vars,
                    free_types: self.free_types,
                }
            }

            &ExprDataInner::Var { usage, index } => {
                ExprContent::Var {
                    free_vars: self.free_vars,
                    free_types: self.free_types,
                    usage,
                    index,
                }
            }

            &ExprDataInner::ForAll {
                ref type_params,
                ref body,
            } => {
                ExprContent::ForAll {
                    type_params: type_params.clone(),
                    body: AnnotExpr {
                        free_types: self.free_types + type_params.len(),
                        free_vars: self.free_vars,
                        data: body.clone(),
                    },
                }
            }

            &ExprDataInner::Func {
                ref arg_name,
                ref arg_type,
                arg_phase,
                ref body,
            } => {
                ExprContent::Func {
                    arg_name: arg_name.clone(),
                    arg_type: arg_type.clone(),
                    arg_phase,
                    body: AnnotExpr {
                        free_types: self.free_types,
                        free_vars: self.free_vars + 1,
                        data: body.clone(),
                    },
                }
            }

            &ExprDataInner::Inst {
                ref receiver,
                ref type_params,
            } => {
                ExprContent::Inst {
                    receiver: AnnotExpr {
                        free_types: self.free_types,
                        free_vars: self.free_vars,
                        data: receiver.clone(),
                    },
                    type_params: type_params.clone(),
                }
            }

            &ExprDataInner::App {
                ref callee,
                ref arg,
            } => {
                ExprContent::App {
                    callee: AnnotExpr {
                        free_types: self.free_types,
                        free_vars: self.free_vars,
                        data: callee.clone(),
                    },
                    arg: AnnotExpr {
                        free_types: self.free_types,
                        free_vars: self.free_vars,
                        data: arg.clone(),
                    },
                }
            }

            &ExprDataInner::Pair {
                ref left,
                ref right,
            } => {
                ExprContent::Pair {
                    left: AnnotExpr {
                        free_types: self.free_types,
                        free_vars: self.free_vars,
                        data: left.clone(),
                    },
                    right: AnnotExpr {
                        free_types: self.free_types,
                        free_vars: self.free_vars,
                        data: right.clone(),
                    },
                }
            }

            &ExprDataInner::Let {
                ref names,
                ref val,
                ref body,
            } => {
                ExprContent::Let {
                    names: names.clone(),
                    val: AnnotExpr {
                        free_types: self.free_types,
                        free_vars: self.free_vars,
                        data: val.clone(),
                    },
                    body: AnnotExpr {
                        free_types: self.free_types,
                        free_vars: self.free_vars + names.len(),
                        data: body.clone(),
                    },
                }
            }

            &ExprDataInner::LetExists {
                ref type_names,
                ref val_name,
                ref val,
                ref body,
            } => {
                ExprContent::LetExists {
                    type_names: type_names.clone(),
                    val_name: val_name.clone(),
                    val: AnnotExpr {
                        free_types: self.free_types,
                        free_vars: self.free_vars,
                        data: val.clone(),
                    },
                    body: AnnotExpr {
                        free_types: self.free_types + type_names.len(),
                        free_vars: self.free_vars + 1,
                        data: body.clone(),
                    },
                }
            }

            &ExprDataInner::MakeExists {
                ref params,
                ref type_body,
                ref body,
            } => {
                ExprContent::MakeExists {
                    params: params.clone(),
                    type_body: type_body.clone(),
                    body: AnnotExpr {
                        free_vars: self.free_vars,
                        free_types: self.free_types,
                        data: body.clone(),
                    },
                }
            }

            &ExprDataInner::Cast {
                ref param,
                ref type_body,
                ref equivalence,
                ref body,
            } => {
                ExprContent::Cast {
                    param: param.clone(),
                    type_body: type_body.clone(),
                    equivalence: AnnotExpr {
                        free_vars: self.free_vars,
                        free_types: self.free_types,
                        data: equivalence.clone(),
                    },
                    body: AnnotExpr {
                        free_vars: self.free_vars,
                        free_types: self.free_types,
                        data: body.clone(),
                    },
                }
            }

            &ExprDataInner::ReflEquiv { ref ty } => {
                ExprContent::ReflEquiv {
                    free_vars: self.free_vars,
                    ty: ty.clone(),
                }
            }
        }
    }
}

impl<TAnnot: Clone, Name: Clone> AnnotExpr<TAnnot, (), Name> {
    pub fn from_content(content: ExprContent<TAnnot, (), Name>) -> Self {
        AnnotExpr::from_content_annot((), content)
    }
}
