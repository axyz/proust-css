use crate::parser::{error::ParseError, tokenizer::*};
use std::iter::Peekable;

pub mod error;
pub mod tokenizer;
pub mod visitor;

#[derive(Debug, PartialEq)]
pub struct Comment<'a> {
    pub text: &'a str,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, PartialEq)]
pub struct Declaration<'a> {
    pub prop: &'a str,
    pub value: &'a str,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, PartialEq)]
pub struct AtRule<'a> {
    pub name: &'a str,
    pub params: &'a str,
    pub nodes: Vec<BlockChild<'a>>,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, PartialEq)]
pub enum BlockChild<'a> {
    AtRule(AtRule<'a>),
    Declaration(Declaration<'a>),
    Comment(Comment<'a>),
}

#[derive(Debug, PartialEq)]
pub struct Rule<'a> {
    pub selector: &'a str,
    pub nodes: Vec<BlockChild<'a>>,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, PartialEq)]
pub enum RootChild<'a> {
    Rule(Rule<'a>),
    AtRule(AtRule<'a>),
    Comment(Comment<'a>),
}

#[derive(Debug, PartialEq)]
pub struct Root<'a> {
    pub nodes: Vec<RootChild<'a>>,
    pub start: usize,
    pub end: usize,
}

pub struct Parser<'a> {
    tokenizer: Peekable<Tokenizer<'a>>,
    source: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Parser {
            tokenizer: Tokenizer::new(input).peekable(),
            source: input,
            pos: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Root<'a>, ParseError> {
        use TokenKind::*;
        let mut nodes: Vec<RootChild<'a>> = vec![];
        let start = self.pos;

        while let Some(token) = self.tokenizer.peek() {
            match token {
                Token(Space, ..) => self.skip_while(|t| matches!(t, Some(Token(Space, ..)))),
                Token(At, ..) => nodes.push(RootChild::AtRule(self.parse_at_rule()?)),
                Token(Word, ..) => nodes.push(RootChild::Rule(self.parse_rule()?)),
                Token(Comment, ..) => nodes.push(RootChild::Comment(self.parse_comment()?)),
                _ => {
                    self.next_token();
                }
            }
        }

        let end = self.pos;
        Ok(Root { nodes, start, end })
    }

    fn parse_comment(&mut self) -> Result<Comment<'a>, ParseError> {
        let start = self.pos;

        let text = if let Some(token) = self.next_token() {
            let Token(_, start, end) = token;
            &self.source[start + 2..end - 1]
        } else {
            return Err(ParseError::Error);
        };

        let end = self.pos;

        Ok(Comment { text, start, end })
    }

    fn parse_declaration(&mut self) -> Result<Declaration<'a>, ParseError> {
        use TokenKind::*;
        let decl_start;
        let prop = if let Some(token) = self.next_token() {
            let Token(_, start, end) = token;
            decl_start = start;

            &self.source[start..end + 1]
        } else {
            return Err(ParseError::Error);
        };

        self.skip_while(|t| matches!(t, Some(Token(Space, ..)) | Some(Token(Comment, ..))));

        if let Some(Token(Colon, ..)) = self.next_token() {
            self.skip_while(|t| matches!(t, Some(Token(Space, ..)) | Some(Token(Comment, ..))));
        } else {
            return Err(ParseError::Error);
        }

        let value = if let Some(token) = self.next_token() {
            let Token(_, start, end) = token;

            &self.source[start..end + 1]
        } else {
            return Err(ParseError::Error);
        };

        let end = self.pos;
        Ok(Declaration {
            prop,
            value,
            start: decl_start,
            end,
        })
    }

    fn parse_rule(&mut self) -> Result<Rule<'a>, ParseError> {
        use TokenKind::*;

        let start = if let Some(Token(Word, start, _)) = self.next_token() {
            start
        } else {
            return Err(ParseError::Error);
        };

        self.skip_while(|t| !matches!(t, Some(Token(OpenCurly, ..))));

        let selector = self.source[start..self.pos + 1]
            .split("/*")
            .next()
            .expect("cannot filter comments on rule name")
            .trim();

        Ok(Rule {
            selector,
            nodes: self.parse_block()?,
            start,
            end: self.pos,
        })
    }

    fn parse_at_rule(&mut self) -> Result<AtRule<'a>, ParseError> {
        use TokenKind::*;
        self.next_token(); // skip @
        let start = self.pos;

        let name = if let Some(Token(Word, start, end)) = self.next_token() {
            &self.source[start..end + 1]
        } else {
            return Err(ParseError::Error);
        };

        self.skip_while(|t| matches!(t, Some(Token(Comment, ..)) | Some(Token(Space, ..))));

        let start_params = self.pos + 1;

        self.skip_while(|t| !matches!(t, Some(Token(OpenCurly, ..))));

        let params = self.source[start_params..self.pos + 1]
            .split("/*")
            .next()
            .expect("cannot filter comments on at_rule params")
            .trim();

        Ok(AtRule {
            name,
            params,
            nodes: self.parse_block()?,
            start,
            end: self.pos,
        })
    }

    fn parse_declartion_or_at_rule_list(&mut self) -> Result<Vec<BlockChild<'a>>, ParseError> {
        use TokenKind::*;
        let mut nodes: Vec<BlockChild<'a>> = vec![];

        self.skip_while(|t| matches!(t, Some(Token(Space, ..))));

        while let Some(token) = self.tokenizer.peek() {
            match token {
                Token(Word, ..) => {
                    nodes.push(BlockChild::Declaration(self.parse_declaration()?));
                    self.skip_while(|t| {
                        matches!(t, Some(Token(Space, ..)) | Some(Token(Comment, ..)))
                    });

                    match self.tokenizer.peek() {
                        Some(Token(Semicolon, ..)) => {
                            self.next_token();
                            nodes.extend(self.parse_declartion_or_at_rule_list()?)
                        }
                        Some(Token(ClosedCurly, ..)) => break,
                        _ => return Err(ParseError::Error),
                    }
                }
                Token(At, ..) => {
                    nodes.push(BlockChild::AtRule(self.parse_at_rule()?));
                    self.skip_while(|t| {
                        matches!(t, Some(Token(Space, ..)) | Some(Token(Comment, ..)))
                    });
                    nodes.extend(self.parse_declartion_or_at_rule_list()?);
                }
                Token(Comment, ..) => nodes.push(BlockChild::Comment(self.parse_comment()?)),
                _ => break,
            }
        }

        self.skip_while(|t| matches!(t, Some(Token(Space, ..))));

        Ok(nodes)
    }

    fn parse_block(&mut self) -> Result<Vec<BlockChild<'a>>, ParseError> {
        use TokenKind::*;

        if let Some(Token(OpenCurly, ..)) = self.next_token() {
        } else {
            return Err(ParseError::Error);
        }

        let nodes = self.parse_declartion_or_at_rule_list()?;

        if let Some(Token(ClosedCurly, ..)) = self.next_token() {
        } else {
            return Err(ParseError::Error);
        }

        Ok(nodes)
    }

    fn next_token(&mut self) -> Option<Token> {
        let token = self.tokenizer.next()?;
        self.pos = token.2;
        Some(token)
    }

    fn skip_while<F>(&mut self, condition: F)
    where
        F: Fn(Option<&Token>) -> bool,
    {
        while condition(self.tokenizer.peek()) {
            if self.next_token().is_none() {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_parse_ok {
        ($input: expr, $output: expr) => {
            assert_eq!(Parser::new($input).parse(), Ok($output));
        };
    }

    #[test]
    fn parse_empty() {
        assert_parse_ok!(
            "",
            Root {
                start: 0,
                end: 0,
                nodes: vec![]
            }
        );
    }

    #[test]
    fn parse_root_level_comment() {
        assert_parse_ok!(
            "/* hello */",
            Root {
                start: 0,
                end: 10,
                nodes: vec![RootChild::Comment(Comment {
                    start: 0,
                    end: 10,
                    text: " hello "
                })]
            }
        );
    }

    #[test]
    fn parse_empty_rule() {
        assert_parse_ok!(
            "foo {}",
            Root {
                start: 0,
                end: 5,
                nodes: vec![RootChild::Rule(Rule {
                    start: 0,
                    end: 5,
                    selector: "foo",
                    nodes: vec![]
                })]
            }
        );
    }

    #[test]
    fn parse_block_level_comment() {
        assert_parse_ok!(
            "foo {a:b; /* hello */ c:b}",
            Root {
                start: 0,
                end: 25,
                nodes: vec![RootChild::Rule(Rule {
                    start: 0,
                    end: 25,
                    selector: "foo",
                    nodes: vec![
                        BlockChild::Declaration(Declaration {
                            start: 5,
                            end: 7,
                            prop: "a",
                            value: "b"
                        }),
                        BlockChild::Comment(Comment {
                            start: 9,
                            end: 20,
                            text: " hello "
                        }),
                        BlockChild::Declaration(Declaration {
                            start: 22,
                            end: 24,
                            prop: "c",
                            value: "b"
                        })
                    ]
                })]
            }
        );
    }

    #[test]
    fn parse_rule_with_declaration() {
        assert_parse_ok!(
            "foo { a: b }",
            Root {
                start: 0,
                end: 11,
                nodes: vec![RootChild::Rule(Rule {
                    start: 0,
                    end: 11,
                    selector: "foo",
                    nodes: vec![BlockChild::Declaration(Declaration {
                        start: 6,
                        end: 9,
                        prop: "a",
                        value: "b"
                    })]
                })]
            }
        );
    }

    #[test]
    fn parse_empty_at_rule() {
        assert_parse_ok!(
            "@foo {}",
            Root {
                start: 0,
                end: 6,
                nodes: vec![RootChild::AtRule(AtRule {
                    start: 0,
                    end: 6,
                    name: "foo",
                    params: "",
                    nodes: vec![]
                })]
            }
        );
    }

    #[test]
    fn parse_empty_at_rule_with_params() {
        assert_parse_ok!(
            "@foo (bar) {}",
            Root {
                start: 0,
                end: 12,
                nodes: vec![RootChild::AtRule(AtRule {
                    start: 0,
                    end: 12,
                    name: "foo",
                    params: "(bar)",
                    nodes: vec![]
                })]
            }
        );
    }

    #[test]
    fn parse_at_rule_with_declaration() {
        assert_parse_ok!(
            "@foo { a: b }",
            Root {
                start: 0,
                end: 12,
                nodes: vec![RootChild::AtRule(AtRule {
                    start: 0,
                    end: 12,
                    name: "foo",
                    params: "",
                    nodes: vec![BlockChild::Declaration(Declaration {
                        start: 7,
                        end: 10,
                        prop: "a",
                        value: "b"
                    })]
                })]
            }
        );
    }

    #[test]
    fn parse_at_rule_with_declaration_and_params() {
        assert_parse_ok!(
            "@foo (bar) { a: b }",
            Root {
                start: 0,
                end: 18,
                nodes: vec![RootChild::AtRule(AtRule {
                    start: 0,
                    end: 18,
                    name: "foo",
                    params: "(bar)",
                    nodes: vec![BlockChild::Declaration(Declaration {
                        start: 13,
                        end: 16,
                        prop: "a",
                        value: "b"
                    })]
                })]
            }
        );
    }

    #[test]
    fn parse_mested_at_rule() {
        assert_parse_ok!(
            "foo { hello: world ; foo : bar; @foo { a:b } }",
            Root {
                start: 0,
                end: 45,
                nodes: vec![RootChild::Rule(Rule {
                    start: 0,
                    end: 45,
                    selector: "foo",
                    nodes: vec![
                        BlockChild::Declaration(Declaration {
                            prop: "hello",
                            value: "world",
                            start: 6,
                            end: 17
                        }),
                        BlockChild::Declaration(Declaration {
                            prop: "foo",
                            value: "bar",
                            start: 21,
                            end: 29
                        }),
                        BlockChild::AtRule(AtRule {
                            start: 32,
                            end: 43,
                            name: "foo",
                            params: "",
                            nodes: vec![BlockChild::Declaration(Declaration {
                                prop: "a",
                                value: "b",
                                start: 39,
                                end: 41
                            })]
                        })
                    ]
                })]
            }
        );
    }
}
