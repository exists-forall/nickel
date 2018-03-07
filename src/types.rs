use std::rc::Rc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    Type,
    Place,
    Version,
    Cons(Rc<Kind>, Rc<Kind>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quantifier {
    ForAll,
    Exists,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FuncAccess {
    Many,
    Once,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TypeDataInner {
    Var { index: usize },
    Quantified {
        quantifier: Quantifier,
        kind: Kind,
        body: TypeData,
    },
    Func {
        access: FuncAccess,
        arg: TypeData,
        ret: TypeData,
    },
    Pair { left: TypeData, right: TypeData },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TypeData {
    max_index: usize, // exclusive upper bound
    inner: Rc<TypeDataInner>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeContent {
    Var { free: usize, index: usize },
    Quantified {
        quantifier: Quantifier,
        kind: Kind,
        body: Type,
    },
    Func {
        access: FuncAccess,
        arg: Type,
        ret: Type,
    },
    Pair { left: Type, right: Type },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Type {
    free: usize,
    data: TypeData,
}

impl Type {
    pub fn free(&self) -> usize {
        self.free
    }

    pub fn from_content(content: TypeContent) -> Self {
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

            TypeContent::Quantified {
                quantifier,
                kind,
                body,
            } => {
                assert!(1 <= body.free, "Must have at least one free variable");
                Type {
                    free: body.free - 1,
                    data: TypeData {
                        max_index: body.data.max_index,
                        inner: Rc::new(TypeDataInner::Quantified {
                            quantifier,
                            kind,
                            body: body.data,
                        }),
                    },
                }
            }

            TypeContent::Func { access, arg, ret } => {
                assert_eq!(arg.free, ret.free, "Free variables do not match");
                Type {
                    free: arg.free,
                    data: TypeData {
                        max_index: arg.data.max_index.max(ret.data.max_index),
                        inner: Rc::new(TypeDataInner::Func {
                            access,
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
        }
    }

    pub fn to_content(&self) -> TypeContent {
        match &*self.data.inner {
            &TypeDataInner::Var { index } => {
                TypeContent::Var {
                    free: self.free,
                    index,
                }
            }

            &TypeDataInner::Quantified {
                quantifier,
                ref kind,
                ref body,
            } => {
                TypeContent::Quantified {
                    quantifier,
                    kind: kind.clone(),
                    body: Type {
                        free: self.free + 1,
                        data: body.clone(),
                    },
                }
            }

            &TypeDataInner::Func {
                access,
                ref arg,
                ref ret,
            } => {
                TypeContent::Func {
                    access,
                    arg: Type {
                        free: self.free,
                        data: arg.clone(),
                    },
                    ret: Type {
                        free: self.free,
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

            TypeContent::Quantified {
                quantifier,
                kind,
                body,
            } => {
                TypeContent::Quantified {
                    quantifier,
                    kind,
                    body: body.increment_above(index, inc_by),
                }
            }

            TypeContent::Func { access, arg, ret } => {
                TypeContent::Func {
                    access,
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

    pub fn subst(&self, replacements: &[Type]) -> Self {
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

    fn subst_inner(&self, start_index: usize, replacements: &[Type]) -> Self {
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

            TypeContent::Quantified {
                quantifier,
                kind,
                body,
            } => {
                Type::from_content(TypeContent::Quantified {
                    quantifier,
                    kind,
                    body: body.subst_inner(start_index, replacements),
                })
            }

            TypeContent::Func { access, arg, ret } => {
                Type::from_content(TypeContent::Func {
                    access,
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
        }
    }
}

#[cfg(test)]
mod test {
    // Convenience functions

    use super::*;
    use super::FuncAccess::*;
    use super::Quantifier::*;

    fn var(free: usize, index: usize) -> Type {
        Type::from_content(TypeContent::Var { free, index })
    }

    fn quant(quantifier: Quantifier, kind: Kind, body: Type) -> Type {
        Type::from_content(TypeContent::Quantified {
            quantifier,
            kind,
            body,
        })
    }

    fn func(access: FuncAccess, arg: Type, ret: Type) -> Type {
        Type::from_content(TypeContent::Func { access, arg, ret })
    }

    fn pair(left: Type, right: Type) -> Type {
        Type::from_content(TypeContent::Pair { left, right })
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
    fn invalid_quant() {
        quant(ForAll, Kind::Type, quant(ForAll, Kind::Type, var(1, 0)));
    }

    #[test]
    #[should_panic]
    fn invalid_func() {
        func(Many, var(1, 0), var(2, 0));
    }

    #[test]
    #[should_panic]
    fn invalid_pair() {
        pair(var(1, 0), var(2, 0));
    }

    #[test]
    fn free_var() {
        assert_eq!(var(1, 0).free(), 1);
        assert_eq!(var(10, 5).free(), 10);
    }

    #[test]
    fn free_quant() {
        assert_eq!(quant(ForAll, Kind::Type, var(1, 0)).free(), 0);
        assert_eq!(quant(ForAll, Kind::Type, var(2, 0)).free(), 1);
        assert_eq!(quant(ForAll, Kind::Type, var(3, 2)).free(), 2);

        assert_eq!(quant(Exists, Kind::Type, var(1, 0)).free(), 0);
        assert_eq!(quant(Exists, Kind::Type, var(2, 0)).free(), 1);
        assert_eq!(quant(Exists, Kind::Type, var(3, 2)).free(), 2);

        assert_eq!(
            quant(ForAll, Kind::Type, quant(ForAll, Kind::Type, var(2, 1))).free(),
            0
        );
        assert_eq!(
            quant(ForAll, Kind::Type, quant(ForAll, Kind::Type, var(5, 1))).free(),
            3
        );
    }

    #[test]
    fn free_func() {
        assert_eq!(func(Many, var(2, 0), var(2, 1)).free(), 2);
        assert_eq!(func(Once, var(4, 3), var(4, 0)).free(), 4);
    }

    #[test]
    fn free_pair() {
        assert_eq!(pair(var(2, 0), var(2, 1)).free(), 2);
        assert_eq!(pair(var(4, 3), var(4, 0)).free(), 4);
    }

    #[test]
    fn accomodate_free_var() {
        assert_eq!(var(1, 0).accomodate_free(1), var(1, 0));
        assert_eq!(var(1, 0).accomodate_free(2), var(2, 0));
        assert_eq!(var(2, 0).accomodate_free(4), var(4, 0));
        assert_eq!(var(2, 1).accomodate_free(4), var(4, 1));
    }

    #[test]
    fn accomodate_free_quant() {
        assert_eq!(
            quant(ForAll, Kind::Type, var(1, 0)).accomodate_free(0),
            quant(ForAll, Kind::Type, var(1, 0))
        );

        assert_eq!(
            quant(ForAll, Kind::Type, var(1, 0)).accomodate_free(3),
            quant(ForAll, Kind::Type, var(4, 3))
        );

        assert_eq!(
            quant(ForAll, Kind::Type, var(2, 0)).accomodate_free(1),
            quant(ForAll, Kind::Type, var(2, 0))
        );

        assert_eq!(
            quant(ForAll, Kind::Type, var(2, 0)).accomodate_free(5),
            quant(ForAll, Kind::Type, var(6, 0))
        );

        assert_eq!(
            quant(ForAll, Kind::Type, var(2, 1)).accomodate_free(1),
            quant(ForAll, Kind::Type, var(2, 1))
        );

        assert_eq!(
            quant(ForAll, Kind::Type, var(2, 1)).accomodate_free(5),
            quant(ForAll, Kind::Type, var(6, 5))
        );

        assert_eq!(
            quant(
                ForAll,
                Kind::Type,
                quant(
                    ForAll,
                    Kind::Type,
                    pair(pair(var(3, 0), var(3, 1)), var(3, 2)),
                ),
            ).accomodate_free(2),
            quant(
                ForAll,
                Kind::Type,
                quant(
                    ForAll,
                    Kind::Type,
                    pair(pair(var(4, 0), var(4, 2)), var(4, 3)),
                ),
            )
        );
    }

    #[test]
    fn accomodate_free_func() {
        assert_eq!(
            func(Many, var(2, 0), var(2, 1)).accomodate_free(4),
            func(Many, var(4, 0), var(4, 1))
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
    fn subst_simple() {
        assert_eq!(var(2, 0).subst(&[var(1, 0)]), var(1, 0));

        assert_eq!(var(2, 1).subst(&[var(1, 0)]), var(1, 0));

        assert_eq!(
            pair(pair(var(4, 1), var(4, 2)), var(4, 3)).subst(&[var(2, 0), var(2, 1)]),
            pair(pair(var(2, 1), var(2, 0)), var(2, 1))
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
            func(Many, func(Once, var(4, 1), var(4, 2)), var(4, 3))
                .subst(&[pair(var(2, 0), var(2, 0)), var(2, 1)]),
            func(
                Many,
                func(Once, var(2, 1), pair(var(2, 0), var(2, 0))),
                var(2, 1),
            )
        );
    }

    #[test]
    fn subst_quant() {
        assert_eq!(
            quant(ForAll, Kind::Type, pair(var(3, 1), var(3, 2)))
                .subst(&[pair(var(1, 0), var(1, 0))]),
            quant(
                ForAll,
                Kind::Type,
                pair(pair(var(2, 0), var(2, 0)), var(2, 1)),
            )
        );

        assert_eq!(
            pair(var(2, 0), var(2, 1)).subst(&[quant(ForAll, Kind::Type, var(2, 1))]),
            pair(var(1, 0), quant(ForAll, Kind::Type, var(2, 1)))
        );

        assert_eq!(
            quant(
                ForAll,
                Kind::Type,
                pair(var(3, 0), pair(var(3, 1), var(3, 2))),
            ).subst(&[quant(ForAll, Kind::Type, pair(var(2, 0), var(2, 1)))]),
            quant(
                ForAll,
                Kind::Type,
                pair(
                    var(2, 0),
                    pair(
                        quant(ForAll, Kind::Type, pair(var(3, 0), var(3, 2))),
                        var(2, 1),
                    ),
                ),
            )
        );
    }
}
