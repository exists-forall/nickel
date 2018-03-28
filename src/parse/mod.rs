pub mod grammar;

use lalrpop_util::ParseError;

pub fn name(s: &str) -> Result<String, ParseError<usize, grammar::Token, &'static str>> {
    grammar::NameParser::new().parse(s)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn unquoted_name() {
        assert_eq!(name("hello"), Ok("hello".to_owned()));
        assert_eq!(name("HeLlO_wOrLd"), Ok("HeLlO_wOrLd".to_owned()));
        assert_eq!(name("_foo_bar_42_baz0"), Ok("_foo_bar_42_baz0".to_owned()));

        assert!(name("42").is_err());
        assert!(name("-hello").is_err());
        assert!(name("hello world").is_err());
    }

    #[test]
    fn quoted_name() {
        assert_eq!(name("`hello`"), Ok("hello".to_owned()));
        assert_eq!(name("`hello world`"), Ok("hello world".to_owned()));
        assert_eq!(name("`hello\\\\world`"), Ok("hello\\world".to_owned()));
        assert_eq!(name("`hello\\`world`"), Ok("hello`world".to_owned()));

        assert!(name("` ` `").is_err());
    }
}
