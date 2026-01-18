use crate::op::trim::{TrimArg, TrimMode};
use crate::op::{CaseArg, JoinInfo, Op, PeekArg, SortBy, TakeDropMode};
use crate::parse::token::condition::parse_cond;
use crate::parse::token::{arg, arg_end, arg_exclude_cmd, general_file_info, parse_arg_as, ParserError};
use crate::{Float, Integer};
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{space1, usize};
use nom::combinator::{map, opt, value, verify};
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
            parse_case,
            parse_replace,
            parse_trim,
            parse_uniq,
            parse_join,
            parse_take_drop,
            parse_count,
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
                context("(trailing_space1)", space1), // 结尾空格
            ),
            |file_info| match file_info {
                Some((file, append_opt, postfix_opt)) => Op::Peek(PeekArg::File {
                    file,
                    append: append_opt.is_some(),
                    crlf: postfix_opt.map(|s| s.eq_ignore_ascii_case("crlf")),
                }),
                None => Op::Peek(PeekArg::StdOut),
            },
        ),
    )
    .parse(input)
}

fn parse_case(input: &str) -> OpResult<'_> {
    context(
        "Op::Case",
        alt((
            map(terminated(tag_no_case(":lower"), space1), |_| Op::Case(CaseArg::Lower)),
            map(terminated(tag_no_case(":upper"), space1), |_| Op::Case(CaseArg::Upper)),
            map(terminated(tag_no_case(":case"), space1), |_| Op::Case(CaseArg::Switch)),
        )),
    )
    .parse(input)
}

fn parse_replace(input: &str) -> OpResult<'_> {
    context(
        "Op::Replace",
        map(
            preceded(
                tag_no_case(":replace"), // 丢弃：命令+空格
                terminated(
                    (
                        preceded(space1, context("<from>", arg)), // 被替换文本
                        (
                            preceded(space1, context("<to>", arg)),           // 替换为文本
                            opt(preceded(space1, context("<count>", usize))), // 替换次数
                            opt(preceded(space1, tag_no_case("nocase"))),     // 忽略大小写
                        ),
                    ),
                    context("(trailing_space1)", space1), // 丢弃：结尾空格
                ),
            ),
            |(from, (to, count_opt, nocase_opt))| Op::new_replace(from, to, count_opt, nocase_opt.is_some()),
        ),
    )
    .parse(input)
}

fn parse_trim(input: &str) -> OpResult<'_> {
    context(
        "Op::Trim",
        map(
            terminated(
                (
                    alt((
                        value((TrimMode::All, false), (tag_no_case(":trim"), arg_end)),
                        value((TrimMode::Left, false), (tag_no_case(":ltrim"), arg_end)),
                        value((TrimMode::Right, false), (tag_no_case(":rtrim"), arg_end)),
                        value((TrimMode::All, true), (tag_no_case(":trimc"), arg_end)),
                        value((TrimMode::Left, true), (tag_no_case(":ltrimc"), arg_end)),
                        value((TrimMode::Right, true), (tag_no_case(":rtrimc"), arg_end)),
                    )),
                    opt(preceded(
                        space1,
                        (context("<pattern>", arg_exclude_cmd), opt(preceded(space1, tag_no_case("nocase")))),
                    )),
                ),
                context("(trailing_space1)", space1), // 结尾空格
            ),
            |((trim_mode, char_mode), pattern_and_nocase)| match pattern_and_nocase {
                Some((pattern, nocase)) => {
                    Op::Trim(TrimArg::new(trim_mode, Some(pattern), char_mode, nocase.is_some()))
                }
                None => Op::Trim(TrimArg::new(trim_mode, None, char_mode, false)),
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
                tag_no_case(":uniq"),                         // 丢弃：命令
                opt(preceded(space1, tag_no_case("nocase"))), // 可选：空格+nocase选项
                context("(trailing_space1)", space1),         // 丢弃：结尾空格
            ),
            |nocase_opt| Op::Uniq(nocase_opt.is_some()),
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
                        context("<delimiter>", preceded(space1, arg_exclude_cmd)), // 分隔符
                        opt((
                            context("<prefix>", preceded(space1, arg_exclude_cmd)), // 前缀
                            opt((
                                context("<postfix>", preceded(space1, arg_exclude_cmd)), // 后缀
                                opt(context("<batch>", preceded(space1, verify(usize, |s| *s > 0)))), // 分组大小
                            )),
                        )),
                    )),
                ),
                context("(trailing_space1)", space1),
            ),
            |delimiter_opt| {
                let (join_info, batch) = if let Some((delimiter, prefix_opt)) = delimiter_opt {
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
                Op::new_join(join_info, batch)
            },
        ),
    )
    .parse(input)
}

fn parse_take_drop(input: &str) -> OpResult<'_> {
    context(
        "Op::TakeDrop",
        alt((
            map(
                preceded(
                    (tag_no_case(":take"), space1, tag_no_case("while"), space1),
                    context("<condition>", parse_cond),
                ),
                |cond| Op::new_take_drop(TakeDropMode::TakeWhile, cond),
            ),
            map(
                preceded(
                    (tag_no_case(":drop"), space1, tag_no_case("while"), space1),
                    context("<condition>", parse_cond),
                ),
                |cond| Op::new_take_drop(TakeDropMode::DropWhile, cond),
            ),
            map(preceded((tag_no_case(":take"), space1), context("<condition>", parse_cond)), |cond| {
                Op::new_take_drop(TakeDropMode::Take, cond)
            }),
            map(preceded((tag_no_case(":drop"), space1), context("<condition>", parse_cond)), |cond| {
                Op::new_take_drop(TakeDropMode::Drop, cond)
            }),
        )),
    )
    .parse(input)
}

fn parse_count(input: &str) -> OpResult<'_> {
    context("Op::Count", map(preceded(tag_no_case(":count"), space1), |_| Op::Count)).parse(input)
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
                                            context("<default>", parse_arg_as::<Integer>), // 默认整数值
                                            opt((space1, tag_no_case("desc"))),            // 可选逆序
                                        ),
                                    ),
                                    |(integer, desc): (Integer, Option<_>)| {
                                        (SortBy::Num(Some(integer), None), desc.is_some())
                                    },
                                ),
                                map(
                                    preceded(
                                        space1,
                                        (
                                            context("<default>", parse_arg_as::<Float>), // 默认浮点值
                                            opt((space1, tag_no_case("desc"))),          // 可选逆序
                                        ),
                                    ),
                                    |(float, desc): (Float, Option<_>)| {
                                        (SortBy::Num(None, Some(float)), desc.is_some())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::Cond;

    #[test]
    fn test_parse_case() {
        assert_eq!(parse_case(":lower "), Ok(("", Op::Case(CaseArg::Lower))));
        assert_eq!(parse_case(":upper "), Ok(("", Op::Case(CaseArg::Upper))));
        assert_eq!(parse_case(":case "), Ok(("", Op::Case(CaseArg::Switch))));
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
    fn test_parse_trim() {
        // trim
        assert_eq!(parse_trim(":trim "), Ok(("", Op::Trim(TrimArg::new(TrimMode::All, None, false, false)))));
        assert_eq!(
            parse_trim(":trim abc "),
            Ok(("", Op::Trim(TrimArg::new(TrimMode::All, Some("abc".to_owned()), false, false))))
        );
        assert_eq!(
            parse_trim(":trim abc nocase "),
            Ok(("", Op::Trim(TrimArg::new(TrimMode::All, Some("abc".to_owned()), false, true))))
        );
        assert_eq!(parse_trim(":trim :abc "), Ok((":abc ", Op::Trim(TrimArg::new(TrimMode::All, None, false, false)))));
        assert_eq!(
            parse_trim(":trim \\:abc "),
            Ok(("", Op::Trim(TrimArg::new(TrimMode::All, Some(":abc".to_owned()), false, false))))
        );
        // ltrim
        assert_eq!(parse_trim(":ltrim "), Ok(("", Op::Trim(TrimArg::new(TrimMode::Left, None, false, false)))));
        assert_eq!(
            parse_trim(":ltrim abc "),
            Ok(("", Op::Trim(TrimArg::new(TrimMode::Left, Some("abc".to_owned()), false, false))))
        );
        assert_eq!(
            parse_trim(":ltrim abc nocase "),
            Ok(("", Op::Trim(TrimArg::new(TrimMode::Left, Some("abc".to_owned()), false, true))))
        );
        assert_eq!(
            parse_trim(":ltrim :abc "),
            Ok((":abc ", Op::Trim(TrimArg::new(TrimMode::Left, None, false, false))))
        );
        assert_eq!(
            parse_trim(":ltrim \\:abc "),
            Ok(("", Op::Trim(TrimArg::new(TrimMode::Left, Some(":abc".to_owned()), false, false))))
        );
        // rtrim
        assert_eq!(parse_trim(":rtrim "), Ok(("", Op::Trim(TrimArg::new(TrimMode::Right, None, false, false)))));
        assert_eq!(
            parse_trim(":rtrim abc "),
            Ok(("", Op::Trim(TrimArg::new(TrimMode::Right, Some("abc".to_owned()), false, false))))
        );
        assert_eq!(
            parse_trim(":rtrim abc nocase "),
            Ok(("", Op::Trim(TrimArg::new(TrimMode::Right, Some("abc".to_owned()), false, true))))
        );
        assert_eq!(
            parse_trim(":rtrim :abc "),
            Ok((":abc ", Op::Trim(TrimArg::new(TrimMode::Right, None, false, false))))
        );
        assert_eq!(
            parse_trim(":rtrim \\:abc "),
            Ok(("", Op::Trim(TrimArg::new(TrimMode::Right, Some(":abc".to_owned()), false, false))))
        );
        // trimc
        assert_eq!(parse_trim(":trimc "), Ok(("", Op::Trim(TrimArg::new(TrimMode::All, None, true, false)))));
        assert_eq!(
            parse_trim(":trimc abc "),
            Ok(("", Op::Trim(TrimArg::new(TrimMode::All, Some("abc".to_owned()), true, false))))
        );
        assert_eq!(
            parse_trim(":trimc abc nocase "),
            Ok(("", Op::Trim(TrimArg::new(TrimMode::All, Some("abc".to_owned()), true, true))))
        );
        assert_eq!(parse_trim(":trimc :abc "), Ok((":abc ", Op::Trim(TrimArg::new(TrimMode::All, None, true, false)))));
        assert_eq!(
            parse_trim(":trimc \\:abc "),
            Ok(("", Op::Trim(TrimArg::new(TrimMode::All, Some(":abc".to_owned()), true, false))))
        );
        // ltrimc
        assert_eq!(parse_trim(":ltrimc "), Ok(("", Op::Trim(TrimArg::new(TrimMode::Left, None, true, false)))));
        assert_eq!(
            parse_trim(":ltrimc abc "),
            Ok(("", Op::Trim(TrimArg::new(TrimMode::Left, Some("abc".to_owned()), true, false))))
        );
        assert_eq!(
            parse_trim(":ltrimc abc nocase "),
            Ok(("", Op::Trim(TrimArg::new(TrimMode::Left, Some("abc".to_owned()), true, true))))
        );
        assert_eq!(
            parse_trim(":ltrimc :abc "),
            Ok((":abc ", Op::Trim(TrimArg::new(TrimMode::Left, None, true, false))))
        );
        assert_eq!(
            parse_trim(":ltrimc \\:abc "),
            Ok(("", Op::Trim(TrimArg::new(TrimMode::Left, Some(":abc".to_owned()), true, false))))
        );
        // rtrimc
        assert_eq!(parse_trim(":rtrimc "), Ok(("", Op::Trim(TrimArg::new(TrimMode::Right, None, true, false)))));
        assert_eq!(
            parse_trim(":rtrimc abc "),
            Ok(("", Op::Trim(TrimArg::new(TrimMode::Right, Some("abc".to_owned()), true, false))))
        );
        assert_eq!(
            parse_trim(":rtrimc abc nocase "),
            Ok(("", Op::Trim(TrimArg::new(TrimMode::Right, Some("abc".to_owned()), true, true))))
        );
        assert_eq!(
            parse_trim(":rtrimc :abc "),
            Ok((":abc ", Op::Trim(TrimArg::new(TrimMode::Right, None, true, false))))
        );
        assert_eq!(
            parse_trim(":rtrimc \\:abc "),
            Ok(("", Op::Trim(TrimArg::new(TrimMode::Right, Some(":abc".to_owned()), true, false))))
        );
    }

    #[test]
    fn test_parse_uniq() {
        assert_eq!(parse_uniq(":uniq "), Ok(("", Op::Uniq(false))));
        assert_eq!(parse_uniq(":uniq nocase "), Ok(("", Op::Uniq(true))));
    }

    #[test]
    fn test_parse_peek() {
        assert_eq!(parse_peek(":peek "), Ok(("", Op::Peek(PeekArg::StdOut))));
        assert_eq!(parse_peek(":peek :abc "), Ok((":abc ", Op::Peek(PeekArg::StdOut))));
        assert_eq!(
            parse_peek(":peek out.txt "),
            Ok(("", Op::Peek(PeekArg::File { file: "out.txt".to_string(), append: false, crlf: None })))
        );
        assert_eq!(
            parse_peek(":peek out.txt append "),
            Ok(("", Op::Peek(PeekArg::File { file: "out.txt".to_string(), append: true, crlf: None })))
        );
        assert_eq!(
            parse_peek(":peek out.txt append crlf "),
            Ok(("", Op::Peek(PeekArg::File { file: "out.txt".to_string(), append: true, crlf: Some(true) })))
        );
        assert_eq!(
            parse_peek(":peek out.txt crlf "),
            Ok(("", Op::Peek(PeekArg::File { file: "out.txt".to_string(), append: false, crlf: Some(true) })))
        );
        assert_eq!(
            parse_peek(r#":peek "out .txt" "#),
            Ok(("", Op::Peek(PeekArg::File { file: "out .txt".to_string(), append: false, crlf: None })))
        );
        assert_eq!(parse_peek(":peek :replace crlf "), Ok((":replace crlf ", Op::Peek(PeekArg::StdOut))));
    }

    #[test]
    fn test_parse_take_drop() {
        assert_eq!(
            parse_take_drop(":take while num "),
            Ok(("", Op::new_take_drop(TakeDropMode::TakeWhile, Cond::new_number(None, false))))
        );
        assert_eq!(
            parse_take_drop(":drop while num "),
            Ok(("", Op::new_take_drop(TakeDropMode::DropWhile, Cond::new_number(None, false))))
        );
        assert_eq!(
            parse_take_drop(":take num "),
            Ok(("", Op::new_take_drop(TakeDropMode::Take, Cond::new_number(None, false))))
        );
        assert_eq!(
            parse_take_drop(":drop num "),
            Ok(("", Op::new_take_drop(TakeDropMode::Drop, Cond::new_number(None, false))))
        );
    }

    #[test]
    fn test_parse_count() {
        assert_eq!(parse_count(":count "), Ok(("", Op::Count)));
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
