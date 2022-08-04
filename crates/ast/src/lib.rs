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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
