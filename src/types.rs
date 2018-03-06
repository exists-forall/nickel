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
pub enum TypeContent {
    Var { index: usize },
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Type {
    offset: Option<usize>,
    content: Rc<TypeContent>,
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

impl Type {
    pub fn from_content(content: TypeContent) -> Self {
        match content {
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
        }
    }

    pub fn to_content(&self) -> TypeContent {
        match &*self.content {
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
        }
    }
}
