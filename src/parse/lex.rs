use std::iter::Peekable;
use std::str::CharIndices;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Char(usize, char),
    End,
    Empty,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Name(String),
    UInt(u64),

    KeyMove,
    KeyFunc,
    KeyLet,
    KeyLetExists,
    KeyIn,
    KeyMakeExists,
    KeyOf,
    KeyCast,
    KeyBy,
    KeyReflEquiv,

    KeyForall,
    KeyExists,
    KeyEquiv,
    KeySize,
    KeyStatic,

    NumSign,
    Comma,
    Semicolon,
    Equals,
    Colon,
    Star,
    Arrow,

    OpenPar,
    ClosePar,

    OpenCurly,
    CloseCurly,
}

pub struct Lexer<Chars: Iterator> {
    chars: Peekable<Chars>,
}

impl<Chars: Iterator<Item = (usize, char)>> Lexer<Chars> {
    pub fn new(chars: Chars) -> Self {
        Lexer {
            chars: chars.peekable(),
        }
    }
}

impl<'a> Lexer<CharIndices<'a>> {
    pub fn from_str(s: &'a str) -> Self {
        Lexer::new(s.char_indices())
    }
}

lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, Token> = {
        let mut keywords = HashMap::new();

        keywords.insert("move", Token::KeyMove);
        keywords.insert("func", Token::KeyFunc);
        keywords.insert("let", Token::KeyLet);
        keywords.insert("let_exists", Token::KeyLetExists);
        keywords.insert("in", Token::KeyIn);
        keywords.insert("make_exists", Token::KeyMakeExists);
        keywords.insert("of", Token::KeyOf);
        keywords.insert("cast", Token::KeyCast);
        keywords.insert("by", Token::KeyBy);
        keywords.insert("refl_equiv", Token::KeyReflEquiv);

        keywords.insert("forall", Token::KeyForall);
        keywords.insert("exists", Token::KeyExists);
        keywords.insert("equiv", Token::KeyEquiv);
        keywords.insert("size", Token::KeySize);
        keywords.insert("static", Token::KeyStatic);

        keywords
    };
}

pub fn is_keyword(s: &str) -> bool {
    KEYWORDS.contains_key(s)
}

pub fn valid_name(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        None => {
            return false;
        }
        Some(first_char) => match first_char {
            'a'...'z' | 'A'...'Z' | '_' => while let Some(c) = chars.next() {
                match c {
                    'a'...'z' | 'A'...'Z' | '_' | '0'...'9' => {}
                    _ => {
                        return false;
                    }
                }
            },
            _ => {
                return false;
            }
        },
    }

    // Passed basic syntactic test
    // Now all that remains is to see if it's a reserved word
    !is_keyword(s)
}

pub fn quote_name(s: &str) -> String {
    let mut result = String::new();
    result.push('`');
    for c in s.chars() {
        if c == '\\' || c == '`' {
            result.push('\\');
        }
        result.push(c);
    }
    result.push('`');
    result
}

impl<Chars: Iterator<Item = (usize, char)>> Iterator for Lexer<Chars> {
    type Item = Result<(usize, Token, usize), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        'outer: while let Some((loc, next_char)) = self.chars.next() {
            match next_char {
                ' ' | '\t' | '\n' | '\r' | '\x0B' | '\x0C' => {}

                'a'...'z' | 'A'...'Z' | '_' => {
                    let mut name = String::new();
                    name.push(next_char);
                    let mut final_loc = loc;
                    while let Some(&(new_loc, word_char)) = self.chars.peek() {
                        final_loc = new_loc;
                        match word_char {
                            'a'...'z' | 'A'...'Z' | '_' | '0'...'9' => {
                                self.chars.next(); // consume peeked character
                                name.push(word_char)
                            }

                            _ => {
                                break;
                            }
                        }
                    }

                    let keyword = KEYWORDS.get(&name as &str);

                    if let Some(keyword) = keyword.cloned() {
                        return Some(Ok((loc, keyword, final_loc)));
                    }

                    return Some(Ok((loc, Token::Name(name), final_loc)));
                }

                '`' => {
                    let mut name = String::new();
                    while let Some((new_loc, word_char)) = self.chars.next() {
                        if word_char == '`' {
                            return Some(Ok((loc, Token::Name(name), new_loc)));
                        } else if word_char == '\\' {
                            if let Some((_, escaped_char)) = self.chars.next() {
                                name.push(escaped_char);
                            } else {
                                return Some(Err(Error::End));
                            }
                        } else {
                            name.push(word_char);
                        }
                    }
                    return Some(Err(Error::End));
                }

                '0'...'9' => {
                    let mut result: u64 = next_char
                        .to_digit(10)
                        .expect("Digit was already checked to lie in range 0...9")
                        as u64;
                    let mut final_loc = loc + 1;
                    while let Some(&(new_loc, digit @ '0'...'9')) = self.chars.peek() {
                        // This character is known to be part of the uint, so the end of the range
                        // is at least one byte after it.
                        final_loc = new_loc + 1;
                        if let Some(new_result) = result.checked_mul(10).and_then(|r| {
                            r.checked_add(digit
                                .to_digit(10)
                                .expect("Digit was already checked to lie in range 0...9")
                                as u64)
                        }) {
                            self.chars.next(); // consume peeked character
                            result = new_result;
                        } else {
                            return Some(Err(Error::Char(new_loc, digit)));
                        }
                    }
                    return Some(Ok((loc, Token::UInt(result), final_loc)));
                }

                _ => {
                    let token = match next_char {
                        '#' => Token::NumSign,
                        ',' => Token::Comma,
                        ';' => Token::Semicolon,
                        '=' => Token::Equals,
                        ':' => Token::Colon,
                        '*' => Token::Star,
                        '-' => {
                            match self.chars.next() {
                                Some((_, '>')) => Token::Arrow,
                                Some((_, '-')) => {
                                    // comment
                                    while let Some((_, comment_char)) = self.chars.next() {
                                        if comment_char == '\n' {
                                            continue 'outer;
                                        }
                                    }
                                    return None;
                                }
                                _ => return Some(Err(Error::Char(loc, '-'))),
                            }
                        }

                        '(' => Token::OpenPar,
                        ')' => Token::ClosePar,

                        '{' => Token::OpenCurly,
                        '}' => Token::CloseCurly,

                        _ => return Some(Err(Error::Char(loc, next_char))),
                    };

                    return Some(Ok((loc, token, loc + 1)));
                }
            }
        }
        None
    }
}
