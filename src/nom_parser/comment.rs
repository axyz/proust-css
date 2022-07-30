use nom::{
    bytes::complete::{tag, take_until},
    combinator::map,
    sequence::delimited,
    IResult,
};
use std::fmt::{Formatter, Result};

use crate::nom_parser::{Token, Token::*};

pub fn comment(input: &str) -> IResult<&str, Token> {
    map(delimited(tag("/*"), take_until("*/"), tag("*/")), |text| {
        Comment { text }
    })(input)
}

pub fn fmt_comment(token: &Token, f: &mut Formatter) -> Result {
    match token {
        Comment { text } => write!(f, "/*{}*/", text),
        _ => write!(f, "todo"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn parse_comment() {
        assert_valid!(comment, "/* hello */", Comment { text: " hello " });
        assert_valid!(
            comment,
            "/* hello */foo",
            "foo",
            Comment { text: " hello " }
        );
    }

    #[test]
    fn comment_from_string() {
        assert_eq!(
            Token::from("/* hello */"),
            Token::Comment { text: " hello " }
        );
    }

    #[test]
    fn comment_to_string() {
        assert_eq!(Token::Comment { text: "hello" }.to_string(), "/*hello*/");
    }
}
