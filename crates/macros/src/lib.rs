#[macro_export]
macro_rules! assert_parse_ok {
    ($input: expr, $output: expr) => {
        assert_eq!(Parser::new($input).parse(), Ok($output));
    };
}

#[macro_export]
macro_rules! root_comment {
    ($start: expr, $end: expr, $text: expr) => {
        RootChild::Comment(Comment {
            start: $start,
            end: $end,
            text: $text.to_string(),
        })
    };
}

#[macro_export]
macro_rules! comment {
    ($start: expr, $end: expr, $text: expr) => {
        BlockChild::Comment(Comment {
            start: $start,
            end: $end,
            text: $text.to_string(),
        })
    };
}

#[macro_export]
macro_rules! root {
    ($start: expr, $end: expr, $nodes: expr) => {
        Root {
            start: $start,
            end: $end,
            nodes: $nodes,
        }
    };
}

#[macro_export]
macro_rules! root_rule {
    ($start: expr, $end: expr, $selector: expr, $nodes: expr) => {
        RootChild::Rule(Rule {
            start: $start,
            end: $end,
            selector: $selector.to_string(),
            nodes: $nodes,
        })
    };
}

#[macro_export]
macro_rules! root_at_rule {
    ($start: expr, $end: expr, $name: expr, $params: expr, $nodes: expr) => {
        RootChild::AtRule(AtRule {
            start: $start,
            end: $end,
            name: $name.to_string(),
            params: $params.to_string(),
            nodes: $nodes,
        })
    };
}

#[macro_export]
macro_rules! at_rule {
    ($start: expr, $end: expr, $name: expr, $params: expr, $nodes: expr) => {
        BlockChild::AtRule(AtRule {
            start: $start,
            end: $end,
            name: $name.to_string(),
            params: $params.to_string(),
            nodes: $nodes,
        })
    };
}

#[macro_export]
macro_rules! decl {
    ($start: expr, $end: expr, $prop: expr, $value: expr) => {
        BlockChild::Declaration(Declaration {
            start: $start,
            end: $end,
            prop: $prop.to_string(),
            value: $value.to_string(),
        })
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
