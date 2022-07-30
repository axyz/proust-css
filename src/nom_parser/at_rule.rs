use nom::{
    combinator::recognize,
    combinator::{map, opt},
    sequence::tuple,
    IResult,
};

use crate::nom_parser::{Token, Token::*, *};

fn at_keyword_token(input: &str) -> IResult<&str, &str> {
    preceded(char('@'), recognize(ident))(input)
}

pub fn at_rule(input: &str) -> IResult<&str, Token> {
    map(
        tuple((
            at_keyword_token,
            ws_star,
            recognize(selector),
            ws_star,
            opt(block),
        )),
        |(name, _, params, _, rest)| Token::AtRule {
            name,
            params,
            nodes: if let Some(Block { nodes }) = rest {
                nodes
            } else {
                vec![]
            },
        },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn parse_at_keyword_token() {
        assert_valid!(at_keyword_token, "@--foo", "--foo");
        assert_valid!(at_keyword_token, "@foo bar", " bar", "foo");
    }

    #[test]
    fn parse_at_rule() {
        assert_valid!(
            at_rule,
            "@media",
            AtRule {
                name: "media",
                params: "",
                nodes: vec![]
            }
        );

        assert_valid!(
            at_rule,
            "@media {}",
            AtRule {
                name: "media",
                params: "",
                nodes: vec![]
            }
        );

        assert_valid!(
            at_rule,
            "@media (min-width: 1024px) { foo: 'bar' }",
            AtRule {
                name: "media",
                params: "(min-width: 1024px)",
                nodes: vec![Declaration {
                    prop: "foo",
                    value: "'bar'",
                    important: false
                }]
            }
        );
    }
}
