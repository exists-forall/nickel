use parse::syntax;

pub fn mk_ident(s: &str) -> syntax::Ident {
    syntax::Ident {
        name: s.to_owned(),
        collision_id: 0,
    }
}

pub fn mk_ident_collision(s: &str, collision_id: u64) -> syntax::Ident {
    syntax::Ident {
        name: s.to_owned(),
        collision_id,
    }
}
