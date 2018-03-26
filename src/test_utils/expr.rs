use std::rc::Rc;
use std::iter::repeat;

use expr::*;
use types::*;

pub fn unit(free_vars: usize, free_types: usize) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::Unit {
        free_vars,
        free_types,
    })
}

pub fn var(usage: VarUsage, free_vars: usize, free_types: usize, index: usize) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::Var {
        usage,
        free_vars,
        free_types,
        index,
    })
}

pub fn func(arg_type: Type<Rc<String>>, body: Expr<Rc<String>>) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::Func {
        type_params: Rc::new(Vec::new()),
        arg_name: Rc::new("".to_owned()),
        arg_type,
        body,
    })
}

pub fn func_forall(
    param_kinds: &[Kind],
    arg_type: Type<Rc<String>>,
    body: Expr<Rc<String>>,
) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::Func {
        type_params: Rc::new(
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
        arg_name: Rc::new("".to_owned()),
        arg_type,
        body,
    })
}

pub fn func_named(
    arg_name: &str,
    arg_type: Type<Rc<String>>,
    body: Expr<Rc<String>>,
) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::Func {
        type_params: Rc::new(Vec::new()),
        arg_name: Rc::new(arg_name.to_owned()),
        arg_type,
        body,
    })
}

pub fn func_forall_named(
    params: &[(&str, Kind)],
    arg_name: &str,
    arg_type: Type<Rc<String>>,
    body: Expr<Rc<String>>,
) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::Func {
        type_params: Rc::new(
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
        arg_name: Rc::new(arg_name.to_owned()),
        arg_type,
        body,
    })
}

pub fn app(callee: Expr<Rc<String>>, arg: Expr<Rc<String>>) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::App {
        callee,
        type_params: Rc::new(Vec::new()),
        arg,
    })
}

pub fn app_forall(
    callee: Expr<Rc<String>>,
    type_params: &[Type<Rc<String>>],
    arg: Expr<Rc<String>>,
) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::App {
        callee,
        type_params: Rc::new(type_params.iter().cloned().collect()),
        arg,
    })
}

pub fn pair(left: Expr<Rc<String>>, right: Expr<Rc<String>>) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::Pair { left, right })
}

pub fn let_vars(count: usize, val: Expr<Rc<String>>, body: Expr<Rc<String>>) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::Let {
        names: Rc::new(repeat(Rc::new("".to_owned())).take(count).collect()),
        val,
        body,
    })
}

pub fn let_vars_named(
    names: &[&str],
    val: Expr<Rc<String>>,
    body: Expr<Rc<String>>,
) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::Let {
        names: Rc::new(names.iter().map(|&name| Rc::new(name.to_owned())).collect()),
        val,
        body,
    })
}

pub fn let_exists(
    type_count: usize,
    val: Expr<Rc<String>>,
    body: Expr<Rc<String>>,
) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::LetExists {
        type_names: Rc::new(repeat(Rc::new("".to_owned())).take(type_count).collect()),
        val_name: Rc::new("".to_owned()),
        val,
        body,
    })
}

pub fn let_exists_named(
    type_names: &[&str],
    val_name: &str,
    val: Expr<Rc<String>>,
    body: Expr<Rc<String>>,
) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::LetExists {
        type_names: Rc::new(
            type_names
                .iter()
                .map(|&name| Rc::new(name.to_owned()))
                .collect(),
        ),
        val_name: Rc::new(val_name.to_owned()),
        val,
        body,
    })
}

pub fn make_exists(
    params: &[Type<Rc<String>>],
    type_body: Type<Rc<String>>,
    body: Expr<Rc<String>>,
) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::MakeExists {
        params: Rc::new(
            params
                .iter()
                .cloned()
                .map(|ty| (Rc::new("".to_owned()), ty))
                .collect(),
        ),
        type_body,
        body,
    })
}

pub fn make_exists_named(
    params: &[(&str, Type<Rc<String>>)],
    type_body: Type<Rc<String>>,
    body: Expr<Rc<String>>,
) -> Expr<Rc<String>> {
    Expr::from_content(ExprContent::MakeExists {
        params: Rc::new(
            params
                .iter()
                .cloned()
                .map(|(name, ty)| (Rc::new(name.to_owned()), ty))
                .collect(),
        ),
        type_body,
        body,
    })
}
