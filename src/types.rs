use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeParam<Name> {
    pub name: Name,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quantifier {
    Exists,
    ForAll,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Static,
    Dynamic,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TypeDataInner<TAnnot, Name> {
    Unit,
    Var { index: usize },
    Quantified {
        quantifier: Quantifier,
        param: TypeParam<Name>,
        body: TypeData<TAnnot, Name>,
    },
    Func {
        arg: TypeData<TAnnot, Name>,
        arg_phase: Phase,
        ret: TypeData<TAnnot, Name>,
        ret_phase: Phase,
    },
    Pair {
        left: TypeData<TAnnot, Name>,
        right: TypeData<TAnnot, Name>,
    },
    App {
        constructor: TypeData<TAnnot, Name>,
        param: TypeData<TAnnot, Name>,
    },
    Equiv {
        orig: TypeData<TAnnot, Name>,
        dest: TypeData<TAnnot, Name>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TypeData<TAnnot, Name> {
    annot: TAnnot,
    max_index: usize, // exclusive upper bound
    inner: Rc<TypeDataInner<TAnnot, Name>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeContent<TAnnot, Name> {
    Unit { free: usize },
    Var { free: usize, index: usize },
    Quantified {
        quantifier: Quantifier,
        param: TypeParam<Name>,
        body: AnnotType<TAnnot, Name>,
    },
    Func {
        arg: AnnotType<TAnnot, Name>,
        arg_phase: Phase,
        ret: AnnotType<TAnnot, Name>,
        ret_phase: Phase,
    },
    Pair {
        left: AnnotType<TAnnot, Name>,
        right: AnnotType<TAnnot, Name>,
    },
    App {
        constructor: AnnotType<TAnnot, Name>,
        param: AnnotType<TAnnot, Name>,
    },
    Equiv {
        orig: AnnotType<TAnnot, Name>,
        dest: AnnotType<TAnnot, Name>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnnotType<TAnnot, Name> {
    free: usize,
    data: TypeData<TAnnot, Name>,
}

pub type Type<Name> = AnnotType<(), Name>;

impl<TAnnot: Clone, Name: Clone> AnnotType<TAnnot, Name> {
    pub fn free(&self) -> usize {
        self.free
    }

    pub fn annot(&self) -> TAnnot {
        self.data.annot.clone()
    }

    pub fn from_content_annot(annot: TAnnot, content: TypeContent<TAnnot, Name>) -> Self {
        match content {
            TypeContent::Unit { free } => {
                AnnotType {
                    free,
                    data: TypeData {
                        annot,
                        max_index: 0,
                        inner: Rc::new(TypeDataInner::Unit),
                    },
                }
            }

            TypeContent::Var { free, index } => {
                assert!(index < free);
                AnnotType {
                    free,
                    data: TypeData {
                        annot,
                        max_index: index + 1,
                        inner: Rc::new(TypeDataInner::Var { index }),
                    },
                }
            }

            TypeContent::Quantified {
                quantifier,
                param,
                body,
            } => {
                assert!(1 <= body.free, "Must have at least one free variable");
                AnnotType {
                    free: body.free - 1,
                    data: TypeData {
                        annot,
                        max_index: body.data.max_index,
                        inner: Rc::new(TypeDataInner::Quantified {
                            quantifier,
                            param: param.clone(),
                            body: body.data,
                        }),
                    },
                }
            }

            TypeContent::Func {
                arg,
                arg_phase,
                ret,
                ret_phase,
            } => {
                assert_eq!(arg.free, ret.free, "Free variables do not match");
                AnnotType {
                    free: arg.free,
                    data: TypeData {
                        annot,
                        max_index: arg.data.max_index.max(ret.data.max_index),
                        inner: Rc::new(TypeDataInner::Func {
                            arg: arg.data,
                            arg_phase,
                            ret: ret.data,
                            ret_phase,
                        }),
                    },
                }
            }

            TypeContent::Pair { left, right } => {
                assert_eq!(left.free, right.free, "Free variables do not match");
                AnnotType {
                    free: left.free,
                    data: TypeData {
                        annot,
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
                AnnotType {
                    free: constructor.free,
                    data: TypeData {
                        annot,
                        max_index: constructor.data.max_index.max(param.data.max_index),
                        inner: Rc::new(TypeDataInner::App {
                            constructor: constructor.data,
                            param: param.data,
                        }),
                    },
                }
            }

            TypeContent::Equiv { orig, dest } => {
                assert_eq!(orig.free, dest.free, "Free variables do not match");
                AnnotType {
                    free: orig.free,
                    data: TypeData {
                        annot,
                        max_index: orig.data.max_index.max(dest.data.max_index),
                        inner: Rc::new(TypeDataInner::Equiv {
                            orig: orig.data,
                            dest: dest.data,
                        }),
                    },
                }
            }
        }
    }

    pub fn to_content(&self) -> TypeContent<TAnnot, Name> {
        match &*self.data.inner {
            &TypeDataInner::Unit => TypeContent::Unit { free: self.free },

            &TypeDataInner::Var { index } => {
                TypeContent::Var {
                    free: self.free,
                    index,
                }
            }

            &TypeDataInner::Quantified {
                quantifier,
                ref param,
                ref body,
            } => {
                TypeContent::Quantified {
                    quantifier,
                    param: param.clone(),
                    body: AnnotType {
                        free: self.free + 1,
                        data: body.clone(),
                    },
                }
            }

            &TypeDataInner::Func {
                ref arg,
                arg_phase,
                ref ret,
                ret_phase,
            } => {
                TypeContent::Func {
                    arg: AnnotType {
                        free: self.free,
                        data: arg.clone(),
                    },
                    arg_phase,
                    ret: AnnotType {
                        free: self.free,
                        data: ret.clone(),
                    },
                    ret_phase,
                }
            }

            &TypeDataInner::Pair {
                ref left,
                ref right,
            } => {
                TypeContent::Pair {
                    left: AnnotType {
                        free: self.free,
                        data: left.clone(),
                    },
                    right: AnnotType {
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
                    constructor: AnnotType {
                        free: self.free,
                        data: constructor.clone(),
                    },
                    param: AnnotType {
                        free: self.free,
                        data: param.clone(),
                    },
                }
            }

            &TypeDataInner::Equiv { ref orig, ref dest } => {
                TypeContent::Equiv {
                    orig: AnnotType {
                        free: self.free,
                        data: orig.clone(),
                    },
                    dest: AnnotType {
                        free: self.free,
                        data: dest.clone(),
                    },
                }
            }
        }
    }

    fn increment_above(&self, index: usize, inc_by: usize) -> Self {
        debug_assert!(index <= self.free);

        if self.data.max_index <= index {
            return AnnotType {
                free: self.free + inc_by,
                data: self.data.clone(),
            };
        }

        let new_content = match self.to_content() {
            TypeContent::Unit { free } => TypeContent::Unit { free: free + inc_by },

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

            TypeContent::Quantified {
                quantifier,
                param,
                body,
            } => {
                TypeContent::Quantified {
                    quantifier,
                    param,
                    body: body.increment_above(index, inc_by),
                }
            }

            TypeContent::Func {
                arg,
                arg_phase,
                ret,
                ret_phase,
            } => {
                TypeContent::Func {
                    arg: arg.increment_above(index, inc_by),
                    arg_phase,
                    ret: ret.increment_above(index, inc_by),
                    ret_phase,
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

            TypeContent::Equiv { orig, dest } => {
                TypeContent::Equiv {
                    orig: orig.increment_above(index, inc_by),
                    dest: dest.increment_above(index, inc_by),
                }
            }
        };

        AnnotType::from_content_annot(self.annot(), new_content)
    }

    fn increment_bound(&self, inc_by: usize) -> Self {
        self.increment_above(self.free, inc_by)
    }

    pub fn accomodate_free(&self, new_free: usize) -> Self {
        assert!(self.free <= new_free);
        self.increment_bound(new_free - self.free)
    }

    pub fn subst(&self, replacements: &[Self]) -> Self {
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

    fn subst_inner(&self, start_index: usize, replacements: &[Self]) -> Self {
        if self.data.max_index <= start_index {
            return AnnotType {
                free: self.free - replacements.len(),
                data: self.data.clone(),
            };
        }

        match self.to_content() {
            TypeContent::Unit { free } => {
                AnnotType::from_content_annot(
                    self.annot(),
                    TypeContent::Unit { free: free - replacements.len() },
                )
            }

            TypeContent::Var { free, index } => {
                let new_free = free - replacements.len();
                if start_index + replacements.len() <= index {
                    AnnotType::from_content_annot(
                        self.annot(),
                        TypeContent::Var {
                            free: new_free,
                            index: index - replacements.len(),
                        },
                    )
                } else if index < start_index {
                    AnnotType::from_content_annot(
                        self.annot(),
                        TypeContent::Var {
                            free: new_free,
                            index,
                        },
                    )
                } else {
                    // index lies inside substitution range
                    debug_assert!(start_index <= index && index < start_index + replacements.len());
                    let replacement = &replacements[index - start_index];
                    replacement.accomodate_free(new_free)
                }
            }

            TypeContent::Quantified {
                quantifier,
                param,
                body,
            } => {
                AnnotType::from_content_annot(
                    self.annot(),
                    TypeContent::Quantified {
                        quantifier,
                        param,
                        body: body.subst_inner(start_index, replacements),
                    },
                )
            }

            TypeContent::Func {
                arg,
                arg_phase,
                ret,
                ret_phase,
            } => {
                AnnotType::from_content_annot(
                    self.annot(),
                    TypeContent::Func {
                        arg: arg.subst_inner(start_index, replacements),
                        arg_phase,
                        ret: ret.subst_inner(start_index, replacements),
                        ret_phase,
                    },
                )
            }

            TypeContent::Pair { left, right } => {
                AnnotType::from_content_annot(
                    self.annot(),
                    TypeContent::Pair {
                        left: left.subst_inner(start_index, replacements),
                        right: right.subst_inner(start_index, replacements),
                    },
                )
            }

            TypeContent::App { constructor, param } => {
                AnnotType::from_content_annot(
                    self.annot(),
                    TypeContent::App {
                        constructor: constructor.subst_inner(start_index, replacements),
                        param: param.subst_inner(start_index, replacements),
                    },
                )
            }

            TypeContent::Equiv { orig, dest } => {
                AnnotType::from_content_annot(
                    self.annot(),
                    TypeContent::Equiv {
                        orig: orig.subst_inner(start_index, replacements),
                        dest: dest.subst_inner(start_index, replacements),
                    },
                )
            }
        }
    }
}

impl<Name: Clone> Type<Name> {
    pub fn from_content(content: TypeContent<(), Name>) -> Self {
        AnnotType::from_content_annot((), content)
    }
}

#[cfg(test)]
mod test {
    use test_utils::types::*;

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
        exists(exists(var(1, 0)));
    }

    #[test]
    #[should_panic]
    fn invalid_func() {
        func(var(1, 0), var(2, 0));
    }

    #[test]
    #[should_panic]
    fn invalid_func_forall() {
        func_forall(2, var(1, 0), var(1, 0));
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
    #[should_panic]
    fn invalid_equiv() {
        equiv_ty(var(1, 0), var(2, 0));
    }

    #[test]
    fn free_unit() {
        assert_eq!(unit(0).free(), 0);
        assert_eq!(unit(5).free(), 5);
    }

    #[test]
    fn free_var() {
        assert_eq!(var(1, 0).free(), 1);
        assert_eq!(var(10, 5).free(), 10);
    }

    #[test]
    fn free_exists() {
        assert_eq!(exists(var(1, 0)).free(), 0);
        assert_eq!(exists(var(2, 0)).free(), 1);
        assert_eq!(exists(var(3, 2)).free(), 2);

        assert_eq!(exists(var(1, 0)).free(), 0);
        assert_eq!(exists(var(2, 0)).free(), 1);
        assert_eq!(exists(var(3, 2)).free(), 2);

        assert_eq!(exists(exists(var(2, 1))).free(), 0);
        assert_eq!(exists(exists(var(5, 1))).free(), 3);
    }

    #[test]
    fn free_func() {
        assert_eq!(func(var(2, 0), var(2, 1)).free(), 2);
        assert_eq!(func(var(4, 3), var(4, 0)).free(), 4);
    }

    #[test]
    fn free_func_forall() {
        assert_eq!(func_forall(1, var(1, 0), var(1, 0)).free(), 0);
        assert_eq!(func_forall(1, var(2, 0), var(2, 1)).free(), 1);
        assert_eq!(func_forall(2, var(2, 0), var(2, 1)).free(), 0);
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
    fn free_equiv() {
        assert_eq!(equiv_ty(var(2, 0), var(2, 1)).free(), 2);
        assert_eq!(equiv_ty(var(4, 3), var(4, 0)).free(), 4);
    }

    #[test]
    fn accomodate_free_unit() {
        assert_eq!(unit(0).accomodate_free(0), unit(0));
        assert_eq!(unit(10).accomodate_free(10), unit(10));
        assert_eq!(unit(3).accomodate_free(5), unit(5));
        assert_eq!(unit(0).accomodate_free(5), unit(5));
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
        assert_eq!(exists(var(1, 0)).accomodate_free(0), exists(var(1, 0)));
        assert_eq!(exists(var(1, 0)).accomodate_free(3), exists(var(4, 3)));
        assert_eq!(exists(var(2, 0)).accomodate_free(1), exists(var(2, 0)));
        assert_eq!(exists(var(2, 0)).accomodate_free(5), exists(var(6, 0)));
        assert_eq!(exists(var(2, 1)).accomodate_free(1), exists(var(2, 1)));
        assert_eq!(exists(var(2, 1)).accomodate_free(5), exists(var(6, 5)));
        assert_eq!(
            exists(exists(pair(pair(var(3, 0), var(3, 1)), var(3, 2)))).accomodate_free(2),
            exists(exists(pair(pair(var(4, 0), var(4, 2)), var(4, 3))))
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
            func_forall(1, var(2, 0), var(2, 1)).accomodate_free(2),
            func_forall(1, var(3, 0), var(3, 2))
        );

        assert_eq!(
            func_forall(
                1,
                pair(var(2, 0), var(2, 1)),
                func_forall(2, pair(var(4, 0), var(4, 1)), pair(var(4, 2), var(4, 3))),
            ).accomodate_free(3),
            func_forall(
                1,
                pair(var(4, 0), var(4, 3)),
                func_forall(2, pair(var(6, 0), var(6, 3)), pair(var(6, 4), var(6, 5))),
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
    fn accomodate_free_equiv() {
        assert_eq!(
            equiv_ty(var(2, 0), var(2, 1)).accomodate_free(4),
            equiv_ty(var(4, 0), var(4, 1))
        );
    }

    #[test]
    fn subst_simple() {
        assert_eq!(unit(4).subst(&[var(2, 0), var(2, 1)]), unit(2));

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
            exists(pair(var(3, 1), var(3, 2))).subst(&[pair(var(1, 0), var(1, 0))]),
            exists(pair(pair(var(2, 0), var(2, 0)), var(2, 1)))
        );

        assert_eq!(
            pair(var(2, 0), var(2, 1)).subst(&[exists(var(2, 1))]),
            pair(var(1, 0), exists(var(2, 1)))
        );

        assert_eq!(
            exists(pair(var(3, 0), pair(var(3, 1), var(3, 2)))).subst(
                &[
                    exists(
                        pair(
                            var(
                                2,
                                0,
                            ),
                            var(
                                2,
                                1,
                            ),
                        ),
                    ),
                ],
            ),
            exists(pair(
                var(2, 0),
                pair(exists(pair(var(3, 0), var(3, 2))), var(2, 1)),
            ))
        );
    }

    #[test]
    fn subst_func_forall() {
        assert_eq!(
            func_forall(1, var(3, 1), var(3, 2)).subst(&[var(1, 0)]),
            func_forall(1, var(2, 0), var(2, 1))
        );

        assert_eq!(
            func_forall(1, var(3, 2), var(3, 1)).subst(&[var(1, 0)]),
            func_forall(1, var(2, 1), var(2, 0))
        );

        assert_eq!(
            func_forall(1, var(2, 0), var(2, 1)).subst(&[func_forall(1, var(1, 0), var(1, 0))]),
            func_forall(1, func_forall(1, var(2, 1), var(2, 1)), var(1, 0))
        );

        assert_eq!(
            func_forall(1, var(3, 1), var(3, 2)).subst(&[func_forall(1, var(2, 0), var(2, 1))]),
            func_forall(1, func_forall(1, var(3, 0), var(3, 2)), var(2, 1))
        );
    }
}
