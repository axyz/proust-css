use crate::parser::{error::ParseError, tokenizer::*};
use std::iter::Peekable;

pub mod error;
pub mod tokenizer;
pub mod visitor;
#[macro_use]
mod macros;

#[derive(Debug, PartialEq)]
pub struct Comment {
    pub text: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, PartialEq)]
pub struct Declaration {
    pub prop: String,
    pub value: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, PartialEq)]
pub struct AtRule {
    pub name: String,
    pub params: String,
    pub nodes: Vec<BlockChild>,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, PartialEq)]
pub enum BlockChild {
    AtRule(AtRule),
    Declaration(Declaration),
    Comment(Comment),
}

#[derive(Debug, PartialEq)]
pub struct Rule {
    pub selector: String,
    pub nodes: Vec<BlockChild>,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, PartialEq)]
pub enum RootChild {
    Rule(Rule),
    AtRule(AtRule),
    Comment(Comment),
}

#[derive(Debug, PartialEq)]
pub struct Root {
    pub nodes: Vec<RootChild>,
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

    pub fn parse(&mut self) -> Result<Root, ParseError> {
        use TokenKind::*;
        let mut nodes: Vec<RootChild> = vec![];
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

    fn parse_comment(&mut self) -> Result<Comment, ParseError> {
        let start = self.pos;

        let text = if let Some(token) = self.next_token() {
            let Token(_, start, end) = token;
            &self.source[start + 2..end - 1]
        } else {
            return Err(ParseError::Error);
        }
        .to_string();

        let end = self.pos;

        Ok(Comment { text, start, end })
    }

    fn parse_declaration(&mut self) -> Result<Declaration, ParseError> {
        use TokenKind::*;
        let decl_start;
        let prop = if let Some(token) = self.next_token() {
            let Token(_, start, end) = token;
            decl_start = start;

            &self.source[start..end + 1]
        } else {
            return Err(ParseError::Error);
        }
        .to_string();

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
        }
        .to_string();

        let end = self.pos;
        Ok(Declaration {
            prop,
            value,
            start: decl_start,
            end,
        })
    }

    fn parse_rule(&mut self) -> Result<Rule, ParseError> {
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
            .trim()
            .to_string();

        Ok(Rule {
            selector,
            nodes: self.parse_block()?,
            start,
            end: self.pos,
        })
    }

    fn parse_at_rule(&mut self) -> Result<AtRule, ParseError> {
        use TokenKind::*;
        self.next_token(); // skip @
        let start = self.pos;

        let name = if let Some(Token(Word, start, end)) = self.next_token() {
            &self.source[start..end + 1]
        } else {
            return Err(ParseError::Error);
        }
        .to_string();

        self.skip_while(|t| matches!(t, Some(Token(Comment, ..)) | Some(Token(Space, ..))));

        let start_params = self.pos + 1;

        self.skip_while(|t| !matches!(t, Some(Token(OpenCurly, ..))));

        let params = self.source[start_params..self.pos + 1]
            .split("/*")
            .next()
            .expect("cannot filter comments on at_rule params")
            .trim()
            .to_string();

        Ok(AtRule {
            name,
            params,
            nodes: self.parse_block()?,
            start,
            end: self.pos,
        })
    }

    fn parse_declartion_or_at_rule_list(&mut self) -> Result<Vec<BlockChild>, ParseError> {
        use TokenKind::*;
        let mut nodes: Vec<BlockChild> = vec![];

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

    fn parse_block(&mut self) -> Result<Vec<BlockChild>, ParseError> {
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

    #[test]
    fn parse_empty() {
        assert_parse_ok!("", root!(0, 0, vec![]));
    }

    #[test]
    fn parse_root_level_comment() {
        assert_parse_ok!(
            "/* hello */",
            root!(0, 10, vec![root_comment!(0, 10, " hello ")])
        );
    }

    #[test]
    fn parse_empty_rule() {
        assert_parse_ok!("foo {}", root!(0, 5, vec![root_rule!(0, 5, "foo", vec![])]));
    }

    #[test]
    fn parse_block_level_comment() {
        assert_parse_ok!(
            "foo {a:b; /* hello */ c:b}",
            root!(
                0,
                25,
                vec![root_rule!(
                    0,
                    25,
                    "foo",
                    vec![
                        decl!(5, 7, "a", "b"),
                        comment!(9, 20, " hello "),
                        decl!(22, 24, "c", "b")
                    ]
                )]
            )
        );
    }

    #[test]
    fn parse_rule_with_declaration() {
        assert_parse_ok!(
            "foo { a: b }",
            root!(
                0,
                11,
                vec![root_rule!(0, 11, "foo", vec![decl!(6, 9, "a", "b")])]
            )
        );
    }

    #[test]
    fn parse_empty_at_rule() {
        assert_parse_ok!(
            "@foo {}",
            root!(0, 6, vec![root_at_rule!(0, 6, "foo", "", vec![])])
        );
    }

    #[test]
    fn parse_empty_at_rule_with_params() {
        assert_parse_ok!(
            "@foo (bar) {}",
            root!(0, 12, vec![root_at_rule!(0, 12, "foo", "(bar)", vec![])])
        );
    }

    #[test]
    fn parse_at_rule_with_declaration() {
        assert_parse_ok!(
            "@foo { a: b }",
            root!(
                0,
                12,
                vec![root_at_rule!(
                    0,
                    12,
                    "foo",
                    "",
                    vec![decl!(7, 10, "a", "b")]
                )]
            )
        );
    }

    #[test]
    fn parse_at_rule_with_declaration_and_params() {
        assert_parse_ok!(
            "@foo (bar) { a: b }",
            root!(
                0,
                18,
                vec![root_at_rule!(
                    0,
                    18,
                    "foo",
                    "(bar)",
                    vec![decl!(13, 16, "a", "b")]
                )]
            )
        );
    }

    #[test]
    fn parse_mested_at_rule() {
        assert_parse_ok!(
            "foo { hello: world ; foo : bar; @foo { a:b } }",
            root!(
                0,
                45,
                vec![root_rule!(
                    0,
                    45,
                    "foo",
                    vec![
                        decl!(6, 17, "hello", "world"),
                        decl!(21, 29, "foo", "bar"),
                        at_rule!(32, 43, "foo", "", vec![decl!(39, 41, "a", "b")])
                    ]
                )]
            )
        );
    }
}
