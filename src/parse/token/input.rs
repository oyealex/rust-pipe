use crate::input::Input;
use crate::parse::token::parse_integer;
use crate::parse::token::{arg_exclude_cmd, cmd_arg1};
use crate::parse::RpParseErr;
use crate::Integer;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::space1;
use nom::character::complete::{char, usize};
use nom::combinator::{map, opt, success, verify};
use nom::error::context;
use nom::sequence::{preceded, terminated};
use nom::{IResult, Parser};

pub(in crate::parse) type InputIResult<'a> = IResult<&'a str, Input, RpParseErr<'a>>;

pub(in crate::parse) fn parse_input(input: &str) -> InputIResult<'_> {
    context(
        "Input",
        alt((
            parse_std_in,
            parse_file,
            #[cfg(windows)]
            parse_clip,
            parse_of,
            parse_gen,
            parse_repeat,
            context("Input::StdIn", map(success(()), |_| Input::new_std_in())), // 默认从标准输入获取
        )),
    )
    .parse(input)
}

fn parse_std_in(input: &str) -> InputIResult<'_> {
    context("Input::StdIn", map((tag_no_case(":in"), context("(trailing_space1)", space1)), |_| Input::new_std_in()))
        .parse(input)
}

fn parse_file(input: &str) -> InputIResult<'_> {
    context(
        "Input::File",
        map(terminated(cmd_arg1(":file", "<file>"), context("(trailing_space1)", space1)), |files| {
            Input::new_file(files)
        }),
    )
    .parse(input)
}

#[cfg(windows)]
fn parse_clip(input: &str) -> InputIResult<'_> {
    context("Input::Clip", map((tag_no_case(":clip"), context("(trailing_space1)", space1)), |_| Input::new_clip()))
        .parse(input)
}

fn parse_of(input: &str) -> InputIResult<'_> {
    context(
        "Input::Of",
        map(terminated(cmd_arg1(":of", "<text>"), context("(trailing_space1)", space1)), |values| {
            Input::new_of(values)
        }),
    )
    .parse(input)
}

fn parse_gen(input: &str) -> InputIResult<'_> {
    context(
        "Input::Gen",
        map(
            terminated(
                preceded(
                    tag_no_case(":gen"), // 命令
                    (
                        preceded(space1, parse_range_in_gen),                     // 范围
                        opt(context("<fmt>", preceded(space1, arg_exclude_cmd))), // 格式化字符串
                    ),
                ),
                context("(trailing_space1)", space1),
            ),
            |((start, end, step), fmt)| Input::new_gen(start, end, step, fmt),
        ),
    )
    .parse(input)
}

pub(in crate::parse) fn parse_range_in_gen(input: &str) -> IResult<&str, (Integer, Integer, Integer), RpParseErr<'_>> {
    map(
        (
            context("<start>", parse_integer), // 必选起始值
            opt(preceded(
                char(','), // 结束值分隔符
                (
                    opt(context("<end>", parse_integer)), //可选结束值
                    opt(preceded(char(','), verify(context("<step>", parse_integer), |s| *s != 0))), // 可选步长
                ),
            )),
        ),
        |(start, end_and_step_opt)| {
            if let Some((end_opt, step_opt)) = end_and_step_opt {
                (start, end_opt.unwrap_or(Integer::MAX), step_opt.unwrap_or(1))
            } else {
                (start, Integer::MAX, 1)
            }
        },
    )
    .parse(input)
}

fn parse_repeat(input: &str) -> InputIResult<'_> {
    context(
        "Input::Repeat",
        map(
            terminated(
                preceded(
                    tag_no_case(":repeat"),                                            // 命令
                    (preceded(space1, arg_exclude_cmd), opt(preceded(space1, usize))), // 重复的值和可选的次数
                ),
                space1, // 结尾空格
            ),
            |(value, count)| Input::new_repeat(value, count),
        ),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_std_in() {
        assert_eq!(parse_std_in(":in "), Ok(("", Input::new_std_in())));
        assert_eq!(parse_std_in(":IN "), Ok(("", Input::new_std_in())));
        assert!(parse_std_in(":ina ").is_err());
    }

    #[test]
    fn test_parse_file() {
        assert_eq!(parse_file(":file f.txt "), Ok(("", Input::new_file(vec!["f.txt".to_string()]))));
        assert_eq!(parse_file(":file [ "), Ok(("", Input::new_file(vec!["[".to_string()]))));
        assert_eq!(parse_file(":file ] "), Ok(("", Input::new_file(vec!["]".to_string()]))));
        assert_eq!(
            parse_file(":file [ ] [] "),
            Ok(("", Input::new_file(vec!["[".to_string(), "]".to_string(), "[]".to_string()])))
        );
        assert_eq!(parse_file(r#":file "f .txt" "#), Ok(("", Input::new_file(vec!["f .txt".to_string()]))));
        assert_eq!(parse_file(":file f.txt "), Ok(("", Input::new_file(vec!["f.txt".to_string()]))));
        assert_eq!(
            parse_file(r#":file f.txt "f .txt" "#),
            Ok(("", Input::new_file(vec!["f.txt".to_string(), "f .txt".to_string()])))
        );
        assert!(parse_file(":file ").is_err());
    }

    #[test]
    #[cfg(windows)]
    fn test_parse_clip() {
        assert_eq!(parse_clip(":clip "), Ok(("", Input::new_clip())));
        assert!(parse_clip(":clip").is_err());
    }

    #[test]
    fn test_parse_of() {
        assert_eq!(parse_of(":of value "), Ok(("", Input::new_of(vec!["value".to_string()]))));
        assert_eq!(parse_of(":of [ "), Ok(("", Input::new_of(vec!["[".to_string()]))));
        assert_eq!(parse_of(":of ] "), Ok(("", Input::new_of(vec!["]".to_string()]))));
        assert_eq!(
            parse_of(":of [ ] [] "),
            Ok(("", Input::new_of(vec!["[".to_string(), "]".to_string(), "[]".to_string()])))
        );
        assert_eq!(parse_of(r#":of "v lue" "#), Ok(("", Input::new_of(vec!["v lue".to_string()]))));
        assert_eq!(parse_of(":of value "), Ok(("", Input::new_of(vec!["value".to_string()]))));
        assert_eq!(
            parse_of(r#":of value "v lue" "#),
            Ok(("", Input::new_of(vec!["value".to_string(), "v lue".to_string()])))
        );
        assert!(parse_of(":of ").is_err());
    }

    #[test]
    fn test_parse_gen() {
        assert_eq!(parse_gen(":gen 0          "), Ok(("", Input::new_gen(0, Integer::MAX, 1, None))));
        assert_eq!(parse_gen(":gen 0,         "), Ok(("", Input::new_gen(0, Integer::MAX, 1, None))));
        assert_eq!(parse_gen(":gen 0,10       "), Ok(("", Input::new_gen(0, 10, 1, None))));
        assert_eq!(parse_gen(":gen 0,10,2     "), Ok(("", Input::new_gen(0, 10, 2, None))));
        assert_eq!(parse_gen(":gen 0,,2       "), Ok(("", Input::new_gen(0, Integer::MAX, 2, None))));
        assert_eq!(parse_gen(":gen 10,0       "), Ok(("", Input::new_gen(10, 0, 1, None))));
        assert_eq!(parse_gen(":gen 0,10,-1    "), Ok(("", Input::new_gen(0, 10, -1, None))));
        assert_eq!(parse_gen(":gen 0,10 n{v}  "), Ok(("", Input::new_gen(0, 10, 1, Some("n{v}".to_string())))));
        assert!(parse_gen(":gen 0,10,0     ").is_err());
    }

    #[test]
    fn test_parse_repeat() {
        assert_eq!(parse_repeat(":repeat abc "), Ok(("", Input::new_repeat("abc".to_string(), None))));
        assert_eq!(parse_repeat(":repeat abc 10 "), Ok(("", Input::new_repeat("abc".to_string(), Some(10)))));
    }
}
