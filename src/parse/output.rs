use crate::output::Output;
use crate::parse::arg;
use crate::parse::ParserError;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::space1;
use nom::combinator::{map, success};
use nom::error::context;
use nom::sequence::{preceded, terminated};
use nom::IResult;
use nom::Parser;

pub(super) type OutputResult<'a> = IResult<&'a str, Output, ParserError<'a>>;

pub(super) fn parse_out(input: &'static str) -> OutputResult<'static> {
    context(
        "Output",
        alt((
            parse_to_file,
            parse_to_clip,
            context("Output::Out", map(success(()), |_| Output::Out)), // 最后默认使用`Output::Out`
        )),
    )
    .parse(input)
}

fn parse_to_file(input: &'static str) -> OutputResult<'static> {
    context(
        "Output::File",
        map(
            terminated(
                preceded(
                    (tag_no_case("to"), space1, tag_no_case("file"), space1), // 丢弃：`to file `
                    arg,                                                      // 文件
                ),
                space1, // 丢弃：结尾空格
            ),
            |file| Output::File { file },
        ),
    )
    .parse(input)
}

fn parse_to_clip(input: &str) -> OutputResult<'_> {
    context(
        "Output::Clip",
        map(
            (tag_no_case("to"), space1, tag_no_case("clip"), space1), // 丢弃：`to clip `
            |_| Output::Clip,
        ),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_to_file() {
        assert_eq!(parse_to_file("to file out.txt "), Ok(("", Output::File { file: "out.txt" })));
        assert_eq!(parse_to_file(r#"to file "out .txt" "#), Ok(("", Output::File { file: "out .txt" })));
        assert!(parse_to_file("to").is_err());
        assert!(parse_to_file("to file ").is_err());
        assert!(parse_to_file("to file [").is_err());
    }

    #[test]
    fn test_parse_to_clip() {
        assert_eq!(parse_to_clip("to clip "), Ok(("", Output::Clip)));
        assert_eq!(parse_to_clip("to  clip  "), Ok(("", Output::Clip)));
        assert!(parse_to_clip("to ").is_err());
    }
}
