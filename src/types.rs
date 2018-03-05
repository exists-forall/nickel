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
    offset: usize,
    content: Rc<TypeContent>,
}

impl Type {
    pub fn from_content(content: TypeContent) -> Self {
        match content {
            TypeContent::Var { index } => {
                Type {
                    offset: index,
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
                            offset: 0,
                            content: body.content,
                        },
                    }),
                }
            }

            TypeContent::Func { access, arg, ret } => {
                let offset = arg.offset.min(ret.offset);
                Type {
                    offset,
                    content: Rc::new(TypeContent::Func {
                        access,
                        arg: Type {
                            offset: arg.offset - offset,
                            content: arg.content,
                        },
                        ret: Type {
                            offset: ret.offset - offset,
                            content: ret.content,
                        },
                    }),
                }
            }
        }
    }

    pub fn to_content(&self) -> TypeContent {
        match &*self.content {
            &TypeContent::Var { index } => {
                debug_assert_eq!(index, 0);
                TypeContent::Var { index: index + self.offset }
            }

            &TypeContent::Quantified {
                quantifier,
                ref kind,
                ref body,
            } => {
                debug_assert_eq!(body.offset, 0);
                TypeContent::Quantified {
                    quantifier,
                    kind: kind.clone(),
                    body: Type {
                        offset: body.offset + self.offset,
                        content: body.content.clone(),
                    },
                }
            }

            &TypeContent::Func {
                access,
                ref arg,
                ref ret,
            } => {
                debug_assert!(arg.offset == 0 || ret.offset == 0);
                TypeContent::Func {
                    access,
                    arg: Type {
                        offset: arg.offset + self.offset,
                        content: arg.content.clone(),
                    },
                    ret: Type {
                        offset: ret.offset + self.offset,
                        content: ret.content.clone(),
                    },
                }
            }
        }
    }
}
