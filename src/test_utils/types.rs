use std::rc::Rc;

use types::*;

pub fn unit(free: usize) -> Type<Rc<String>> {
    Type::from_content(TypeContent::Unit { free })
}

pub fn var(free: usize, index: usize) -> Type<Rc<String>> {
    Type::from_content(TypeContent::Var { free, index })
}

pub fn exists(kind: Kind, body: Type<Rc<String>>) -> Type<Rc<String>> {
    Type::from_content(TypeContent::Quantified {
        quantifier: Quantifier::Exists,
        param: TypeParam {
            name: Rc::new("".to_owned()),
            kind,
        },
        body,
    })
}

pub fn exists_named(name: &str, kind: Kind, body: Type<Rc<String>>) -> Type<Rc<String>> {
    Type::from_content(TypeContent::Quantified {
        quantifier: Quantifier::Exists,
        param: TypeParam {
            name: Rc::new(name.to_owned()),
            kind,
        },
        body,
    })
}

pub fn func(arg: Type<Rc<String>>, ret: Type<Rc<String>>) -> Type<Rc<String>> {
    Type::from_content(TypeContent::Func {
        params: Rc::new(Vec::new()),
        arg,
        ret,
    })
}

pub fn func_forall(
    param_kinds: &[Kind],
    arg: Type<Rc<String>>,
    ret: Type<Rc<String>>,
) -> Type<Rc<String>> {
    Type::from_content(TypeContent::Func {
        params: Rc::new(
            param_kinds
                .iter()
                .cloned()
                .map(|kind| {
                    TypeParam {
                        name: Rc::new("".to_owned()),
                        kind,
                    }
                })
                .collect(),
        ),
        arg,
        ret,
    })
}

pub fn func_forall_named(
    params: &[(&str, Kind)],
    arg: Type<Rc<String>>,
    ret: Type<Rc<String>>,
) -> Type<Rc<String>> {
    Type::from_content(TypeContent::Func {
        params: Rc::new(
            params
                .iter()
                .cloned()
                .map(|(name, kind)| {
                    TypeParam {
                        name: Rc::new(name.to_owned()),
                        kind,
                    }
                })
                .collect(),
        ),
        arg,
        ret,
    })
}

pub fn pair(left: Type<Rc<String>>, right: Type<Rc<String>>) -> Type<Rc<String>> {
    Type::from_content(TypeContent::Pair { left, right })
}

pub fn app(constructor: Type<Rc<String>>, param: Type<Rc<String>>) -> Type<Rc<String>> {
    Type::from_content(TypeContent::App { constructor, param })
}
