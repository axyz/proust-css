use nom::{
    branch::alt,
    combinator::{eof, map, opt},
    multi::many0,
    sequence::{delimited, pair, tuple},
    IResult,
};

use crate::nom_parser::{Token, Token::*, *};

pub fn root(input: &str) -> IResult<&str, Token> {
    map(
        delimited(
            ws_star,
            opt(alt((
                tuple((comment, many0(rule_set))),
                tuple((at_rule, many0(rule_set))),
                tuple((rule, many0(rule_set))),
            ))),
            pair(ws_star, eof),
        ),
        |tokens| {
            if let Some((r, rs)) = tokens {
                let nodes = rs
                    .into_iter()
                    .fold(vec![r], |prev, next| [prev, next].concat());

                Root { nodes }
            } else {
                Root { nodes: vec![] }
            }
        },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn parse_root() {
        assert_valid!(root, "", Root { nodes: vec![] });
        assert_valid!(
            root,
            ".foo { bar: baz !important }",
            Root {
                nodes: vec![Rule {
                    selector: ".foo",
                    nodes: vec![Declaration {
                        prop: "bar",
                        value: "baz",
                        important: true
                    }]
                }]
            }
        );

        assert_valid!(
            root,
            r#"
/* hello */
.foo { bar: baz !important }

@media (min-width: 1024px) {
  /* hello */

  .foo {
    color: red
  }

  & + #qux {
    a: b;
    /* hello */
    c: d; /* hello */
  }
}
"#,
            Root {
                nodes: vec![
                    Comment { text: " hello " },
                    Rule {
                        selector: ".foo",
                        nodes: vec![Declaration {
                            prop: "bar",
                            value: "baz",
                            important: true
                        }]
                    },
                    AtRule {
                        name: "media",
                        params: "(min-width: 1024px)",
                        nodes: vec![
                            Comment { text: " hello " },
                            Rule {
                                selector: ".foo",
                                nodes: vec![Declaration {
                                    prop: "color",
                                    value: "red",
                                    important: false
                                }]
                            },
                            Rule {
                                selector: "& + #qux",
                                nodes: vec![
                                    Declaration {
                                        prop: "a",
                                        value: "b",
                                        important: false
                                    },
                                    Comment { text: " hello " },
                                    Declaration {
                                        prop: "c",
                                        value: "d",
                                        important: false
                                    },
                                    Comment { text: " hello " },
                                ]
                            }
                        ]
                    }
                ]
            }
        );
    }
}
