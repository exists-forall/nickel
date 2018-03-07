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

    pub fn subst(&self, start_index: usize, replacements: &[Type]) -> Self {
        assert!(start_index + replacements.len() <= self.free);
        for replacement in replacements {
            // TODO: Assess whether or not this check is actually necessary or desireable
            assert_eq!(replacement.free, self.free, "Free variables do not match");
        }
        self.subst_inner(start_index, replacements)
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
