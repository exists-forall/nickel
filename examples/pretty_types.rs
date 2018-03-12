extern crate nickel;
extern crate pretty_trait;

use std::rc::Rc;

use pretty_trait::println_simple;

use nickel::types::*;
use nickel::pretty_syntax::names::Names;
use nickel::pretty_syntax::types::{Place, to_pretty};

fn print_type(names: &mut Names, ty: Type<Rc<String>>) {
    println_simple(&to_pretty(names, Place::Root, ty));
}

fn rc_str(s: &str) -> Rc<String> {
    Rc::new(s.to_owned())
}

fn main() {
    let mut names = Names::new();

    names.add_name(rc_str("foo"));
    let foo = Type::from_content(TypeContent::Var { free: 2, index: 0 });

    names.add_name(rc_str("bar"));
    let bar = Type::from_content(TypeContent::Var { free: 2, index: 1 });

    println!("Simple variables:");
    print_type(&mut names, foo.clone());
    print_type(&mut names, bar.clone());

    println!();
    println!("Simple pairs:");
    print_type(
        &mut names,
        Type::from_content(TypeContent::Pair {
            left: foo.clone(),
            right: bar.clone(),
        }),
    );
    print_type(
        &mut names,
        Type::from_content(TypeContent::Pair {
            left: foo.clone(),
            right: Type::from_content(TypeContent::Pair {
                left: bar.clone(),
                right: foo.clone(),
            }),
        }),
    );
    print_type(
        &mut names,
        Type::from_content(TypeContent::Pair {
            left: Type::from_content(TypeContent::Pair {
                left: foo.clone(),
                right: bar.clone(),
            }),
            right: foo.clone(),
        }),
    );

    println!();
    println!("Simple functions:");
    print_type(
        &mut names,
        Type::from_content(TypeContent::Func {
            params: Rc::new(Vec::new()),
            arg: foo.clone(),
            ret: bar.clone(),
        }),
    );
    print_type(
        &mut names,
        Type::from_content(TypeContent::Func {
            params: Rc::new(Vec::new()),
            arg: Type::from_content(TypeContent::Pair {
                left: foo.clone(),
                right: bar.clone(),
            }),
            ret: bar.clone(),
        }),
    );
    print_type(
        &mut names,
        Type::from_content(TypeContent::Func {
            params: Rc::new(Vec::new()),
            arg: foo.clone(),
            ret: Type::from_content(TypeContent::Pair {
                left: bar.clone(),
                right: foo.clone(),
            }),
        }),
    );

    println!();
    println!("Existentials:");
    print_type(
        &mut names,
        Type::from_content(TypeContent::Exists {
            param: TypeParam {
                name: rc_str("x"),
                kind: Kind::Type,
            },
            body: Type::from_content(TypeContent::Var { free: 3, index: 2 }),
        }),
    );
    print_type(
        &mut names,
        Type::from_content(TypeContent::Exists {
            param: TypeParam {
                name: rc_str("foo"),
                kind: Kind::Type,
            },
            body: Type::from_content(TypeContent::Var { free: 3, index: 2 }),
        }),
    );
    print_type(
        &mut names,
        Type::from_content(TypeContent::Exists {
            param: TypeParam {
                name: rc_str("x"),
                kind: Kind::Type,
            },
            body: Type::from_content(TypeContent::Exists {
                param: TypeParam {
                    name: rc_str("y"),
                    kind: Kind::Type,
                },
                body: Type::from_content(TypeContent::Pair {
                    left: Type::from_content(TypeContent::Var { free: 4, index: 2 }),
                    right: Type::from_content(TypeContent::Var { free: 4, index: 3 }),
                }),
            }),
        }),
    );

    println!();
    println!("Universals:");
    print_type(
        &mut names,
        Type::from_content(TypeContent::Func {
            params: Rc::new(vec![
                TypeParam {
                    name: rc_str("x"),
                    kind: Kind::Type,
                },
                TypeParam {
                    name: rc_str("y"),
                    kind: Kind::Type,
                },
            ]),
            arg: Type::from_content(TypeContent::Var { free: 4, index: 2 }),
            ret: Type::from_content(TypeContent::Var { free: 4, index: 3 }),
        }),
    );
}
