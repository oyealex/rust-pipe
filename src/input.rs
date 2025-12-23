use nom::branch::alt;
use nom::bytes::complete::take_while1;
use nom::bytes::{tag_no_case, take_until};
use nom::character::char;
use nom::character::complete::{space0, space1};
use nom::combinator::{map, opt};
use nom::multi::separated_list1;
use nom::sequence::{delimited, preceded, terminated};
use nom::{IResult, Parser};

#[derive(Debug, PartialEq)]
pub(crate) enum Input {
    StdIn,
    File { files: Vec<&'static str> },
    Clip,
    // Of { values: Vec<&'static str> },
}

pub(crate) fn parse(input: &'static str) -> IResult<&'static str, Input> {
    alt((parse_std_in, parse_file)).parse(input)
}

fn parse_std_in(input: &str) -> IResult<&str, Input> {
    map((tag_no_case("in"), space1), |_| Input::StdIn).parse(input)
}

fn parse_clip(input: &str) -> IResult<&str, Input> {
    map((tag_no_case("clip"), space1), |_| Input::Clip).parse(input)
}

fn parse_file(input: &'static str) -> IResult<&'static str, Input> {
    alt((
        _single_file,    // 单个文件
        _multiple_files, // 多个文件
    ))
    .parse(input)
}

fn _single_file(input: &'static str) -> IResult<&'static str, Input> {
    map(
        terminated(
            preceded(
                (tag_no_case("file"), space1), // 命令标记和空格
                _file_name,                    // 文件名
            ),
            space1, // 丢弃结尾空格
        ),
        |file: &str| Input::File { files: vec![file] },
    )
    .parse(input)
}

fn _multiple_files(input: &'static str) -> IResult<&'static str, Input> {
    map(
        terminated(
            preceded(
                (tag_no_case("file"), space1), // 命令标记和空格
                delimited(
                    (char('['), space1),                 // 丢弃前置括号和空格
                    separated_list1(space1, _file_name), // 解析空格分隔的文件名
                    (space1, char(']')),                 // 丢弃空格和后置括号
                ),
            ),
            space1, // 丢弃结尾空格
        ),
        |files: Vec<&str>| Input::File { files },
    )
    .parse(input)
}

fn _file_name(input: &str) -> IResult<&str, &str> {
    alt((
        delimited(char('"'), take_until("\""), char('"')), // 带引号的文件名
        take_while1(|c: char| !c.is_whitespace() && c != ']' && c != '"'), // 不带引号的文件名
    ))
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_std_in() {
        assert_eq!(parse_std_in("in "), Ok(("", Input::StdIn)),);
        assert_eq!(parse_std_in("in  "), Ok(("", Input::StdIn)),);
    }
    #[test]
    fn test_parse_std_in_fail() {
        assert!(parse_std_in("in").is_err());
        assert!(parse_std_in("input").is_err());
    }

    #[test]
    fn test_parse_clip() {
        assert_eq!(parse_clip("clip "), Ok(("", Input::Clip)),);
        assert_eq!(parse_clip("clip  "), Ok(("", Input::Clip)),);
    }
    #[test]
    fn test_parse_clip_fail() {
        assert!(parse_clip("clip").is_err());
        assert!(parse_clip("clipboard").is_err());
    }

    #[test]
    fn test_parse_single_file() {
        let expected = Ok((
            "",
            Input::File {
                files: vec!["sample.txt"],
            },
        ));
        assert_eq!(_single_file("file sample.txt "), expected);
        assert_eq!(_single_file("file sample.txt  "), expected);
        assert_eq!(
            _single_file(r#"file "sam ple.txt"  "#),
            Ok((
                "",
                Input::File {
                    files: vec!["sam ple.txt"]
                }
            ))
        );
    }
    #[test]
    fn test_parse_single_file_with_quoted_space() {
        assert_eq!(
            _single_file("file \"sam ple.txt\"  "),
            Ok((
                "",
                Input::File {
                    files: vec!["sam ple.txt"]
                }
            ))
        );
    }
    #[test]
    fn test_parse_single_file_fail() {
        assert!(_single_file("file").is_err());
        assert!(_single_file("filename").is_err());
        assert!(_single_file("file sample.txt").is_err());
    }

    #[test]
    fn test_parse_multiple_files() {
        let expected = Ok((
            "",
            Input::File {
                files: vec!["input1.txt", "input2.txt", "input 3.txt"],
            },
        ));
        assert_eq!(
            _multiple_files(r#"file [ input1.txt input2.txt "input 3.txt" ] "#),
            expected
        );
    }
    #[test]
    fn test_parse_multiple_files_fail() {
        assert!(_multiple_files("file").is_err());
        assert!(_multiple_files("filename").is_err());
        assert!(_multiple_files("file [input.txt ] ").is_err());
        assert!(_multiple_files("file [ input.txt] ").is_err());
        assert!(_multiple_files("file input.txt] ").is_err());
        assert!(_multiple_files("file [ input.txt ").is_err());
        assert!(_multiple_files("file [ input.txt ]").is_err());
    }
}
