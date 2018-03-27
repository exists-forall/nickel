use std::str;
use std::borrow::ToOwned;

use nom::{is_hex_digit, be_u8, digit};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Ident {
    pub name: Vec<u8>,
    pub collision_id: usize,
}

fn is_unquoted_start_char(c: u8) -> bool {
    match c {
        b'a'...b'z' => true,
        b'A'...b'Z' => true,
        b'_' => true,
        _ => false,
    }
}

fn is_unquoted_char(c: u8) -> bool {
    match c {
        b'a'...b'z' => true,
        b'A'...b'Z' => true,
        b'_' => true,
        b'0'...b'9' => true,
        _ => false,
    }
}

named!(
    unquoted_name < &[u8], Vec<u8> >,
    map!(
        recognize!(preceded!(
            take_while_m_n!(1, 1, is_unquoted_start_char),
            take_while!(is_unquoted_char)
        )),
        ToOwned::to_owned
    )
);

fn hex_to_byte(hex: &[u8]) -> u8 {
    debug_assert_eq!(hex.len(), 2);
    u8::from_str_radix(
        str::from_utf8(hex).expect(
            "Parser should have rejected hex code if it was not valid UTF8",
        ),
        16,
    ).expect("Parser should have rejected hex code if it is not valid")
}

named!(
    hex_escape < &[u8], u8 >,
    map!(take_while_m_n!(2, 2, is_hex_digit), hex_to_byte)
);

fn is_unescaped_quoted_char(c: u8) -> bool {
    match c {
        b'\\' => false,
        b'`' => false,
        _ => true,
    }
}

named!(
    quoted_name < &[u8], Vec<u8> >,
    delimited!(char!('`'), many0!(alt!(
        verify!(be_u8, is_unescaped_quoted_char) |
        preceded!(char!('\\'), hex_escape)
    )), char!('`'))
);

named!(
    parse_name < &[u8], Vec<u8> >,
    alt!(
        unquoted_name |
        quoted_name
    )
);

fn parse_digits(digits: &[u8]) -> Option<usize> {
    usize::from_str_radix(str::from_utf8(digits).ok()?, 10).ok()
}

named!(
    parse_collision_id < &[u8], usize >,
    do_parse!(
        opt_id: opt!(map_opt!(preceded!(char!('#'), digit), parse_digits)) >>
        (opt_id.unwrap_or(0))
    )
);

named!(
    pub ident < &[u8], Ident >,
    do_parse!(
        name: parse_name >>
        collision_id: parse_collision_id >>
        (Ident { name, collision_id })
    )
);
