use crate::op::Op;
use crate::parse::{arg, ParserError};
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{space1, usize};
use nom::combinator::{map, opt};
use nom::error::context;
use nom::multi::many0;
use nom::sequence::{delimited, preceded, terminated};
use nom::{IResult, Parser};

pub(super) type OpsResult<'a> = IResult<&'a str, Vec<Op>, ParserError<'a>>;
pub(super) type OpResult<'a> = IResult<&'a str, Op, ParserError<'a>>;

pub(super) fn parse_ops(input: &'static str) -> OpsResult<'static> {
    context("Op", many0(alt((parse_upper, parse_lower, parse_replace, parse_uniq)))).parse(input)
}

fn parse_upper(input: &str) -> OpResult<'_> {
    context("Op::Upper", map(terminated(tag_no_case("upper"), space1), |_| Op::Upper)).parse(input)
}

fn parse_lower(input: &str) -> OpResult<'_> {
    context("Op::Lower", map(terminated(tag_no_case("lower"), space1), |_| Op::Lower)).parse(input)
}

fn parse_replace(input: &'static str) -> OpResult<'static> {
    context(
        "Op::Replace",
        map(
            preceded(
                (tag_no_case("replace"), space1), // 丢弃：命令+空格
                terminated(
                    // 兼容:
                    //  abc
                    //  abc def
                    //  abc def 10
                    //  abc def 10 nocase
                    //  abc def    nocase
                    (
                        arg, // 被替换文本
                        opt(preceded(
                            space1,
                            (
                                arg,                                          // 替换为文本
                                opt(preceded(space1, usize)),                 // 替换次数
                                opt(preceded(space1, tag_no_case("nocase"))), // 忽略大小写
                            ),
                        )),
                    ),
                    space1,
                ),
            ), // 丢弃：结尾空格
            |(from, to_opt)| {
                let mut to = "";
                let mut count = None;
                let mut nocase = false;
                if let Some((to_value, count_opt, nocase_opt)) = to_opt {
                    to = to_value;
                    count = count_opt;
                    nocase = nocase_opt.is_some();
                }
                Op::Replace { from, to, count, nocase }
            },
        ),
    )
    .parse(input)
}

fn parse_uniq(input: &str) -> OpResult<'_> {
    context(
        "Op::Uniq",
        map(
            delimited(
                tag_no_case("uniq"),                          // 丢弃：命令
                opt(preceded(space1, tag_no_case("nocase"))), // 可选：空格+nocase选项
                space1,
            ), // 丢弃：结尾空格
            |nocase_opt| Op::Uniq { nocase: nocase_opt.is_some() },
        ),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_upper() {
        assert_eq!(parse_upper("upper "), Ok(("", Op::Upper)));
    }

    #[test]
    fn test_parse_lower() {
        assert_eq!(parse_lower("lower "), Ok(("", Op::Lower)));
    }

    #[test]
    fn test_parse_replace() {
        assert_eq!(
            parse_replace("replace abc "),
            Ok(("", Op::Replace { from: "abc", to: "", count: None, nocase: false }))
        );
        assert_eq!(
            parse_replace("replace abc 123 "),
            Ok(("", Op::Replace { from: "abc", to: "123", count: None, nocase: false }))
        );
        assert_eq!(
            parse_replace("replace abc 123 5 "),
            Ok(("", Op::Replace { from: "abc", to: "123", count: Some(5), nocase: false }))
        );
        assert_eq!(
            parse_replace("replace abc 123 5 nocase "),
            Ok(("", Op::Replace { from: "abc", to: "123", count: Some(5), nocase: true }))
        );
        assert_eq!(
            parse_replace(r#"replace abc "" 5 nocase "#),
            Ok(("", Op::Replace { from: "abc", to: "", count: Some(5), nocase: true }))
        );
        assert_eq!(
            parse_replace(r#"replace abc "" nocase "#),
            Ok(("", Op::Replace { from: "abc", to: "", count: None, nocase: true }))
        );
        assert_eq!(
            parse_replace(r#"replace abc '' nocase "#),
            Ok(("", Op::Replace { from: "abc", to: "", count: None, nocase: true }))
        );
        assert_eq!(
            parse_replace(r#"replace abc def nocase "#),
            Ok(("", Op::Replace { from: "abc", to: "def", count: None, nocase: true }))
        );
    }

    #[test]
    fn test_parse_uniq() {
        assert_eq!(parse_uniq("uniq "), Ok(("", Op::Uniq { nocase: false })));
        assert_eq!(parse_uniq("uniq nocase "), Ok(("", Op::Uniq { nocase: true })));
    }
}
