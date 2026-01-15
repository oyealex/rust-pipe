use crate::output::Output;
use crate::parse::token::general_file_info;
use crate::parse::token::ParserError;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::space1;
use nom::combinator::{map, opt, success};
use nom::error::context;
use nom::sequence::{preceded, terminated};
use nom::IResult;
use nom::Parser;

pub(in crate::parse) type OutputResult<'a> = IResult<&'a str, Output, ParserError<'a>>;

pub(in crate::parse) fn parse_out(input: &str) -> OutputResult<'_> {
    context(
        "Output",
        alt((
            parse_to_std_out,
            parse_to_file,
            #[cfg(windows)]
            parse_to_clip,
            context("Output::Out", map(success(()), |_| Output::new_std_out())), // 最后默认使用`Output::Out`
        )),
    )
    .parse(input)
}

fn parse_to_std_out(input: &str) -> OutputResult<'_> {
    context(
        "Output::StdOut",
        map(
            (tag_no_case(":to"), space1, tag_no_case("out"), space1), // 命令
            |_| Output::new_std_out(),
        ),
    )
    .parse(input)
}

fn parse_to_file(input: &str) -> OutputResult<'_> {
    context(
        "Output::File",
        map(
            terminated(
                preceded(
                    (tag_no_case(":to"), space1, tag_no_case("file")), // 命令
                    preceded(space1, general_file_info(false)),
                ),
                space1, // 丢弃：结尾空格
            ),
            |(file, append_opt, postfix_opt): (String, Option<_>, Option<&str>)| {
                Output::new_file(file, append_opt.is_some(), postfix_opt.map(|s| s.eq_ignore_ascii_case("crlf")))
            },
        ),
    )
    .parse(input)
}

#[cfg(windows)]
fn parse_to_clip(input: &str) -> OutputResult<'_> {
    context(
        "Output::Clip",
        map(
            preceded(
                (tag_no_case(":to"), space1, tag_no_case("clip")), // 固定`:to clip`
                terminated(
                    opt(preceded(space1, alt((tag_no_case("lf"), tag_no_case("crlf"))))), // 换行符
                    space1,                                                               // 结尾空格
                ),
            ), // 丢弃：`to clip `
            |postfix_opt: Option<&str>| Output::new_clip(postfix_opt.map(|s| s.eq_ignore_ascii_case("crlf"))),
        ),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_to_file() {
        assert_eq!(parse_to_file(":to file out.txt "), Ok(("", Output::new_file("out.txt".to_string(), false, None))));
        assert_eq!(
            parse_to_file(":to file out.txt append "),
            Ok(("", Output::new_file("out.txt".to_string(), true, None)))
        );
        assert_eq!(
            parse_to_file(":to file out.txt append crlf "),
            Ok(("", Output::new_file("out.txt".to_string(), true, Some(true))))
        );
        assert_eq!(
            parse_to_file(":to file out.txt crlf "),
            Ok(("", Output::new_file("out.txt".to_string(), false, Some(true))))
        );
        assert_eq!(
            parse_to_file(r#":to file "out .txt" "#),
            Ok(("", Output::new_file("out .txt".to_string(), false, None)))
        );
        assert!(parse_to_file(":to").is_err());
        assert!(parse_to_file(":to file ").is_err());
        assert!(parse_to_file(":to file [").is_err());
    }

    #[test]
    #[cfg(windows)]
    fn test_parse_to_clip() {
        assert_eq!(parse_to_clip(":to clip "), Ok(("", Output::new_clip(None))));
        assert_eq!(parse_to_clip(":to  clip  "), Ok(("", Output::new_clip(None))));
        assert_eq!(parse_to_clip(":to clip lf "), Ok(("", Output::new_clip(Some(false)))));
        assert_eq!(parse_to_clip(":to clip crlf "), Ok(("", Output::new_clip(Some(true)))));
        assert!(parse_to_clip(":to ").is_err());
    }
}
