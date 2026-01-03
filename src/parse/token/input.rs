use crate::input::Input;
use crate::parse::token::cmd_arg_or_args1;
use crate::parse::token::{arg, parse_integer, ParserError};
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

pub(in crate::parse) fn parse_input(input: &'static str) -> InputResult<'static> {
    context(
        "Input",
        alt((
            parse_std_in,
            parse_file,
            parse_clip,
            parse_of,
            parse_gen,
            parse_repeat,
            context("Input::StdIn", map(success(()), |_| Input::StdIn)), // 默认从标准输入获取
        )),
    )
    .parse(input)
}

fn parse_std_in(input: &str) -> InputResult<'_> {
    context("Input::StdIn", map((tag_no_case("in"), space1), |_| Input::StdIn)).parse(input)
}

fn parse_file(input: &'static str) -> InputResult<'static> {
    context("Input::File", map(cmd_arg_or_args1("file"), |files| Input::File { files })).parse(input)
}

fn parse_clip(input: &str) -> InputResult<'_> {
    context("Input::Clip", map((tag_no_case("clip"), space1), |_| Input::Clip)).parse(input)
}

fn parse_of(input: &'static str) -> InputResult<'static> {
    context("Input::Of", map(cmd_arg_or_args1("of"), |values| Input::Of { values })).parse(input)
}

fn parse_gen(input: &str) -> InputResult<'_> {
    preceded(
        (tag_no_case("gen"), space1), // 丢弃：命令+空格
        terminated(parse_range_in_gen, space1),
    )
    .parse(input)
}

pub(in crate::parse) fn parse_range_in_gen(input: &str) -> InputResult<'_> {
    context(
        "Input::Gen",
        map(
            alt((
                // OPT 2025-12-28 23:16 使用opt重构？
                (parse_integer, char(','), char('='), parse_integer, char(','), verify(parse_integer, |s| *s != 0)), // 0,=10,2
                (parse_integer, char(','), success(' '), parse_integer, char(','), verify(parse_integer, |s| *s != 0)), // 0,10,2
                (parse_integer, char(','), char('='), parse_integer, success(','), success(1)), // 0,=10
                (parse_integer, char(','), success(' '), parse_integer, success(','), success(1)), // 0,10
                (
                    parse_integer,
                    char(','),
                    success(' '),
                    success(Integer::MAX),
                    char(','),
                    verify(parse_integer, |s| *s != 0),
                ), // 0,,2
                (parse_integer, success(','), success(' '), success(Integer::MAX), success(','), success(1)), // 0
            )),
            |(start, _, close, end, _, step)| Input::Gen { start, end, included: close == '=', step },
        ),
    )
    .parse(input)
}

fn parse_repeat(input: &'static str) -> InputResult<'static> {
    context(
        "Input::Repeat",
        map(
            terminated(
                preceded(
                    (tag_no_case("repeat"), space1),     // 丢弃：命令+空格
                    (arg, opt(preceded(space1, usize))), // 保留：重复的值和可选的次数
                ),
                space1, // 丢弃：结尾空格
            ),
            |(value, count)| Input::Repeat { value, count },
        ),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_std_in() {
        assert_eq!(parse_std_in("in "), Ok(("", Input::StdIn)));
        assert_eq!(parse_std_in("IN "), Ok(("", Input::StdIn)));
        assert!(parse_std_in("ina ").is_err());
    }

    #[test]
    fn test_parse_file() {
        assert_eq!(parse_file("file f.txt "), Ok(("", Input::File { files: vec!["f.txt".to_string()] })));
        assert_eq!(parse_file(r#"file "f .txt" "#), Ok(("", Input::File { files: vec!["f .txt".to_string()] })));
        assert_eq!(parse_file("file [ f.txt ] "), Ok(("", Input::File { files: vec!["f.txt".to_string()] })));
        assert_eq!(
            parse_file(r#"file [ f.txt "f .txt" ] "#),
            Ok(("", Input::File { files: vec!["f.txt".to_string(), "f .txt".to_string()] }))
        );
        assert!(parse_file("files f.txt ").is_err());
        assert!(parse_file("file [ ] ").is_err());
        assert!(parse_file("file [  ] ").is_err());
        assert!(parse_file("file [ [ ] ").is_err());
        assert!(parse_file("file [ ] ] ").is_err());
        assert!(parse_file("file [ f.txt [ ] ").is_err());
    }

    #[test]
    fn test_parse_clip() {
        assert_eq!(parse_clip("clip "), Ok(("", Input::Clip)));
        assert!(parse_clip("clip").is_err());
    }

    #[test]
    fn test_parse_of() {
        assert_eq!(parse_of("of str "), Ok(("", Input::Of { values: vec!["str".to_string()] })));
        assert_eq!(parse_of(r#"of "s tr" "#), Ok(("", Input::Of { values: vec!["s tr".to_string()] })));
        assert_eq!(parse_of("of [ str ] "), Ok(("", Input::Of { values: vec!["str".to_string()] })));
        assert_eq!(
            parse_of(r#"of [ str "s tr" ] "#),
            Ok(("", Input::Of { values: vec!["str".to_string(), "s tr".to_string()] }))
        );
        assert_eq!(
            parse_of("of [ \\[ \\[ \\] ] "),
            Ok(("", Input::Of { values: vec!["[".to_string(), "[".to_string(), "]".to_string()] }))
        );
        assert!(parse_of("ofs str ").is_err());
        assert!(parse_of("of [ ] ").is_err());
        assert!(parse_of("of [  ] ").is_err());
        assert!(parse_of("of [ [ ] ").is_err());
        assert!(parse_of("of [ ] ] ").is_err());
        assert!(parse_of("of [ str [ ] ").is_err());
    }

    #[test]
    fn test_parse_gen() {
        // 0,=10,2
        assert_eq!(parse_gen("gen 0,=10,2 "), Ok(("", Input::Gen { start: 0, end: 10, included: true, step: 2 })));
        // 0,10,2
        assert_eq!(parse_gen("gen 0,10,2 "), Ok(("", Input::Gen { start: 0, end: 10, included: false, step: 2 })));
        // 0,=10
        assert_eq!(parse_gen("gen 0,=10 "), Ok(("", Input::Gen { start: 0, end: 10, included: true, step: 1 })));
        // 0,10
        assert_eq!(parse_gen("gen 0,10 "), Ok(("", Input::Gen { start: 0, end: 10, included: false, step: 1 })));
        // 0,,2
        assert_eq!(parse_gen("gen 0,,2 "), Ok(("", Input::Gen { start: 0, end: i64::MAX, included: false, step: 2 })));
        // 0
        assert_eq!(parse_gen("gen 0 "), Ok(("", Input::Gen { start: 0, end: i64::MAX, included: false, step: 1 })));
    }

    #[test]
    fn test_parse_repeat() {
        assert_eq!(parse_repeat("repeat abc "), Ok(("", Input::Repeat { value: "abc".to_string(), count: None })));
        assert_eq!(
            parse_repeat("repeat abc 10 "),
            Ok(("", Input::Repeat { value: "abc".to_string(), count: Some(10) }))
        );
    }
}
