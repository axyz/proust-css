use crate::parser::{error::ParseError, tokenizer::*};
use std::iter::Peekable;

pub mod error;
pub mod tokenizer;
pub mod visitor;

#[derive(Debug, PartialEq)]
pub struct VectorAllocator<T> {
    pub data: Vec<Option<T>>
}

impl <T>VectorAllocator<T> {
    pub fn new() -> Self {
        VectorAllocator { data: vec![] }
    }

    pub fn push(&mut self, el: T) -> usize {
        let index = self.data.len();
        self.data.push(Some(el));
        index
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if let Some(el) = self.data.get(index) {
            el.as_ref()
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if let Some(el) = self.data.get_mut(index) {
            el.as_mut()
        } else {
            None
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if let Some(el) = self.data.remove(index) {
            self.data.insert(index, None);
            Some(el)
        } else {
            None
        }
    }
}


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
    pub allocator: &'a VectorAllocator<Node<'a>>,
    pub name: &'a str,
    pub params: &'a str,
    pub nodes: Vec<usize>,
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
pub enum Node<'a> {
    Rule(Rule<'a>),
    AtRule(AtRule<'a>),
    Declaration(Declaration<'a>),
    Comment(Comment<'a>),
}

#[derive(Debug, PartialEq)]
pub struct Rule<'a> {
    pub allocator: &'a VectorAllocator<Node<'a>>,
    pub selector: &'a str,
    pub nodes: Vec<usize>,
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
    pub allocator: &'a VectorAllocator<Node<'a>>,
    pub nodes: Vec<usize>,
    pub start: usize,
    pub end: usize,
}

pub struct Parser<'a> {
    tokenizer: Peekable<Tokenizer<'a>>,
    allocator: VectorAllocator<Node<'a>>,
    source: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Parser {
            tokenizer: Tokenizer::new(input).peekable(),
            allocator: VectorAllocator::new(),
            source: input,
            pos: 0,
        }
    }

    pub fn parse(&'a mut self) -> Result<Root, ParseError> {
        use TokenKind::*;
        let mut nodes: Vec<usize> = vec![];
        let start = self.pos;
        let allocator = &mut self.allocator;

        while let Some(token) = self.tokenizer.peek() {
            match token {
                Token(Space, ..) => {self.skip_while(|t| matches!(t, Some(Token(Space, ..))));}
                Token(At, ..) => {
                    nodes.push(allocator.push(Node::AtRule(self.parse_at_rule()?)));
                }
                Token(Word, ..) => {
                    nodes.push(allocator.push(Node::Rule(self.parse_rule()?)));
                }
                Token(Comment, ..) => {
                    nodes.push(allocator.push(Node::Comment(self.parse_comment()?)));}
                _ => {
                    self.next_token();
                }
            }
        }

        let end = self.pos;
        Ok(Root { allocator: &self.allocator, nodes, start, end })
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

    fn parse_rule(&'a mut self) -> Result<Rule<'a>, ParseError> {
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
            allocator: &self.allocator,
            selector,
            nodes: self.parse_block()?,
            start,
            end: self.pos,
        })
    }

    fn parse_at_rule(&'a mut self) -> Result<AtRule<'a>, ParseError> {
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
            allocator: &self.allocator,
            name,
            params,
            nodes: self.parse_block()?,
            start,
            end: self.pos,
        })
    }

    fn parse_declartion_or_at_rule_list(&'a mut self) -> Result<Vec<usize>, ParseError> {
        use TokenKind::*;
        let nodes: Vec<usize> = vec![];
        let allocator = &self.allocator;

        self.skip_while(|t| matches!(t, Some(Token(Space, ..))));

        while let Some(token) = self.tokenizer.peek() {
            match token {
                Token(Word, ..) => {
                    nodes.push(allocator.push(Node::Declaration(self.parse_declaration()?)));
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
                    nodes.push(allocator.push(Node::AtRule(self.parse_at_rule()?)));
                    self.skip_while(|t| {
                        matches!(t, Some(Token(Space, ..)) | Some(Token(Comment, ..)))
                    });
                    nodes.extend(self.parse_declartion_or_at_rule_list()?);
                }
                Token(Comment, ..) => nodes.push(allocator.push(Node::Comment(self.parse_comment()?))),
                _ => break,
            }
        }

        self.skip_while(|t| matches!(t, Some(Token(Space, ..))));

        Ok(nodes)
    }

    fn parse_block(&'a mut self) -> Result<Vec<usize>, ParseError> {
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
}
