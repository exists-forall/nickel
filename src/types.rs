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
pub enum TypeContent<Free> {
    Free { free: Free },
    Var { index: usize },
    Quantified {
        quantifier: Quantifier,
        kind: Kind,
        body: Type<Free>,
    },
    Func {
        access: FuncAccess,
        arg: Type<Free>,
        ret: Type<Free>,
    },
    Pair { left: Type<Free>, right: Type<Free> },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Type<Free> {
    offset: Option<usize>,
    content: Rc<TypeContent<Free>>,
}

fn add_offsets(off1: Option<usize>, off2: Option<usize>) -> Option<usize> {
    match (off1, off2) {
        (Some(a), Some(b)) => Some(a + b),
        (_, _) => None,
    }
}

fn remove_common(off1: &mut Option<usize>, off2: &mut Option<usize>) -> Option<usize> {
    let (new_off1, new_off2, common) = match (*off1, *off2) {
        (Some(a), Some(b)) => {
            let common = a.min(b);
            (Some(a - common), Some(b - common), Some(common))
        }
        (Some(a), None) => (Some(0), None, Some(a)),
        (None, Some(b)) => (None, Some(0), Some(b)),
        (_, _) => {
            // This introduces the easy-to-check optimality condition that at least one of off1 and
            // off2 always ends up set to 0.
            (Some(0), Some(0), None)
        }
    };

    *off1 = new_off1;
    *off2 = new_off2;
    common
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Occurrence {
    CertainlyAbsent,
    PossiblyPresent,
}

impl<Free: Clone> Type<Free> {
    pub fn from_content(content: TypeContent<Free>) -> Self {
        match content {
            TypeContent::Free { free } => {
                Type {
                    offset: None,
                    content: Rc::new(TypeContent::Free { free }),
                }
            }

            TypeContent::Var { index } => {
                Type {
                    offset: Some(index),
                    content: Rc::new(TypeContent::Var { index: 0 }),
                }
            }

            TypeContent::Quantified {
                quantifier,
                kind,
                body,
            } => {
                Type {
                    offset: body.offset,
                    content: Rc::new(TypeContent::Quantified {
                        quantifier,
                        kind,
                        body: Type {
                            offset: Some(0),
                            content: body.content,
                        },
                    }),
                }
            }

            TypeContent::Func {
                access,
                mut arg,
                mut ret,
            } => {
                let offset = remove_common(&mut arg.offset, &mut ret.offset);
                Type {
                    offset,
                    content: Rc::new(TypeContent::Func { access, arg, ret }),
                }
            }

            TypeContent::Pair {
                mut left,
                mut right,
            } => {
                let offset = remove_common(&mut left.offset, &mut right.offset);
                Type {
                    offset,
                    content: Rc::new(TypeContent::Pair { left, right }),
                }
            }
        }
    }

    pub fn to_content(&self) -> TypeContent<Free> {
        match &*self.content {
            &TypeContent::Free { ref free } => TypeContent::Free { free: free.clone() },

            &TypeContent::Var { index } => {
                debug_assert_eq!(index, 0);
                let offset = self.offset.expect(
                    "Cannot have a variable as a leaf of a tree with no minimum index",
                );
                TypeContent::Var { index: index + offset }
            }

            &TypeContent::Quantified {
                quantifier,
                ref kind,
                ref body,
            } => {
                debug_assert_eq!(body.offset, Some(0));
                TypeContent::Quantified {
                    quantifier,
                    kind: kind.clone(),
                    body: Type {
                        offset: add_offsets(body.offset, self.offset),
                        content: body.content.clone(),
                    },
                }
            }

            &TypeContent::Func {
                access,
                ref arg,
                ref ret,
            } => {
                debug_assert!(arg.offset == Some(0) || ret.offset == Some(0));
                TypeContent::Func {
                    access,
                    arg: Type {
                        offset: add_offsets(arg.offset, self.offset),
                        content: arg.content.clone(),
                    },
                    ret: Type {
                        offset: add_offsets(ret.offset, self.offset),
                        content: ret.content.clone(),
                    },
                }
            }

            &TypeContent::Pair {
                ref left,
                ref right,
            } => {
                TypeContent::Pair {
                    left: Type {
                        offset: add_offsets(left.offset, self.offset),
                        content: left.content.clone(),
                    },
                    right: Type {
                        offset: add_offsets(right.offset, self.offset),
                        content: right.content.clone(),
                    },
                }
            }
        }
    }

    pub fn var_occurs(&self, index: usize) -> Occurrence {
        if let Some(offset) = self.offset {
            if index >= offset {
                return Occurrence::PossiblyPresent;
            }
        }
        Occurrence::CertainlyAbsent
    }

    pub fn decrement_above(self, index: usize) -> Type<Free> {
        if let Some(offset) = self.offset {
            if index > offset {
                let new_content = match self.to_content() {
                    TypeContent::Free { free: _ } => {
                        unreachable!("A free variable should never have a minimum index");
                    }
                    TypeContent::Var { index: var_index } => {
                        debug_assert_eq!(var_index, offset);
                        TypeContent::Var { index: var_index - 1 }
                    }
                    TypeContent::Quantified {
                        quantifier,
                        kind,
                        body,
                    } => {
                        TypeContent::Quantified {
                            quantifier,
                            kind,
                            body: body.decrement_above(index),
                        }
                    }
                    TypeContent::Func { access, arg, ret } => {
                        TypeContent::Func {
                            access,
                            arg: arg.decrement_above(index),
                            ret: ret.decrement_above(index),
                        }
                    }
                    TypeContent::Pair { left, right } => {
                        TypeContent::Pair {
                            left: left.decrement_above(index),
                            right: right.decrement_above(index),
                        }
                    }
                };
                Type::from_content(new_content)
            } else if index < offset {
                Type {
                    offset: Some(offset - 1),
                    content: self.content,
                }
            } else {
                panic!("`decrement_above` given a variable which occurs in the type");
            }
        } else {
            self
        }
    }
}
