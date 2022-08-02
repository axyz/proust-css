use core::str::Chars;
use std::iter::{Enumerate, Peekable};

const NEWLINE: char = '\n';
const SPACE: char = ' ';
const TAB: char = '\t';
const CR: char = '\r';
const FEED: char = '\u{c}'; // \f
const SLASH: char = '/';
const ASTERISK: char = '*';
const OPEN_CURLY: char = '{';
const CLOSED_CURLY: char = '}';
const SEMICOLON: char = ';';
const COLON: char = ':';
const AT: char = '@';

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    Space,
    Comment,
    Word,
    OpenCurly,
    ClosedCurly,
    Colon,
    Semicolon,
    At,
}

#[derive(Debug, PartialEq)]
pub struct Token(pub TokenKind, pub usize, pub usize);

pub struct Tokenizer<'a> {
    iter: Peekable<Enumerate<Chars<'a>>>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Tokenizer<'a> {
        Tokenizer {
            iter: input.chars().enumerate().peekable(),
        }
    }

    pub fn peek_n(&mut self, n: usize) -> Option<(usize, char)> {
        let mut iter = self.iter.clone();
        iter.nth(n)
    }
}

macro_rules! match_white_space {
    () => {
        Some((_, SPACE)) | Some((_, TAB)) | Some((_, NEWLINE)) | Some((_, CR)) | Some((_, FEED))
    };
}

macro_rules! match_word_ending {
    () => {
        Some((_, OPEN_CURLY))
            | Some((_, CLOSED_CURLY))
            | Some((_, COLON))
            | Some((_, SEMICOLON))
            | Some((_, AT))
    };
}

macro_rules! consume {
    ($input:ident, $start:ident) => {
        if let Some((pos, _)) = $input.next() {
            $start = pos
        };
    };
    ($input:ident, $expr:expr) => {
        if let Some((_, _)) = $input.next() {
            $expr
        };
    };
}

macro_rules! T {
    ($kind:ident, $start:expr, $end:expr) => {
        Some(Token(TokenKind::$kind, $start, $end))
    };
}

macro_rules! match_token {
    ($expr:ident) => {
        Some((_, $expr))
    };
}

impl Iterator for Tokenizer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        let input = self.iter.by_ref();
        let mut start = 0;
        let mut offset = 0;
        match input.peek() {
            match_white_space!() => {
                consume!(input, start);
                while let match_white_space!() = input.peek() {
                    consume!(input, offset += 1);
                }
                T!(Space, start, start + offset)
            }
            match_token!(SLASH) => {
                consume!(input, start);
                if let match_token!(ASTERISK) = input.peek() {
                    consume!(input, offset += 1);
                    loop {
                        match input.peek() {
                            match_token!(ASTERISK) => {
                                if let match_token!(SLASH) = input.clone().nth(1) {
                                    consume!(input, offset += 1);
                                    consume!(input, offset += 1);
                                    break;
                                } else {
                                    consume!(input, offset += 1);
                                }
                            }
                            None => break,
                            _ => {
                                consume!(input, offset += 1);
                            }
                        }
                    }
                    T!(Comment, start, start + offset)
                } else {
                    loop {
                        match input.peek() {
                            match_word_ending!() | match_white_space!() => break,
                            None => break,
                            _ => {
                                consume!(input, offset += 1);
                            }
                        }
                    }
                    T!(Word, start, start + offset)
                }
            }
            match_token!(OPEN_CURLY) => {
                consume!(input, start);
                T!(OpenCurly, start, start + offset)
            }
            match_token!(CLOSED_CURLY) => {
                consume!(input, start);
                T!(ClosedCurly, start, start + offset)
            }
            match_token!(COLON) => {
                consume!(input, start);
                T!(Colon, start, start + offset)
            }
            match_token!(SEMICOLON) => {
                consume!(input, start);
                T!(Semicolon, start, start + offset)
            }
            match_token!(AT) => {
                consume!(input, start);
                T!(At, start, start + offset)
            }
            None => None,
            _ => {
                consume!(input, start);
                loop {
                    match input.peek() {
                        match_word_ending!() | match_white_space!() => break,
                        match_token!(SLASH) => {
                            if let match_token!(ASTERISK) = input.clone().nth(1) {
                                break;
                            } else {
                                consume!(input, offset += 1);
                            }
                        }
                        None => break,
                        _ => {
                            consume!(input, offset += 1);
                        }
                    }
                }
                T!(Word, start, start + offset)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TokenKind::*, *};

    #[test]
    fn tokenize() {
        let t = Tokenizer::new("/* foo */    \nabc { foo: bar; }foo baz/*foo*/");
        let res = t.collect::<Vec<Token>>();
        assert_eq!(
            res,
            vec![
                Token(Comment, 0, 8),
                Token(Space, 9, 13),
                Token(Word, 14, 16),
                Token(Space, 17, 17),
                Token(OpenCurly, 18, 18),
                Token(Space, 19, 19),
                Token(Word, 20, 22),
                Token(Colon, 23, 23),
                Token(Space, 24, 24),
                Token(Word, 25, 27),
                Token(Semicolon, 28, 28),
                Token(Space, 29, 29),
                Token(ClosedCurly, 30, 30),
                Token(Word, 31, 33),
                Token(Space, 34, 34),
                Token(Word, 35, 37),
                Token(Comment, 38, 44),
            ]
        );
    }
}
