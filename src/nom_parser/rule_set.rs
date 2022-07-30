use nom::{
    combinator::{map, opt},
    IResult,
};

use crate::nom_parser::{Token, *};

pub fn rule_set(input: &str) -> IResult<&str, Vec<Token>> {
    map(
        delimited(
            ws_star,
            alt((
                tuple((at_rule, many0(rule_set))),
                tuple((comment, many0(rule_set))),
                tuple((rule, many0(rule_set))),
                tuple((declaration, many0(preceded(char(';'), rule_set)))),
            )),
            pair(opt(char(';')), ws_star),
        ),
        |(r, rs)| {
            rs.into_iter()
                .fold(vec![r], |prev, next| [prev, next].concat())
        },
    )(input)
}
