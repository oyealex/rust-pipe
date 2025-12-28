use crate::input::Input;
use crate::parse::cmd_arg_or_args1;
use crate::parse::{parse_integer, ParserError};
use crate::Integer;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::char;
use nom::character::complete::space1;
use nom::combinator::{map, success, verify};
use nom::error::context;
use nom::sequence::{preceded, terminated};
use nom::{IResult, Parser};

pub(super) type InputResult<'a> = IResult<&'a str, Input, ParserError<'a>>;

pub(super) fn parse_input(input: &str) -> InputResult<'_> {
    alt((parse_std_in, parse_file, parse_clip, parse_of, parse_gen)).parse(input)
}

fn parse_std_in(input: &str) -> InputResult<'_> {
    context("Input::StdIn", map((tag_no_case("in"), space1), |_| Input::StdIn)).parse(input)
}

fn parse_file(input: &str) -> InputResult<'_> {
    context("Input::File", map(cmd_arg_or_args1("file"), |files| Input::File { files })).parse(input)
}

fn parse_clip(input: &str) -> InputResult<'_> {
    context("Input::Clip", map((tag_no_case("clip"), space1), |_| Input::Clip)).parse(input)
}

fn parse_of(input: &str) -> InputResult<'_> {
    context("Input::Of", map(cmd_arg_or_args1("of"), |values| Input::Of { values })).parse(input)
}

fn parse_gen(input: &str) -> InputResult<'_> {
    preceded(
        (tag_no_case("gen"), space1), // 丢弃：命令+空格
        terminated(parse_range_in_gen, space1),
    )
    .parse(input)
}

fn parse_range_in_gen(input: &str) -> InputResult<'_> {
    map(
        alt((
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
}
