use core::str::Chars;
use std::iter::Peekable;

const NEWLINE: char = '\n';
const SPACE: char = ' ';
const TAB: char = '\t';
const CR: char = '\r';
const FEED: char = '\u{c}'; // \f
const OPEN_CURLY: char = '{';
const CLOSED_CURLY: char = '}';
const SEMICOLON: char = ';';
const COLON: char = ':';
const AT: char = '@';

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    Space,
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
    iter: Peekable<Chars<'a>>,
    position: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Tokenizer<'a> {
        Tokenizer {
            iter: input.chars().peekable(),
            position: 0,
        }
    }
}

macro_rules! match_white_space {
    () => {
        Some(SPACE) | Some(TAB) | Some(NEWLINE) | Some(CR) | Some(FEED)
    };
}

macro_rules! match_white_space_peek {
    () => {
        Some(&SPACE) | Some(&TAB) | Some(&NEWLINE) | Some(&CR) | Some(&FEED)
    };
}

macro_rules! match_word_ending_peek {
    () => {
        Some(&OPEN_CURLY) | Some(&CLOSED_CURLY) | Some(&COLON) | Some(&SEMICOLON) | Some(&AT)
    };
}

impl Iterator for Tokenizer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        let input = self.iter.by_ref();
        let start = self.position;
        let next = match input.next() {
            match_white_space!() => {
                while let match_white_space_peek!() = input.peek() {
                    input.next();
                    self.position += 1;
                }
                Some(Token(TokenKind::Space, start, self.position))
            }
            Some(OPEN_CURLY) => Some(Token(TokenKind::OpenCurly, start, self.position)),
            Some(CLOSED_CURLY) => Some(Token(TokenKind::ClosedCurly, start, self.position)),
            Some(COLON) => Some(Token(TokenKind::Colon, start, self.position)),
            Some(SEMICOLON) => Some(Token(TokenKind::Semicolon, start, self.position)),
            Some(AT) => Some(Token(TokenKind::At, start, self.position)),
            None => None,
            _ => {
                loop {
                    match input.peek() {
                        match_word_ending_peek!() | match_white_space_peek!() => break,
                        None => break,
                        _ => {
                            self.position += 1;
                            input.next();
                        }
                    }
                }
                Some(Token(TokenKind::Word, start, self.position))
            }
        };

        self.position += 1;
        next
    }
}

#[cfg(test)]
mod tests {
    use super::{TokenKind::*, *};

    #[test]
    fn tokenize() {
        let t = Tokenizer::new("    \nabc { foo: bar; }foo baz");
        let res = t.collect::<Vec<Token>>();
        assert_eq!(
            res,
            vec![
                Token(Space, 0, 4),
                Token(Word, 5, 7),
                Token(Space, 8, 8),
                Token(OpenCurly, 9, 9),
                Token(Space, 10, 10),
                Token(Word, 11, 13),
                Token(Colon, 14, 14),
                Token(Space, 15, 15),
                Token(Word, 16, 18),
                Token(Semicolon, 19, 19),
                Token(Space, 20, 20),
                Token(ClosedCurly, 21, 21),
                Token(Word, 22, 24),
                Token(Space, 25, 25),
                Token(Word, 26, 28),
            ]
        );
    }
}
