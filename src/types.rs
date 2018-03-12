use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    Type,
    Place,
    Version,
    Constructor {
        params: Rc<Vec<Kind>>,
        result: Rc<Kind>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeParam<Name> {
    pub name: Name,
    pub kind: Kind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TypeDataInner<Name> {
    Var { index: usize },
    Exists {
        param: TypeParam<Name>,
        body: TypeData<Name>,
    },
    Func {
        params: Rc<Vec<TypeParam<Name>>>,
        arg: TypeData<Name>,
        ret: TypeData<Name>,
    },
    Pair {
        left: TypeData<Name>,
        right: TypeData<Name>,
    },
    /// Represents both partial and (segments of) total type applications
    App {
        constructor: TypeData<Name>,
        param: TypeData<Name>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TypeData<Name> {
    max_index: usize, // exclusive upper bound
    inner: Rc<TypeDataInner<Name>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeContent<Name> {
    Var { free: usize, index: usize },
    Exists {
        param: TypeParam<Name>,
        body: Type<Name>,
    },
    Func {
        params: Rc<Vec<TypeParam<Name>>>,
        arg: Type<Name>,
        ret: Type<Name>,
    },
    Pair { left: Type<Name>, right: Type<Name> },
    App {
        constructor: Type<Name>,
        param: Type<Name>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Type<Name> {
    free: usize,
    data: TypeData<Name>,
}

impl<Name: Clone> Type<Name> {
    pub fn free(&self) -> usize {
        self.free
    }

    pub fn from_content(content: TypeContent<Name>) -> Self {
        match content {
            TypeContent::Var { free, index } => {
                assert!(index < free);
                Type {
                    free,
                    data: TypeData {
                        max_index: index + 1,
                        inner: Rc::new(TypeDataInner::Var { index }),
                    },
                }
            }

            TypeContent::Exists { param, body } => {
                assert!(1 <= body.free, "Must have at least one free variable");
                Type {
                    free: body.free - 1,
                    data: TypeData {
                        max_index: body.data.max_index,
                        inner: Rc::new(TypeDataInner::Exists {
                            param: param.clone(),
                            body: body.data,
                        }),
                    },
                }
            }

            TypeContent::Func { params, arg, ret } => {
                assert_eq!(arg.free, ret.free, "Free variables do not match");
                assert!(
                    params.len() <= arg.free,
                    "Must have at least {} free variables",
                );
                Type {
                    free: arg.free - params.len(),
                    data: TypeData {
                        max_index: arg.data.max_index.max(ret.data.max_index),
                        inner: Rc::new(TypeDataInner::Func {
                            params: params,
                            arg: arg.data,
                            ret: ret.data,
                        }),
                    },
                }
            }

            TypeContent::Pair { left, right } => {
                assert_eq!(left.free, right.free, "Free variables do not match");
                Type {
                    free: left.free,
                    data: TypeData {
                        max_index: left.data.max_index.max(right.data.max_index),
                        inner: Rc::new(TypeDataInner::Pair {
                            left: left.data,
                            right: right.data,
                        }),
                    },
                }
            }

            TypeContent::App { constructor, param } => {
                assert_eq!(constructor.free, param.free, "Free variables do not match");
                Type {
                    free: constructor.free,
                    data: TypeData {
                        max_index: constructor.data.max_index.max(param.data.max_index),
                        inner: Rc::new(TypeDataInner::App {
                            constructor: constructor.data,
                            param: param.data,
                        }),
                    },
                }
            }
        }
    }

    pub fn to_content(&self) -> TypeContent<Name> {
        match &*self.data.inner {
            &TypeDataInner::Var { index } => {
                TypeContent::Var {
                    free: self.free,
                    index,
                }
            }

            &TypeDataInner::Exists {
                ref param,
                ref body,
            } => {
                TypeContent::Exists {
                    param: param.clone(),
                    body: Type {
                        free: self.free + 1,
                        data: body.clone(),
                    },
                }
            }

            &TypeDataInner::Func {
                ref params,
                ref arg,
                ref ret,
            } => {
                TypeContent::Func {
                    params: params.clone(),
                    arg: Type {
                        free: self.free + params.len(),
                        data: arg.clone(),
                    },
                    ret: Type {
                        free: self.free + params.len(),
                        data: ret.clone(),
                    },
                }
            }

            &TypeDataInner::Pair {
                ref left,
                ref right,
            } => {
                TypeContent::Pair {
                    left: Type {
                        free: self.free,
                        data: left.clone(),
                    },
                    right: Type {
                        free: self.free,
                        data: right.clone(),
                    },
                }
            }

            &TypeDataInner::App {
                ref constructor,
                ref param,
            } => {
                TypeContent::App {
                    constructor: Type {
                        free: self.free,
                        data: constructor.clone(),
                    },
                    param: Type {
                        free: self.free,
                        data: param.clone(),
                    },
                }
            }
        }
    }

    fn increment_above(&self, index: usize, inc_by: usize) -> Self {
        debug_assert!(index <= self.free);

        if self.data.max_index <= index {
            return Type {
                free: self.free + inc_by,
                data: self.data.clone(),
            };
        }

        let new_content = match self.to_content() {
            TypeContent::Var {
                free,
                index: var_index,
            } => {
                if index <= var_index {
                    TypeContent::Var {
                        free: free + inc_by,
                        index: var_index + inc_by,
                    }
                } else {
                    TypeContent::Var {
                        free: free + inc_by,
                        index: var_index,
                    }
                }
            }

            TypeContent::Exists { param, body } => {
                TypeContent::Exists {
                    param,
                    body: body.increment_above(index, inc_by),
                }
            }

            TypeContent::Func { params, arg, ret } => {
                TypeContent::Func {
                    params,
                    arg: arg.increment_above(index, inc_by),
                    ret: ret.increment_above(index, inc_by),
                }
            }

            TypeContent::Pair { left, right } => {
                TypeContent::Pair {
                    left: left.increment_above(index, inc_by),
                    right: right.increment_above(index, inc_by),
                }
            }

            TypeContent::App { constructor, param } => {
                TypeContent::App {
                    constructor: constructor.increment_above(index, inc_by),
                    param: param.increment_above(index, inc_by),
                }
            }
        };

        Type::from_content(new_content)
    }

    fn increment_bound(&self, inc_by: usize) -> Self {
        self.increment_above(self.free, inc_by)
    }

    pub fn accomodate_free(&self, new_free: usize) -> Self {
        assert!(self.free <= new_free);
        self.increment_bound(new_free - self.free)
    }

    pub fn subst(&self, replacements: &[Type<Name>]) -> Self {
        assert!(replacements.len() <= self.free);
        for replacement in replacements {
            // TODO: Assess whether or not this check is actually necessary or desireable
            assert_eq!(
                replacement.free,
                self.free - replacements.len(),
                "Free variables do not match"
            );
        }
        self.subst_inner(self.free - replacements.len(), replacements)
    }

    fn subst_inner(&self, start_index: usize, replacements: &[Type<Name>]) -> Self {
        if self.data.max_index <= start_index {
            return Type {
                free: self.free - replacements.len(),
                data: self.data.clone(),
            };
        }

        match self.to_content() {
            TypeContent::Var { free, index } => {
                let new_free = free - replacements.len();
                if start_index + replacements.len() <= index {
                    Type::from_content(TypeContent::Var {
                        free: new_free,
                        index: index - replacements.len(),
                    })
                } else if index < start_index {
                    Type::from_content(TypeContent::Var {
                        free: new_free,
                        index,
                    })
                } else {
                    // index lies inside substitution range
                    debug_assert!(start_index <= index && index < start_index + replacements.len());
                    let replacement = &replacements[index - start_index];
                    replacement.accomodate_free(new_free)
                }
            }

            TypeContent::Exists { param, body } => {
                Type::from_content(TypeContent::Exists {
                    param,
                    body: body.subst_inner(start_index, replacements),
                })
            }

            TypeContent::Func { params, arg, ret } => {
                Type::from_content(TypeContent::Func {
                    params,
                    arg: arg.subst_inner(start_index, replacements),
                    ret: ret.subst_inner(start_index, replacements),
                })
            }

            TypeContent::Pair { left, right } => {
                Type::from_content(TypeContent::Pair {
                    left: left.subst_inner(start_index, replacements),
                    right: right.subst_inner(start_index, replacements),
                })
            }

            TypeContent::App { constructor, param } => {
                Type::from_content(TypeContent::App {
                    constructor: constructor.subst_inner(start_index, replacements),
                    param: param.subst_inner(start_index, replacements),
                })
            }
        }
    }
}

#[cfg(test)]
mod test {
    // Convenience functions

    use super::*;

    fn var(free: usize, index: usize) -> Type<()> {
        Type::from_content(TypeContent::Var { free, index })
    }

    fn exists(kind: Kind, body: Type<()>) -> Type<()> {
        Type::from_content(TypeContent::Exists {
            param: TypeParam { name: (), kind },
            body,
        })
    }

    fn func(arg: Type<()>, ret: Type<()>) -> Type<()> {
        Type::from_content(TypeContent::Func {
            params: Rc::new(Vec::new()),
            arg,
            ret,
        })
    }

    fn func_forall(param_kinds: &[Kind], arg: Type<()>, ret: Type<()>) -> Type<()> {
        Type::from_content(TypeContent::Func {
            params: Rc::new(
                param_kinds
                    .iter()
                    .cloned()
                    .map(|kind| TypeParam { name: (), kind })
                    .collect(),
            ),
            arg,
            ret,
        })
    }

    fn pair(left: Type<()>, right: Type<()>) -> Type<()> {
        Type::from_content(TypeContent::Pair { left, right })
    }

    fn app(constructor: Type<()>, param: Type<()>) -> Type<()> {
        Type::from_content(TypeContent::App { constructor, param })
    }

    #[test]
    #[should_panic]
    fn invalid_var_1() {
        var(0, 0);
    }

    #[test]
    #[should_panic]
    fn invalid_var_2() {
        var(1, 1);
    }

    #[test]
    #[should_panic]
    fn invalid_var_3() {
        var(1, 2);
    }

    #[test]
    #[should_panic]
    fn invalid_exists() {
        exists(Kind::Type, exists(Kind::Type, var(1, 0)));
    }

    #[test]
    #[should_panic]
    fn invalid_func() {
        func(var(1, 0), var(2, 0));
    }

    #[test]
    #[should_panic]
    fn invalid_func_forall() {
        func_forall(&[Kind::Type, Kind::Type], var(1, 0), var(1, 0));
    }

    #[test]
    #[should_panic]
    fn invalid_pair() {
        pair(var(1, 0), var(2, 0));
    }

    #[test]
    #[should_panic]
    fn invalid_app() {
        app(var(1, 0), var(2, 0));
    }

    #[test]
    fn free_var() {
        assert_eq!(var(1, 0).free(), 1);
        assert_eq!(var(10, 5).free(), 10);
    }

    #[test]
    fn free_exists() {
        assert_eq!(exists(Kind::Type, var(1, 0)).free(), 0);
        assert_eq!(exists(Kind::Type, var(2, 0)).free(), 1);
        assert_eq!(exists(Kind::Type, var(3, 2)).free(), 2);

        assert_eq!(exists(Kind::Type, var(1, 0)).free(), 0);
        assert_eq!(exists(Kind::Type, var(2, 0)).free(), 1);
        assert_eq!(exists(Kind::Type, var(3, 2)).free(), 2);

        assert_eq!(exists(Kind::Type, exists(Kind::Type, var(2, 1))).free(), 0);
        assert_eq!(exists(Kind::Type, exists(Kind::Type, var(5, 1))).free(), 3);
    }

    #[test]
    fn free_func() {
        assert_eq!(func(var(2, 0), var(2, 1)).free(), 2);
        assert_eq!(func(var(4, 3), var(4, 0)).free(), 4);
    }

    #[test]
    fn free_func_forall() {
        assert_eq!(func_forall(&[Kind::Type], var(1, 0), var(1, 0)).free(), 0);

        assert_eq!(func_forall(&[Kind::Type], var(2, 0), var(2, 1)).free(), 1);

        assert_eq!(
            func_forall(&[Kind::Type, Kind::Type], var(2, 0), var(2, 1)).free(),
            0
        );
    }

    #[test]
    fn free_pair() {
        assert_eq!(pair(var(2, 0), var(2, 1)).free(), 2);
        assert_eq!(pair(var(4, 3), var(4, 0)).free(), 4);
    }

    #[test]
    fn free_app() {
        assert_eq!(app(var(2, 0), var(2, 1)).free(), 2);
        assert_eq!(app(var(4, 3), var(4, 0)).free(), 4);
    }

    #[test]
    fn accomodate_free_var() {
        assert_eq!(var(1, 0).accomodate_free(1), var(1, 0));
        assert_eq!(var(1, 0).accomodate_free(2), var(2, 0));
        assert_eq!(var(2, 0).accomodate_free(4), var(4, 0));
        assert_eq!(var(2, 1).accomodate_free(4), var(4, 1));
    }

    #[test]
    fn accomodate_free_exists() {
        assert_eq!(
            exists(Kind::Type, var(1, 0)).accomodate_free(0),
            exists(Kind::Type, var(1, 0))
        );

        assert_eq!(
            exists(Kind::Type, var(1, 0)).accomodate_free(3),
            exists(Kind::Type, var(4, 3))
        );

        assert_eq!(
            exists(Kind::Type, var(2, 0)).accomodate_free(1),
            exists(Kind::Type, var(2, 0))
        );

        assert_eq!(
            exists(Kind::Type, var(2, 0)).accomodate_free(5),
            exists(Kind::Type, var(6, 0))
        );

        assert_eq!(
            exists(Kind::Type, var(2, 1)).accomodate_free(1),
            exists(Kind::Type, var(2, 1))
        );

        assert_eq!(
            exists(Kind::Type, var(2, 1)).accomodate_free(5),
            exists(Kind::Type, var(6, 5))
        );

        assert_eq!(
            exists(
                Kind::Type,
                exists(Kind::Type, pair(pair(var(3, 0), var(3, 1)), var(3, 2))),
            ).accomodate_free(2),
            exists(
                Kind::Type,
                exists(Kind::Type, pair(pair(var(4, 0), var(4, 2)), var(4, 3))),
            )
        );
    }

    #[test]
    fn accomodate_free_func() {
        assert_eq!(
            func(var(2, 0), var(2, 1)).accomodate_free(4),
            func(var(4, 0), var(4, 1))
        );
    }

    #[test]
    fn accomodate_free_func_forall() {
        assert_eq!(
            func_forall(&[Kind::Type], var(2, 0), var(2, 1)).accomodate_free(2),
            func_forall(&[Kind::Type], var(3, 0), var(3, 2))
        );

        assert_eq!(
            func_forall(
                &[Kind::Type],
                pair(var(2, 0), var(2, 1)),
                func_forall(
                    &[Kind::Type, Kind::Type],
                    pair(var(4, 0), var(4, 1)),
                    pair(var(4, 2), var(4, 3)),
                ),
            ).accomodate_free(3),
            func_forall(
                &[Kind::Type],
                pair(var(4, 0), var(4, 3)),
                func_forall(
                    &[Kind::Type, Kind::Type],
                    pair(var(6, 0), var(6, 3)),
                    pair(var(6, 4), var(6, 5)),
                ),
            )
        );
    }

    #[test]
    fn accomodate_free_pair() {
        assert_eq!(
            pair(var(2, 0), var(2, 1)).accomodate_free(4),
            pair(var(4, 0), var(4, 1))
        );
    }

    #[test]
    fn accomodate_free_app() {
        assert_eq!(
            app(var(2, 0), var(2, 1)).accomodate_free(4),
            app(var(4, 0), var(4, 1))
        );
    }

    #[test]
    fn subst_simple() {
        assert_eq!(var(2, 0).subst(&[var(1, 0)]), var(1, 0));

        assert_eq!(var(2, 1).subst(&[var(1, 0)]), var(1, 0));

        assert_eq!(
            pair(pair(var(4, 1), var(4, 2)), var(4, 3)).subst(&[var(2, 0), var(2, 1)]),
            pair(pair(var(2, 1), var(2, 0)), var(2, 1))
        );

        assert_eq!(
            app(app(var(4, 1), var(4, 2)), var(4, 3)).subst(&[var(2, 0), var(2, 1)]),
            app(app(var(2, 1), var(2, 0)), var(2, 1))
        );

        assert_eq!(
            pair(pair(var(4, 1), var(4, 2)), var(4, 3)).subst(
                &[
                    pair(
                        var(2, 0),
                        var(2, 0),
                    ),
                    var(2, 1),
                ],
            ),
            pair(pair(var(2, 1), pair(var(2, 0), var(2, 0))), var(2, 1))
        );

        assert_eq!(
            func(func(var(4, 1), var(4, 2)), var(4, 3)).subst(
                &[
                    pair(
                        var(2, 0),
                        var(2, 0),
                    ),
                    var(2, 1),
                ],
            ),
            func(func(var(2, 1), pair(var(2, 0), var(2, 0))), var(2, 1))
        );
    }

    #[test]
    fn subst_exists() {
        assert_eq!(
            exists(Kind::Type, pair(var(3, 1), var(3, 2))).subst(&[pair(var(1, 0), var(1, 0))]),
            exists(Kind::Type, pair(pair(var(2, 0), var(2, 0)), var(2, 1)))
        );

        assert_eq!(
            pair(var(2, 0), var(2, 1)).subst(&[exists(Kind::Type, var(2, 1))]),
            pair(var(1, 0), exists(Kind::Type, var(2, 1)))
        );

        assert_eq!(
            exists(Kind::Type, pair(var(3, 0), pair(var(3, 1), var(3, 2))))
                .subst(&[exists(Kind::Type, pair(var(2, 0), var(2, 1)))]),
            exists(
                Kind::Type,
                pair(
                    var(2, 0),
                    pair(exists(Kind::Type, pair(var(3, 0), var(3, 2))), var(2, 1)),
                ),
            )
        );
    }

    #[test]
    fn subst_func_forall() {
        assert_eq!(
            func_forall(&[Kind::Type], var(3, 1), var(3, 2)).subst(&[var(1, 0)]),
            func_forall(&[Kind::Type], var(2, 0), var(2, 1))
        );

        assert_eq!(
            func_forall(&[Kind::Type], var(3, 2), var(3, 1)).subst(&[var(1, 0)]),
            func_forall(&[Kind::Type], var(2, 1), var(2, 0))
        );

        assert_eq!(
            func_forall(&[Kind::Type], var(2, 0), var(2, 1)).subst(
                &[func_forall(&[Kind::Type], var(1, 0), var(1, 0))],
            ),
            func_forall(
                &[Kind::Type],
                func_forall(&[Kind::Type], var(2, 1), var(2, 1)),
                var(1, 0),
            )
        );

        assert_eq!(
            func_forall(&[Kind::Type], var(3, 1), var(3, 2)).subst(
                &[func_forall(&[Kind::Type], var(2, 0), var(2, 1))],
            ),
            func_forall(
                &[Kind::Type],
                func_forall(&[Kind::Type], var(3, 0), var(3, 2)),
                var(2, 1),
            )
        );
    }
}
