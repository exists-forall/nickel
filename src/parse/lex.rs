use std::iter::Peekable;
use std::str::CharIndices;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Char(usize, char),
    End,
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

    KeyForall,
    KeyExists,
    KeyVersion,
    KeyPlace,

    NumSign,
    Comma,
    Semicolon,
    Equals,
    Colon,

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
        Lexer { chars: chars.peekable() }
    }
}

impl<'a> Lexer<CharIndices<'a>> {
    pub fn from_str(s: &'a str) -> Self {
        Lexer::new(s.char_indices())
    }
}

impl<Chars: Iterator<Item = (usize, char)>> Iterator for Lexer<Chars> {
    type Item = Result<(usize, Token, usize), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((loc, next_char)) = self.chars.next() {
            match next_char {
                ' ' | '\t' | '\n' | '\r' | '\x0B' | '\x0C' => {}

                '/' => {
                    if let Some((_, '/')) = self.chars.next() {
                        // Consume comment
                        while let Some((_, comment_char)) = self.chars.next() {
                            if comment_char == '\n' {
                                break;
                            }
                        }
                    } else {
                        return Some(Err(Error::Char(loc + 1, '/')));
                    }
                }

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

                    let keyword = match &name as &str {
                        "move" => Some(Token::KeyMove),
                        "func" => Some(Token::KeyFunc),
                        "let" => Some(Token::KeyLet),
                        "let_exists" => Some(Token::KeyLetExists),
                        "in" => Some(Token::KeyIn),
                        "make_exists" => Some(Token::KeyMakeExists),
                        "of" => Some(Token::KeyOf),

                        "forall" => Some(Token::KeyForall),
                        "exists" => Some(Token::KeyExists),
                        "Version" => Some(Token::KeyVersion),
                        "Place" => Some(Token::KeyPlace),

                        _ => None,
                    };

                    if let Some(keyword) = keyword {
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
                    let mut result: u64 = next_char.to_digit(10).expect(
                        "Digit was already checked to lie in range 0...9",
                    ) as u64;
                    let mut final_loc = loc + 1;
                    while let Some(&(new_loc, digit @ '0'...'9')) = self.chars.peek() {
                        // This character is known to be part of the uint, so the end of the range
                        // is at least one byte after it.
                        final_loc = new_loc + 1;
                        if let Some(new_result) = result.checked_mul(10).and_then(|r| {
                            r.checked_add(digit.to_digit(10).expect(
                                "Digit was already checked to lie in range 0...9",
                            ) as u64)
                        })
                        {
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
