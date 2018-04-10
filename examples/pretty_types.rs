extern crate nickel_lang;
extern crate pretty_trait;

use std::rc::Rc;

use pretty_trait::println_simple;

use nickel_lang::types::*;
use nickel_lang::test_utils::types::*;
use nickel_lang::pretty_syntax::names::Names;
use nickel_lang::pretty_syntax::types::{Place, to_pretty};

fn print_type(names: &mut Names, ty: Type<Rc<String>>) {
    println_simple(&to_pretty(names, Place::Root, ty));
}

fn rc_str(s: &str) -> Rc<String> {
    Rc::new(s.to_owned())
}

fn main() {
    let mut names = Names::new();

    names.add_name(rc_str("foo"));
    let foo = var(2, 0);

    names.add_name(rc_str("bar"));
    let bar = var(2, 1);

    println!("Simple variables:");
    print_type(&mut names, foo.clone());
    print_type(&mut names, bar.clone());

    println!();
    println!("Simple pairs:");
    print_type(&mut names, pair(foo.clone(), bar.clone()));
    print_type(
        &mut names,
        pair(foo.clone(), pair(bar.clone(), foo.clone())),
    );
    print_type(
        &mut names,
        pair(pair(foo.clone(), bar.clone()), foo.clone()),
    );

    println!();
    println!("Simple functions:");
    print_type(&mut names, func(foo.clone(), bar.clone()));
    print_type(
        &mut names,
        func(pair(foo.clone(), bar.clone()), bar.clone()),
    );
    print_type(
        &mut names,
        func(foo.clone(), pair(bar.clone(), foo.clone())),
    );

    println!();
    println!("Existentials:");
    print_type(&mut names, exists_named("x", Kind::Type, var(3, 2)));
    print_type(&mut names, exists_named("foo", Kind::Type, var(3, 2)));
    print_type(
        &mut names,
        exists_named(
            "x",
            Kind::Type,
            exists_named("y", Kind::Type, pair(var(4, 2), var(4, 3))),
        ),
    );

    println!();
    println!("Universals:");
    print_type(
        &mut names,
        func_forall_named(
            &[("x", Kind::Type), ("y", Kind::Type)],
            var(4, 2),
            var(4, 3),
        ),
    );
}
