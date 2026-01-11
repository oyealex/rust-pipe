use crate::condition::Cond;
use crate::op::{JoinInfo, Op, PeekTo, SortBy};
use crate::parse::token::{arg, arg_exclude_cmd, general_file_info, ParserError};
use crate::{Float, Integer};
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{space1, usize};
use nom::combinator::{map, opt, verify};
use nom::error::context;
use nom::multi::many0;
use nom::sequence::{delimited, preceded, terminated};
use nom::{IResult, Parser};

pub(in crate::parse) type OpsResult<'a> = IResult<&'a str, Vec<Op>, ParserError<'a>>;
pub(in crate::parse) type OpResult<'a> = IResult<&'a str, Op, ParserError<'a>>;

pub(in crate::parse) fn parse_ops(input: &str) -> OpsResult<'_> {
    context(
        "Op",
        many0(alt((
            parse_peek,
            parse_upper,
            parse_lower,
            parse_case,
            parse_replace,
            parse_uniq,
            parse_join,
            parse_sort,
        ))),
    )
    .parse(input)
}

fn parse_peek(input: &str) -> OpResult<'_> {
    context(
        "Op::Peek",
        map(
            terminated(
                preceded(
                    tag_no_case(":peek"),                           // 丢弃命令
                    opt(preceded(space1, general_file_info(true))), // 可选文件信息
                ),
                space1, // 结尾空格
            ),
            |file_info| match file_info {
                Some((file, append_opt, postfix_opt)) => Op::new_peek(PeekTo::File {
                    file,
                    append: append_opt.is_some(),
                    crlf: postfix_opt.map(|s| s.eq_ignore_ascii_case("crlf")),
                }),
                None => Op::new_peek(PeekTo::StdOut),
            },
        ),
    )
    .parse(input)
}

fn parse_upper(input: &str) -> OpResult<'_> {
    context("Op::Upper", map(terminated(tag_no_case(":upper"), space1), |_| Op::new_upper())).parse(input)
}

fn parse_lower(input: &str) -> OpResult<'_> {
    context("Op::Lower", map(terminated(tag_no_case(":lower"), space1), |_| Op::new_lower())).parse(input)
}

fn parse_case(input: &str) -> OpResult<'_> {
    context("Op::Case", map(terminated(tag_no_case(":case"), space1), |_| Op::new_case())).parse(input)
}

fn parse_replace(input: &str) -> OpResult<'_> {
    context(
        "Op::Replace",
        map(
            preceded(
                tag_no_case(":replace"), // 丢弃：命令+空格
                terminated(
                    (
                        preceded(space1, arg), // 被替换文本
                        (
                            preceded(space1, arg),                        // 替换为文本
                            opt(preceded(space1, usize)),                 // 替换次数
                            opt(preceded(space1, tag_no_case("nocase"))), // 忽略大小写
                        ),
                    ),
                    space1, // 丢弃：结尾空格
                ),
            ),
            |(from, (to, count_opt, nocase_opt))| Op::new_replace(from, to, count_opt, nocase_opt.is_some()),
        ),
    )
    .parse(input)
}

fn parse_uniq(input: &str) -> OpResult<'_> {
    context(
        "Op::Uniq",
        map(
            delimited(
                tag_no_case(":uniq"),                         // 丢弃：命令
                opt(preceded(space1, tag_no_case("nocase"))), // 可选：空格+nocase选项
                space1,                                       // 丢弃：结尾空格
            ),
            |nocase_opt| Op::new_uniq(nocase_opt.is_some()),
        ),
    )
    .parse(input)
}

fn parse_join(input: &str) -> OpResult<'_> {
    context(
        "Op::Join",
        map(
            terminated(
                preceded(
                    tag_no_case(":join"),
                    opt((
                        context("Op::Join::<delimiter>", preceded(space1, arg_exclude_cmd)), // 分隔符
                        opt((
                            context("Op::Join::<prefix>", preceded(space1, arg_exclude_cmd)), // 前缀
                            opt((
                                context("Op::Join::<postfix>", preceded(space1, arg_exclude_cmd)), // 后缀
                                opt(context("Op::Join::<size>", preceded(space1, verify(usize, |s| *s > 0)))), // 分组大小
                            )),
                        )),
                    )),
                ),
                context("Op::Join::ending_space1", space1),
            ),
            |delimiter_opt| {
                let (join_info, count) = if let Some((delimiter, prefix_opt)) = delimiter_opt {
                    if let Some((prefix, postfix_opt)) = prefix_opt {
                        if let Some((postfix, size_opt)) = postfix_opt {
                            if let Some(size) = size_opt {
                                (JoinInfo { delimiter, prefix, postfix }, Some(size))
                            } else {
                                (JoinInfo { delimiter, prefix, postfix }, None)
                            }
                        } else {
                            (JoinInfo { delimiter, prefix, postfix: String::new() }, None)
                        }
                    } else {
                        (JoinInfo { delimiter, prefix: String::new(), postfix: String::new() }, None)
                    }
                } else {
                    (JoinInfo::default(), None)
                };
                Op::new_join(join_info, count)
            },
        ),
    )
    .parse(input)
}

fn parse_sort(input: &str) -> OpResult<'_> {
    context(
        "Op::Sort",
        map(
            terminated(
                preceded(
                    tag_no_case(":sort"), // 丢弃：命令
                    alt((
                        preceded(
                            // case 1：按数值排序
                            (space1, tag_no_case("num")), // 固定tag
                            alt((
                                map(
                                    preceded(
                                        space1,
                                        (
                                            verify(arg, |s: &String| s.parse::<Integer>().is_ok()), // 默认整数值
                                            opt((space1, tag_no_case("desc"))),                     // 可选逆序
                                        ),
                                    ),
                                    |(num, desc): (String, Option<_>)| {
                                        (SortBy::Num(num.parse::<Integer>().ok(), None), desc.is_some())
                                    },
                                ),
                                map(
                                    preceded(
                                        space1,
                                        (
                                            verify(arg, |s: &String| s.parse::<Float>().is_ok()), // 默认浮点值
                                            opt((space1, tag_no_case("desc"))),                   // 可选逆序
                                        ),
                                    ),
                                    |(num, desc): (String, Option<_>)| {
                                        (SortBy::Num(None, num.parse::<Float>().ok()), desc.is_some())
                                    },
                                ),
                                map(opt((space1, tag_no_case("desc"))), |desc| {
                                    (SortBy::Num(None, None), desc.is_some())
                                }), // 无任何默认值
                            )),
                        ),
                        map((space1, tag_no_case("random")), |_| (SortBy::Random, false)), // case 2：随机排序
                        map(
                            // case 3：按字典序排序（默认）
                            (opt((space1, tag_no_case("nocase"))), opt((space1, tag_no_case("desc")))),
                            |(nc, desc): (Option<_>, Option<_>)| (SortBy::Text(nc.is_some()), desc.is_some()),
                        ),
                    )),
                ),
                space1, // 结尾空格
            ),
            |(sort_by, desc): (SortBy, bool)| Op::new_sort(sort_by, desc),
        ),
    )
    .parse(input)
}

pub(in crate::parse) fn parse_condition_text(input: &str) -> IResult<&str, Cond, ParserError<'_>> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_upper() {
        assert_eq!(parse_upper(":upper "), Ok(("", Op::new_upper())));
    }

    #[test]
    fn test_parse_lower() {
        assert_eq!(parse_lower(":lower "), Ok(("", Op::new_lower())));
    }

    #[test]
    fn test_parse_case() {
        assert_eq!(parse_case(":case "), Ok(("", Op::new_case())));
    }

    #[test]
    fn test_parse_replace() {
        assert_eq!(
            parse_replace(r#":replace abc "" "#),
            Ok(("", Op::new_replace("abc".to_string(), "".to_string(), None, false)))
        );
        assert_eq!(
            parse_replace(":replace abc 123 "),
            Ok(("", Op::new_replace("abc".to_string(), "123".to_string(), None, false)))
        );
        assert_eq!(
            parse_replace(":replace abc 123 5 "),
            Ok(("", Op::new_replace("abc".to_string(), "123".to_string(), Some(5), false)))
        );
        assert_eq!(
            parse_replace(":replace abc 123 5 nocase "),
            Ok(("", Op::new_replace("abc".to_string(), "123".to_string(), Some(5), true)))
        );
        assert_eq!(
            parse_replace(r#":replace abc "" 5 nocase "#),
            Ok(("", Op::new_replace("abc".to_string(), "".to_string(), Some(5), true)))
        );
        assert_eq!(
            parse_replace(r#":replace abc "" nocase "#),
            Ok(("", Op::new_replace("abc".to_string(), "".to_string(), None, true)))
        );
        assert_eq!(
            parse_replace(r#":replace abc '' nocase "#),
            Ok(("", Op::new_replace("abc".to_string(), "".to_string(), None, true)))
        );
        assert_eq!(
            parse_replace(r#":replace abc def nocase "#),
            Ok(("", Op::new_replace("abc".to_string(), "def".to_string(), None, true)))
        );
    }

    #[test]
    fn test_parse_uniq() {
        assert_eq!(parse_uniq(":uniq "), Ok(("", Op::new_uniq(false))));
        assert_eq!(parse_uniq(":uniq nocase "), Ok(("", Op::new_uniq(true))));
    }

    #[test]
    fn test_parse_peek() {
        assert_eq!(parse_peek(":peek "), Ok(("", Op::new_peek(PeekTo::StdOut))));
        assert_eq!(parse_peek(":peek :abc "), Ok((":abc ", Op::new_peek(PeekTo::StdOut))));
        assert_eq!(
            parse_peek(":peek out.txt "),
            Ok(("", Op::new_peek(PeekTo::File { file: "out.txt".to_string(), append: false, crlf: None })))
        );
        assert_eq!(
            parse_peek(":peek out.txt append "),
            Ok(("", Op::new_peek(PeekTo::File { file: "out.txt".to_string(), append: true, crlf: None })))
        );
        assert_eq!(
            parse_peek(":peek out.txt append crlf "),
            Ok(("", Op::new_peek(PeekTo::File { file: "out.txt".to_string(), append: true, crlf: Some(true) })))
        );
        assert_eq!(
            parse_peek(":peek out.txt crlf "),
            Ok(("", Op::new_peek(PeekTo::File { file: "out.txt".to_string(), append: false, crlf: Some(true) })))
        );
        assert_eq!(
            parse_peek(r#":peek "out .txt" "#),
            Ok(("", Op::new_peek(PeekTo::File { file: "out .txt".to_string(), append: false, crlf: None })))
        );
        assert_eq!(parse_peek(":peek :replace crlf "), Ok((":replace crlf ", Op::new_peek(PeekTo::StdOut))));
    }

    #[test]
    fn test_parse_sort() {
        assert_eq!(parse_sort(":sort "), Ok(("", Op::new_sort(SortBy::Text(false), false))));
        assert_eq!(parse_sort(":sort desc "), Ok(("", Op::new_sort(SortBy::Text(false), true))));
        assert_eq!(parse_sort(":sort nocase "), Ok(("", Op::new_sort(SortBy::Text(true), false))));
        assert_eq!(parse_sort(":sort nocase desc "), Ok(("", Op::new_sort(SortBy::Text(true), true))));
        assert_eq!(parse_sort(":sort num "), Ok(("", Op::new_sort(SortBy::Num(None, None), false))));
        assert_eq!(parse_sort(":sort num desc "), Ok(("", Op::new_sort(SortBy::Num(None, None), true))));
        assert_eq!(parse_sort(":sort num 10 "), Ok(("", Op::new_sort(SortBy::Num(Some(10), None), false))));
        assert_eq!(parse_sort(":sort num 10 desc "), Ok(("", Op::new_sort(SortBy::Num(Some(10), None), true))));
        assert_eq!(parse_sort(":sort num 10.5 "), Ok(("", Op::new_sort(SortBy::Num(None, Some(10.5)), false))));
        assert_eq!(parse_sort(":sort num 10.5 desc "), Ok(("", Op::new_sort(SortBy::Num(None, Some(10.5)), true))));
        assert_eq!(parse_sort(":sort num -10 "), Ok(("", Op::new_sort(SortBy::Num(Some(-10), None), false))));
        assert_eq!(parse_sort(":sort num -10 desc "), Ok(("", Op::new_sort(SortBy::Num(Some(-10), None), true))));
        assert_eq!(parse_sort(":sort num -10.5 "), Ok(("", Op::new_sort(SortBy::Num(None, Some(-10.5)), false))));
        assert_eq!(parse_sort(":sort num -10.5 desc "), Ok(("", Op::new_sort(SortBy::Num(None, Some(-10.5)), true))));
        assert_eq!(parse_sort(":sort random "), Ok(("", Op::new_sort(SortBy::Random, false))));

        assert_eq!(parse_sort(":sort random desc "), Ok(("desc ", Op::new_sort(SortBy::Random, false))));
    }
}
