use crate::parser::*;

pub trait Visitor {
    fn visit_root(&mut self, root: &Root)
    where
        Self: Sized,
    {
        walk_root(self, root);
    }

    fn visit_rule(&mut self, rule: &Rule)
    where
        Self: Sized,
    {
        walk_rule(self, rule);
    }

    fn visit_at_rule(&mut self, at_rule: &AtRule)
    where
        Self: Sized,
    {
        walk_at_rule(self, at_rule);
    }

    fn visit_declaration(&mut self, _: &Declaration) {}

    fn visit_comment(&mut self, _: &Comment) {}
}

pub fn walk_root<V: Visitor>(visitor: &mut V, root: &Root) {
    for child in &root.nodes {
        match child {
            RootChild::Rule(rule) => visitor.visit_rule(rule),
            RootChild::AtRule(at_rule) => visitor.visit_at_rule(at_rule),
            RootChild::Comment(comment) => visitor.visit_comment(comment),
        }
    }
}

pub fn walk_rule<V: Visitor>(visitor: &mut V, rule: &Rule) {
    for child in &rule.nodes {
        match child {
            BlockChild::Declaration(decl) => visitor.visit_declaration(decl),
            BlockChild::AtRule(at_rule) => visitor.visit_at_rule(at_rule),
            BlockChild::Comment(comment) => visitor.visit_comment(comment),
        }
    }
}

pub fn walk_at_rule<V: Visitor>(visitor: &mut V, at_rule: &AtRule) {
    for child in &at_rule.nodes {
        match child {
            BlockChild::Declaration(decl) => visitor.visit_declaration(decl),
            BlockChild::AtRule(at_rule) => visitor.visit_at_rule(at_rule),
            BlockChild::Comment(comment) => visitor.visit_comment(comment),
        }
    }
}

pub trait VisitorMut {
    fn visit_root(&mut self, root: &mut Root)
    where
        Self: Sized,
    {
        walk_root_mut(self, root);
    }

    fn visit_rule(&mut self, rule: &mut Rule)
    where
        Self: Sized,
    {
        walk_rule_mut(self, rule);
    }

    fn visit_at_rule(&mut self, at_rule: &mut AtRule)
    where
        Self: Sized,
    {
        walk_at_rule_mut(self, at_rule);
    }

    fn visit_declaration(&mut self, _: &mut Declaration) {}

    fn visit_comment(&mut self, _: &mut Comment) {}
}

pub fn walk_root_mut<V: VisitorMut>(visitor: &mut V, root: &mut Root) {
    for child in root.nodes.iter_mut() {
        match child {
            RootChild::Rule(rule) => visitor.visit_rule(rule),
            RootChild::AtRule(at_rule) => visitor.visit_at_rule(at_rule),
            RootChild::Comment(comment) => visitor.visit_comment(comment),
        }
    }
}

pub fn walk_rule_mut<V: VisitorMut>(visitor: &mut V, rule: &mut Rule) {
    for child in rule.nodes.iter_mut() {
        match child {
            BlockChild::Declaration(decl) => visitor.visit_declaration(decl),
            BlockChild::AtRule(at_rule) => visitor.visit_at_rule(at_rule),
            BlockChild::Comment(comment) => visitor.visit_comment(comment),
        }
    }
}

pub fn walk_at_rule_mut<V: VisitorMut>(visitor: &mut V, at_rule: &mut AtRule) {
    for child in at_rule.nodes.iter_mut() {
        match child {
            BlockChild::Declaration(decl) => visitor.visit_declaration(decl),
            BlockChild::AtRule(at_rule) => visitor.visit_at_rule(at_rule),
            BlockChild::Comment(comment) => visitor.visit_comment(comment),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visit_counter() {
        pub struct Counter {
            pub count: usize,
        }

        impl Counter {
            pub fn new() -> Self {
                Self { count: 0 }
            }
        }

        impl Visitor for Counter {
            fn visit_root(&mut self, root: &Root) {
                walk_root(self, root);
            }

            fn visit_rule(&mut self, rule: &Rule) {
                self.count += 1;
                walk_rule(self, rule);
            }

            fn visit_at_rule(&mut self, at_rule: &AtRule) {
                self.count += 1;
                walk_at_rule(self, at_rule);
            }

            fn visit_declaration(&mut self, _: &Declaration) {
                self.count += 1;
            }
        }
        let mut p = Parser::new("foo { hello: world ; foo : bar; @foo { a:b } }");

        let mut c = Counter::new();

        c.visit_root(&p.parse().unwrap());

        assert_eq!(c.count, 5);
    }

    #[test]
    fn visit_minify() {
        pub struct Minifier {
            pub css: String,
        }

        impl Minifier {
            pub fn new() -> Self {
                Self {
                    css: "".to_string(),
                }
            }
        }

        impl Visitor for Minifier {
            fn visit_root(&mut self, root: &Root) {
                walk_root(self, root);
            }

            fn visit_rule(&mut self, rule: &Rule) {
                self.css.push_str(&format!("{}{{", rule.selector));
                walk_rule(self, rule);
                self.css.push_str("}");
            }

            fn visit_at_rule(&mut self, at_rule: &AtRule) {
                self.css
                    .push_str(&format!("@{} {}{{", at_rule.name, at_rule.params));
                walk_at_rule(self, at_rule);
                self.css.push_str("}");
            }

            fn visit_declaration(&mut self, decl: &Declaration) {
                self.css.push_str(&format!("{}:{};", decl.prop, decl.value));
            }
        }
        let mut p = Parser::new("/* comment */foo { hello: world ; foo : bar; @foo { a:b } }");

        let mut m = Minifier::new();

        m.visit_root(&p.parse().unwrap());

        assert_eq!(m.css, "foo{hello:world;foo:bar;@foo {a:b;}}");
    }

    #[test]
    fn visit_prefixer() {
        pub struct Prefixer {}

        impl Prefixer {
            pub fn new() -> Self {
                Self {}
            }
        }

        impl VisitorMut for Prefixer {
            fn visit_root(&mut self, root: &mut Root) {
                walk_root_mut(self, root);
            }

            fn visit_rule(&mut self, rule: &mut Rule) {
                rule.selector = format!("-foo-{}", rule.selector);
                walk_rule_mut(self, rule);
            }

            fn visit_at_rule(&mut self, at_rule: &mut AtRule) {
                walk_at_rule_mut(self, at_rule);
            }

            fn visit_declaration(&mut self, decl: &mut Declaration) {
                decl.prop = format!("-foo-{}", decl.prop);
            }
        }

        let mut p = Parser::new("foo { hello: world }");

        let mut m = Prefixer::new();

        let mut root = p.parse().unwrap();

        m.visit_root(&mut root);

        assert_eq!(
            root,
            Root {
                start: 0,
                end: 19,
                nodes: vec![RootChild::Rule(Rule {
                    start: 0,
                    end: 19,
                    selector: "-foo-foo".to_string(),
                    nodes: vec![BlockChild::Declaration(Declaration {
                        start: 6,
                        end: 17,
                        prop: "-foo-hello".to_string(),
                        value: "world".to_string()
                    })]
                })]
            }
        );
    }
}
