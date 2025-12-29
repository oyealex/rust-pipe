use crate::input::Input;
use crate::output::Output;
use crate::parse::input::parse_input;
use crate::parse::output::parse_out;
use nom::branch::alt;
use nom::bytes::complete::{tag_no_case, take_until, take_while1};
use nom::character::complete::char;
use nom::character::complete::space1;
use nom::combinator::{map, verify};
use nom::multi::many_till;
use nom::sequence::{delimited, preceded, terminated};
use nom::{IResult, Parser};
use nom_language::error::VerboseError;

mod input;
mod op;
mod output;

/// 解析错误的类型
pub(crate) type ParserError<'a> = VerboseError<&'a str>;

use crate::op::Op;
use crate::parse::op::parse_ops;
/// 重新导出解析整数的函数
pub(crate) use nom::character::complete::i64 as parse_integer;
use nom::error::context;

pub(crate) fn parse(
    input: &'static str,
) -> IResult<&'static str, (Input, Vec<Op>, Output), VerboseError<&'static str>> {
    (parse_input, parse_ops, parse_out).parse(input)
}

/// 构造一个解析器，支持解析：
///  - `cmd arg `：命令+单个参数；
///  - `cmd [ arg ] `：命令+单个参数，中括号包围；
///  - `cmd [ arg0 arg1 ] `：命令+一个以上的参数，中括号包围；
pub(super) fn cmd_arg_or_args1<'a>(
    cmd: &'static str,
) -> impl Parser<&'static str, Output = Vec<&'static str>, Error = ParserError<'static>> {
    alt((
        map(cmd_arg(cmd), |arg| vec![arg]), // 单个参数
        cmd_args1(cmd),                     // 多个参数
    ))
}

/// 构造一个解析器，支持解析：
///  - `cmd arg `：命令+单个参数；
pub(super) fn cmd_arg(
    cmd: &'static str,
) -> impl Parser<&'static str, Output = &'static str, Error = ParserError<'static>> {
    context(
        "Cmd_Arg",
        terminated(
            preceded(
                (tag_no_case(cmd), space1), // 丢弃：命令标记和空格
                arg,                        // 参数
            ),
            space1, // 丢弃：结尾空格
        ),
    )
}

/// 构造一个解析器，支持解析：
///  - `cmd [ arg ] `：命令+单个参数，中括号包围；
///  - `cmd [ arg0 arg1 ] `：命令+一个以上的参数，中括号包围；
pub(super) fn cmd_args1<'a>(
    cmd: &'static str,
) -> impl Parser<&'static str, Output = Vec<&'static str>, Error = ParserError<'static>> {
    context(
        "Cmd_Args1",
        map(
            terminated(
                preceded(
                    // 丢弃： 命令标记、空格、左括号、空格
                    (tag_no_case(cmd), space1, char('['), space1),
                    verify(
                        many_till(
                            terminated(arg, space1), // 参数、空格（丢弃）
                            char(']'),               // 忽略：右括号
                        ),
                        // OPT 2025-12-25 00:57 是否支持空的参数列表？
                        |(args, _)| !args.is_empty(), // 验证：参数非空
                    ),
                ),
                space1, // 丢弃：结尾空格
            ),
            |(args, _)| args,
        ),
    )
}

/// 解析器，支持解析单个参数。
pub(super) fn arg(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    // TODO 2025-12-24 23:29 实现完整的单个参数解析
    context(
        "Arg",
        verify(
            alt((
                delimited(char('"'), take_until("\""), char('"')),     // 带双引号的参数
                delimited(char('\''), take_until("\'"), char('\'')),   // 带单引号的参数
                take_while1(|c: char| !c.is_whitespace() && c != '"'), // 不带引号的参数
            )),
            |arg: &str| arg != "[" && arg != "]", // 验证：不能是单个括号
        ),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd_arg_or_args1() {
        assert_eq!(cmd_arg_or_args1("cmd").parse("cmd arg "), Ok(("", vec!["arg"])));
        assert_eq!(cmd_arg_or_args1("cmd").parse("cmd [ arg ] "), Ok(("", vec!["arg"])));
        assert_eq!(cmd_arg_or_args1("cmd").parse("cmd [ arg arg1 ] "), Ok(("", vec!["arg", "arg1"])));
        assert_eq!(cmd_arg_or_args1("cmd").parse(r#"cmd [ arg "arg 1" ] "#), Ok(("", vec!["arg", "arg 1"])));
        assert!(cmd_arg_or_args1("cmd").parse("cmd").is_err());
        assert!(cmd_arg_or_args1("cmd").parse("cmd ").is_err());
        assert!(cmd_arg_or_args1("cmd").parse("cmd [ arg ").is_err());
        assert!(cmd_arg_or_args1("cmd").parse("cmd [ ] ").is_err());
        assert!(cmd_arg_or_args1("cmd").parse("cmd [ [ ] ").is_err());
        assert!(cmd_arg_or_args1("cmd").parse("cmd ] ").is_err());
    }

    #[test]
    fn test_cmd_arg() {
        assert_eq!(cmd_arg("cmd").parse("cmd arg "), Ok(("", "arg")));
        assert_eq!(cmd_arg("cmd").parse(r#"cmd "ar g" "#), Ok(("", "ar g")));
        assert!(cmd_arg("cmd1").parse("cmd arg ").is_err());
    }

    #[test]
    fn test_cmd_args1() {
        assert_eq!(cmd_args1("cmd").parse("cmd [ arg ] "), Ok(("", vec!["arg"])));
        assert_eq!(cmd_args1("cmd").parse("cmd [ arg1 arg2 ] "), Ok(("", vec!["arg1", "arg2"])));
        assert_eq!(cmd_args1("cmd").parse(r#"cmd [ arg1 arg2 "arg 3" ] "#), Ok(("", vec!["arg1", "arg2", "arg 3"])));
        assert!(cmd_args1("cmd").parse(r#"cmd [ ] "#).is_err());
        assert!(cmd_args1("cmd").parse(r#"cmd [  ] "#).is_err());
    }

    #[test]
    fn test_arg() {
        assert_eq!(arg("hello"), Ok(("", "hello")));
        assert_eq!(arg("hello "), Ok((" ", "hello")));
        assert_eq!(arg("hello world"), Ok((" world", "hello")));
        assert_eq!(arg(r#"hello" world"#), Ok((r#"" world"#, "hello")));
        assert_eq!(arg(r#""hello " world"#), Ok((r#" world"#, "hello ")));
        assert!(arg(r#""hello "#).is_err());
        assert!(arg("[ ").is_err());
        assert!(arg("] ").is_err());
        assert_eq!(arg(r#""""#), Ok(("", "")));
        assert_eq!(arg(r#"''"#), Ok(("", "")));
    }
}
