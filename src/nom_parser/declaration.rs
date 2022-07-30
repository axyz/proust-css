use nom::{
    character::complete::char,
    combinator::{map, opt},
    sequence::{pair, preceded, separated_pair},
    IResult,
};

use crate::nom_parser::{Token, Token::*, *};

fn important(input: &str) -> IResult<&str, bool> {
    value(
        true,
        preceded(char('!'), delimited(ws_star, tag("important"), ws_star)),
    )(input)
}

pub fn declaration(input: &str) -> IResult<&str, Token> {
    map(
        separated_pair(
            ident,
            preceded(ws_star, pair(char(':'), ws_star)),
            pair(selector, opt(preceded(ws_star, important))),
        ),
        |(token, (selector, important))| match token {
            Ident(prop) => Declaration {
                prop,
                value: if let Selector(value) = selector {
                    value
                } else {
                    ""
                },
                important: matches!(important, Some(true)),
            },
            _ => Declaration {
                prop: "",
                value: "",
                important: false,
            },
        },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn parse_important() {
        assert_valid!(important, "!important", true);
        assert_valid!(important, "! important ", true);
        assert_valid!(important, "! important ;", ";", true);
        assert_error!(important, "foo!");
    }

    #[test]
    fn parse_declaration() {
        assert_valid!(
            declaration,
            "width: 100px",
            Declaration {
                prop: "width",
                value: "100px",
                important: false
            }
        );

        assert_valid!(
            declaration,
            "width : 100px !important",
            Declaration {
                prop: "width",
                value: "100px",
                important: true
            }
        );
    }
}
