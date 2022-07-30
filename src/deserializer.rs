use crate::nom_parser::*;
use serde::{Deserialize, Deserializer};

struct StyleSheet<'a> {
    pub root: Token<'a>,
}

impl<'de> Deserialize<'de> for StyleSheet<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(StyleSheet {
            root: Token::Root { nodes: vec![] },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn deserialize() {}
}
