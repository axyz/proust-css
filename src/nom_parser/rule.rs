use nom::{combinator::map, sequence::tuple, IResult};

use crate::nom_parser::{Token, Token::*, *};

pub fn rule(input: &str) -> IResult<&str, Token> {
    map(tuple((selector, ws_star, block)), |(selector, _, rest)| {
        Token::Rule {
            selector: if let Selector(value) = selector {
                value
            } else {
                ""
            },
            nodes: if let Block { nodes } = rest {
                nodes
            } else {
                vec![]
            },
        }
    })(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn parse_rule() {
        assert_valid!(
            rule,
            ".foo {}",
            Rule {
                selector: ".foo",
                nodes: vec![]
            }
        );

        assert_valid!(
            rule,
            ".foo { a: b }",
            Rule {
                selector: ".foo",
                nodes: vec![Declaration {
                    prop: "a",
                    value: "b",
                    important: false
                }]
            }
        );

        assert_valid!(
            rule,
            ".foo { a: b; c: \"d\" }",
            Rule {
                selector: ".foo",
                nodes: vec![
                    Declaration {
                        prop: "a",
                        value: "b",
                        important: false
                    },
                    Declaration {
                        prop: "c",
                        value: "\"d\"",
                        important: false
                    },
                ]
            }
        );
    }
}
