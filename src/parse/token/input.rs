use crate::input::Input;
use crate::parse::token::{arg_exclude_cmd, cmd_arg1};
use crate::parse::token::{parse_integer, ParserError};
use crate::Integer;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::space1;
use nom::character::complete::{char, usize};
use nom::combinator::{map, opt, success, verify};
use nom::error::context;
use nom::sequence::{preceded, terminated};
use nom::{IResult, Parser};

pub(in crate::parse) type InputResult<'a> = IResult<&'a str, Input, ParserError<'a>>;

pub(in crate::parse) fn parse_input(input: &str) -> InputResult<'_> {
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

fn parse_std_in(input: &str) -> InputResult<'_> {
    context("Input::StdIn", map((tag_no_case(":in"), space1), |_| Input::new_std_in())).parse(input)
}

fn parse_file(input: &str) -> InputResult<'_> {
    context(
        "Input::File",
        map(terminated(cmd_arg1(":file", "Input::File::<file_name>"), space1), |files| Input::new_file(files)),
    )
    .parse(input)
}

#[cfg(windows)]
fn parse_clip(input: &str) -> InputResult<'_> {
    context("Input::Clip", map((tag_no_case(":clip"), space1), |_| Input::new_clip())).parse(input)
}

fn parse_of(input: &str) -> InputResult<'_> {
    context("Input::Of", map(terminated(cmd_arg1(":of", "Input::Of::<text>"), space1), |values| Input::new_of(values)))
        .parse(input)
}

fn parse_gen(input: &str) -> InputResult<'_> {
    context(
        "Input::Gen",
        map(
            terminated(
                preceded(
                    tag_no_case(":gen"), // 命令
                    (
                        context("Input::Gen::<range>", preceded(space1, parse_range_in_gen)), // 范围
                        opt(preceded(space1, arg_exclude_cmd)),                               // 格式化字符串
                    ),
                ),
                space1, // 结尾空白
            ),
            |((start, end, step), fmt)| Input::new_gen(start, end, step, fmt),
        ),
    )
    .parse(input)
}

pub(in crate::parse) fn parse_range_in_gen(input: &str) -> IResult<&str, (Integer, Integer, Integer), ParserError<'_>> {
    map(
        alt((
            // OPT 2025-12-28 23:16 使用opt重构？
            (parse_integer, char(','), parse_integer, char(','), verify(parse_integer, |s| *s != 0)), // 0,=10,2
            (parse_integer, char(','), parse_integer, char(','), verify(parse_integer, |s| *s != 0)), // 0,10,2
            (parse_integer, char(','), parse_integer, success(','), success(1)),                      // 0,=10
            (parse_integer, char(','), parse_integer, success(','), success(1)),                      // 0,10
            (parse_integer, char(','), success(Integer::MAX), char(','), verify(parse_integer, |s| *s != 0)), // 0,,2
            (parse_integer, success(','), success(Integer::MAX), success(','), success(1)),           // 0
        )),
        |(start, _, end, _, step)| (start, end, step),
    )
    .parse(input)
}

fn parse_repeat(input: &str) -> InputResult<'_> {
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
        assert_eq!(parse_gen(":gen 0,10       "), Ok(("", Input::new_gen(0, 10, 1, None))));
        assert_eq!(parse_gen(":gen 0,10,2     "), Ok(("", Input::new_gen(0, 10, 2, None))));
        assert_eq!(parse_gen(":gen 0,,2       "), Ok(("", Input::new_gen(0, Integer::MAX, 2, None))));
        assert_eq!(parse_gen(":gen 10,0       "), Ok(("", Input::new_gen(10, 0, 1, None))));
        assert_eq!(parse_gen(":gen 0,10,-1    "), Ok(("", Input::new_gen(0, 10, -1, None))));
        assert_eq!(parse_gen(":gen 0,10 n{v}  "), Ok(("", Input::new_gen(0, 10, 1, Some("n{v}".to_string())))));
    }

    #[test]
    fn test_parse_repeat() {
        assert_eq!(parse_repeat(":repeat abc "), Ok(("", Input::new_repeat("abc".to_string(), None))));
        assert_eq!(parse_repeat(":repeat abc 10 "), Ok(("", Input::new_repeat("abc".to_string(), Some(10)))));
    }
}
