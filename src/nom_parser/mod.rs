extern crate nom;

use std::{
    fmt,
    fmt::{Display, Formatter},
};

use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    character::complete::{char, one_of, satisfy},
    combinator::recognize,
    combinator::{not, opt, peek, value},
    multi::{many0, many_m_n},
    sequence::{delimited, pair, preceded, tuple},
    IResult,
};

pub mod at_rule;
pub mod block;
pub mod comment;
pub mod declaration;
pub mod ident;
pub mod root;
pub mod rule;
pub mod rule_set;
pub mod selector;
pub mod string;
use at_rule::*;
use block::*;
use comment::*;
use declaration::*;
use ident::*;
use root::*;
use rule::*;
use rule_set::*;
use selector::*;
use string::*;

#[derive(Debug, PartialEq, Clone)]
pub enum Token<'a> {
    Comment {
        text: &'a str,
    },
    Root {
        nodes: Vec<Token<'a>>,
    },
    Declaration {
        prop: &'a str,
        value: &'a str,
        important: bool,
    },
    Rule {
        selector: &'a str,
        nodes: Vec<Token<'a>>,
    },
    AtRule {
        name: &'a str,
        params: &'a str,
        nodes: Vec<Token<'a>>,
    },
    Block {
        nodes: Vec<Token<'a>>,
    },
    String(&'a str),
    Selector(&'a str),
    Ident(&'a str),
}

pub fn parse_token(input: &str) -> IResult<&str, Token> {
    alt((
        comment,
        root,
        at_rule,
        rule,
        block,
        declaration,
        string,
        ident,
        selector,
    ))(input)
}

impl<'a> From<&'a str> for Token<'a> {
    fn from(input: &str) -> Token<'_> {
        parse_token(input).unwrap().1
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Comment { .. } => fmt_comment(self, f),
            _ => write!(f, "todo"),
        }
    }
}

// FIXME: \f does not seem to work
fn newline(input: &str) -> IResult<&str, &str> {
    alt((tag("\n"), tag("\r\n"), tag("\r")))(input)
}

fn whitespace(input: &str) -> IResult<&str, &str> {
    alt((tag(" "), tag("\t"), newline))(input)
}

fn hex_digit(input: &str) -> IResult<&str, &str> {
    recognize(one_of("0123456789abcdefABCDEF"))(input)
}

fn digit(input: &str) -> IResult<&str, &str> {
    recognize(one_of("0123456789"))(input)
}

fn escape(input: &str) -> IResult<&str, &str> {
    recognize(preceded(
        char('\\'),
        alt((
            recognize(preceded(peek(not(alt((newline, hex_digit)))), take(1usize))),
            recognize(preceded(many_m_n(1, 6, hex_digit), opt(whitespace))),
        )),
    ))(input)
}

fn ws_star(input: &str) -> IResult<&str, &str> {
    recognize(many0(whitespace))(input)
}

fn not_ascii(input: &str) -> IResult<&str, &str> {
    recognize(satisfy(|c| !c.is_ascii()))(input)
}

fn ascii_alphabetic(input: &str) -> IResult<&str, &str> {
    recognize(satisfy(|c| c.is_ascii_alphabetic()))(input)
}

// a-z A-Z _ or non ASCII
fn valid_first_ident_char(input: &str) -> IResult<&str, &str> {
    alt((ascii_alphabetic, recognize(char('_')), not_ascii))(input)
}

// a-z A-Z 0-9 _ - or non ASCII
fn valid_second_ident_char(input: &str) -> IResult<&str, &str> {
    alt((valid_first_ident_char, digit, recognize(char('-'))))(input)
}

#[macro_export]
macro_rules! assert_error {
    ($parser:expr, $input:expr) => {
        assert!(matches!($parser($input), Err(_)));
    };
}

#[macro_export]
macro_rules! assert_valid {
    ($parser:expr, $input:expr, $rest:expr, $consumed:expr) => {
        assert_eq!($parser($input), Ok(($rest, $consumed)));
    };

    ($parser:expr, $input:expr, $consumed:expr) => {
        assert_eq!($parser($input), Ok(("", $consumed)));
    };

    ($parser:expr, $input:expr) => {
        assert_eq!($parser($input), Ok(("", $input)));
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_not_ascii() {
        assert_eq!(
            Token::from("/* hello */"),
            Token::Comment { text: " hello " }
        );
        assert_error!(not_ascii, "_");
        assert_valid!(not_ascii, "ಠfoo", "foo", "ಠ");
        assert_error!(not_ascii, "}");
        assert_error!(not_ascii, "\t");
        assert_valid!(not_ascii, "ಠ");
    }

    #[test]
    fn parse_digit() {
        assert_valid!(digit, "0");
        assert_valid!(digit, "1foo", "foo", "1");
        assert_error!(digit, "a");
        assert_error!(digit, "{");
        assert_error!(digit, "ಠ");
    }

    #[test]
    fn parse_newline() {
        assert_valid!(newline, "\n");
        assert_valid!(newline, "\r");
        assert_valid!(newline, "\r\n");
        assert_valid!(newline, "\r\nfoo", "foo", "\r\n");
    }

    #[test]
    fn parse_whitespace() {
        assert_valid!(whitespace, " ");
        assert_valid!(whitespace, "\t");
        assert_valid!(whitespace, "\n");
        assert_valid!(whitespace, "\r\n");
        assert_valid!(whitespace, "\r\nfoo", "foo", "\r\n");
    }

    #[test]
    fn parse_hex_digit() {
        assert_valid!(hex_digit, "0");
        assert_valid!(hex_digit, "A");
        assert_valid!(hex_digit, "f");
        assert_valid!(hex_digit, "ffoo", "foo", "f");
    }

    #[test]
    fn parse_escape() {
        assert_valid!(escape, "\\x");
        assert_valid!(escape, "\\xfoo", "foo", "\\x");
        assert_valid!(escape, "\\aabbcc");
        assert_valid!(escape, "\\aabbcc ");
        assert_valid!(escape, "\\aabbcc foo", "foo", "\\aabbcc ");
    }

    #[test]
    fn parse_ws_star() {
        assert_valid!(ws_star, "");
        assert_valid!(ws_star, "   ");
        assert_valid!(ws_star, "   foo", "foo", "   ");
        assert_valid!(ws_star, "   \r\n");
    }

    #[test]
    fn parse_valid_first_ident_char() {
        assert_valid!(valid_first_ident_char, "a");
        assert_valid!(valid_first_ident_char, "_");
        assert_valid!(valid_first_ident_char, "ಠ");
        assert_error!(valid_first_ident_char, "3");
        assert_error!(valid_first_ident_char, "-");
    }

    #[test]
    fn parse_valid_second_ident_char() {
        assert_valid!(valid_second_ident_char, "a");
        assert_valid!(valid_second_ident_char, "_");
        assert_valid!(valid_second_ident_char, "_foo", "foo", "_");
        assert_valid!(valid_second_ident_char, "ಠ");
        assert_valid!(valid_second_ident_char, "3");
        assert_valid!(valid_second_ident_char, "-");
        assert_error!(valid_second_ident_char, "{");
        assert_error!(valid_second_ident_char, "/");
    }
}
