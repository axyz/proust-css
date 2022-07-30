use nom::{
    combinator::{map, opt},
    IResult,
};

use crate::nom_parser::{Token, Token::*, *};

pub fn block(input: &str) -> IResult<&str, Token> {
    map(delimited(char('{'), opt(rule_set), char('}')), |tokens| {
        Block {
            nodes: if let Some(nodes) = tokens {
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
    fn parse_rule_set() {
        assert_valid!(
            rule_set,
            "a:b",
            vec![Declaration {
                prop: "a",
                value: "b",
                important: false
            }]
        );

        assert_valid!(
            rule_set,
            "a: b",
            vec![Declaration {
                prop: "a",
                value: "b",
                important: false
            }]
        );

        assert_valid!(
            rule_set,
            r#"
a: b;
  c: d
        "#,
            vec![
                Declaration {
                    prop: "a",
                    value: "b",
                    important: false
                },
                Declaration {
                    prop: "c",
                    value: "d",
                    important: false
                },
            ]
        );

        assert_valid!(
            rule_set,
            "a: b; { b: c }   @media{e:f  }",
            vec![
                Declaration {
                    prop: "a",
                    value: "b",
                    important: false
                },
                Rule {
                    selector: "",
                    nodes: vec![Declaration {
                        prop: "b",
                        value: "c",
                        important: false
                    }]
                },
                AtRule {
                    name: "media",
                    params: "",
                    nodes: vec![Declaration {
                        prop: "e",
                        value: "f",
                        important: false
                    }]
                }
            ]
        );
    }

    #[test]
    fn parse_block() {
        assert_valid!(block, "{}", Block { nodes: vec![] });

        assert_valid!(
            block,
            "{ min-width: 1024px; foo: bar }",
            Block {
                nodes: vec![
                    Declaration {
                        prop: "min-width",
                        value: "1024px",
                        important: false
                    },
                    Declaration {
                        prop: "foo",
                        value: "bar",
                        important: false
                    },
                ]
            }
        );

        assert_valid!(
            block,
            r#"{
  a: b;
  c: d;
}"#,
            Block {
                nodes: vec![
                    Declaration {
                        prop: "a",
                        value: "b",
                        important: false
                    },
                    Declaration {
                        prop: "c",
                        value: "d",
                        important: false
                    },
                ]
            }
        );
    }
}
