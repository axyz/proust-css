use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::recognize,
    combinator::{map, opt},
    multi::many0,
    sequence::preceded,
    IResult,
};

use crate::nom_parser::{Token, Token::*, *};

pub fn ident(input: &str) -> IResult<&str, Token> {
    map(
        recognize(preceded(
            alt((
                tag("--"),
                preceded(
                    opt(char('-')),
                    alt((recognize(valid_first_ident_char), recognize(escape))),
                ),
            )),
            many0(alt((recognize(valid_second_ident_char), recognize(escape)))),
        )),
        Ident,
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn parse_ident_token() {
        assert_valid!(ident, "--foo", Ident("--foo"));
        assert_valid!(ident, "--foo: bar", ": bar", Ident("--foo"));
        assert_valid!(ident, "foo", Ident("foo"));
        assert_valid!(ident, "-foo", Ident("-foo"));
        assert_valid!(ident, "-f\\xfoo-_\\x: bar", ": bar", Ident("-f\\xfoo-_\\x"));
    }
}
