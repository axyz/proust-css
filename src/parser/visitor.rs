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
}

pub fn walk_root<V: Visitor>(visitor: &mut V, root: &Root) {
    for child in &root.nodes {
        match child {
            RootChild::Rule(rule) => visitor.visit_rule(rule),
            RootChild::AtRule(at_rule) => visitor.visit_at_rule(at_rule),
        }
    }
}

pub fn walk_rule<V: Visitor>(visitor: &mut V, rule: &Rule) {
    for child in &rule.nodes {
        match child {
            BlockChild::Declaration(decl) => visitor.visit_declaration(decl),
            BlockChild::AtRule(at_rule) => visitor.visit_at_rule(at_rule),
        }
    }
}

pub fn walk_at_rule<V: Visitor>(visitor: &mut V, at_rule: &AtRule) {
    for child in &at_rule.nodes {
        match child {
            BlockChild::Declaration(decl) => visitor.visit_declaration(decl),
            BlockChild::AtRule(at_rule) => visitor.visit_at_rule(at_rule),
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
        let mut p = Parser::new("foo { hello: world ; foo : bar; @foo { a:b } }");

        let mut m = Minifier::new();

        m.visit_root(&p.parse().unwrap());

        assert_eq!(m.css, "foo{hello:world;foo:bar;@foo {a:b;}}");
    }
}
