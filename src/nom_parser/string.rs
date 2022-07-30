use nom::{
    branch::alt,
    bytes::complete::take,
    character::complete::char,
    combinator::recognize,
    combinator::{map, not, peek},
    multi::many0,
    sequence::{delimited, preceded},
    IResult,
};

use crate::nom_parser::{Token, Token::*, *};

pub fn string(input: &str) -> IResult<&str, Token> {
    map(
        alt((
            delimited(
                char('"'),
                recognize(many0(alt((
                    recognize(preceded(
                        peek(not(alt((
                            recognize(char('"')),
                            recognize(char('\\')),
                            newline,
                        )))),
                        take(1usize),
                    )),
                    escape,
                    preceded(char('\\'), newline),
                )))),
                char('"'),
            ),
            delimited(
                char('\''),
                recognize(many0(alt((
                    recognize(preceded(
                        peek(not(alt((
                            recognize(char('\'')),
                            recognize(char('\\')),
                            newline,
                        )))),
                        take(1usize),
                    )),
                    escape,
                    preceded(char('\\'), newline),
                )))),
                char('\''),
            ),
        )),
        String,
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn parse_string_token() {
        assert_valid!(string, "''", String(""));
        assert_valid!(string, "\"\"", String(""));
        assert_valid!(string, "'hello'", String("hello"));
        assert_valid!(string, "\"hello\"", String("hello"));
        assert_valid!(string, "\"he\\xllo\"", String("he\\xllo"));
        assert_valid!(
            string,
            r#""hello\
        world\x\
        ""#,
            String(
                r#"hello\
        world\x\
        "#
            )
        );
        assert_error!(
            string,
            r#""hello
        world\x
        ""#
        );
    }
}
