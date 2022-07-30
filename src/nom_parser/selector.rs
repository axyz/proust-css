use nom::{
    branch::alt,
    character::complete::one_of,
    combinator::map,
    combinator::recognize,
    multi::{many1, separated_list0},
    IResult,
};

use crate::nom_parser::{Token, Token::*, *};

pub fn selector(input: &str) -> IResult<&str, Token> {
    map(
        recognize(separated_list0(
            whitespace,
            many1(alt((
                recognize(ident),
                recognize(string),
                digit,
                recognize(one_of("*+?<>|[].#~=$^()%:&")),
            ))),
        )),
        Selector,
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn parse_selector() {
        assert_valid!(selector, "*", Selector("*"));
        assert_valid!(selector, ".foo   ", "   ", Selector(".foo"));
        assert_valid!(selector, ".foo   ", "   ", Selector(".foo"));
        assert_valid!(selector, ".foo div", Selector(".foo div"));
        assert_valid!(
            selector,
            ".foo:hov\\xer div:nth(3+2) > #bar[class=~hello] + .aaa::bbb",
            Selector(".foo:hov\\xer div:nth(3+2) > #bar[class=~hello] + .aaa::bbb")
        );
    }
}
