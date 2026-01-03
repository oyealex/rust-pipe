pub mod input;
pub mod op;
pub mod output;

use crate::err::RpErr;
use crate::input::Input;
use crate::op::Op;
use crate::output::Output;
use crate::parse::token::input::parse_input;
use crate::parse::token::op::parse_ops;
use crate::parse::token::output::parse_out;
use nom::branch::alt;
use nom::bytes::complete::escaped;
use nom::bytes::complete::{tag_no_case, take_while};
use nom::character::complete::{anychar, char};
use nom::character::complete::{none_of, space1};
use nom::combinator::{map, opt, verify};
use nom::error::context;
use nom::multi::{fold_many1, many_till};
use nom::sequence::{delimited, preceded, terminated};
use nom::{ExtendInto, IResult, Parser};
use nom_language::error::VerboseError;
use std::borrow::Cow;

/// 解析错误的类型
pub(crate) type ParserError<'a> = VerboseError<&'a str>;

/// 重新导出解析整数的函数
pub(crate) use nom::character::complete::i64 as parse_integer;


/// 从一行文本中解析出`(Input, Vec<Op>, Output)`。
///
/// # Arguments
///
/// * `input`: 带解析的文本。
///
/// returns: 解析结果，成功类型：`(&str, (Input, Vec<Op>, Output))`
pub(crate) fn parse(token: &'static str) -> Result<(&'static str, (Input, Vec<Op>, Output)), RpErr> {
    let (remaining, input) = parse_input(token).map_err(|err| RpErr::ParseInputTokenErr(err.to_string()))?;
    let (remaining, ops) = parse_ops(token).map_err(|err| RpErr::ParseOpTokenErr(err.to_string()))?;
    let (remaining, output) = parse_out(token).map_err(|err| RpErr::ParseOutputTokenErr(err.to_string()))?;
    Ok((remaining, (input, ops, output)))
}

/// 构造一个解析器，支持解析：
///  - `cmd arg `：命令+单个参数；
///  - `cmd [ arg ] `：命令+单个参数，中括号包围；
///  - `cmd [ arg0 arg1 ] `：命令+一个以上的参数，中括号包围；
fn cmd_arg_or_args1<'a>(
    cmd: &'static str,
) -> impl Parser<&'static str, Output = Vec<String>, Error = ParserError<'static>> {
    alt((
        map(cmd_arg(cmd), |arg| vec![arg]), // 单个参数
        cmd_args1(cmd),                     // 多个参数
    ))
}

/// 构造一个解析器，支持解析：
///  - `cmd arg `：命令+单个参数；
/// 返回`arg`。
fn cmd_arg(cmd: &'static str) -> impl Parser<&'static str, Output = String, Error = ParserError<'static>> {
    context(
        "Cmd_Arg",
        terminated(
            preceded(
                (tag_no_case(cmd), space1), // 丢弃：命令标记和空格
                arg_escaped_bracket,        // 参数
            ),
            space1, // 丢弃：结尾空格
        ),
    )
}

/// 构造一个解析器，支持解析：
///  - `cmd [ arg ] `：命令+单个参数，中括号包围；
///  - `cmd [ arg0 arg1 ] `：命令+一个以上的参数，中括号包围；
/// 返回`args`。
fn cmd_args1<'a>(cmd: &'static str) -> impl Parser<&'static str, Output = Vec<String>, Error = ParserError<'static>> {
    context(
        "Cmd_Args1",
        map(
            terminated(
                preceded(
                    // 丢弃： 命令标记、空格、左括号、空格
                    (tag_no_case(cmd), space1, char('['), space1),
                    verify(
                        many_till(
                            terminated(arg_escaped_bracket, space1), // 参数、空格（丢弃）
                            char(']'),                               // 忽略：右括号
                        ),
                        |(args, _)| !args.is_empty(), // 验证：参数非空
                    ),
                ),
                space1, // 丢弃：结尾空格
            ),
            |(args, _)| args,
        ),
    )
}

fn arg_escaped_bracket(input: &str) -> IResult<&str, String, ParserError<'_>> {
    context(
        "arg_escaped_bracket",
        map(verify(arg, |s: &String| s != "[" && s != "]"), |s| match &s as &str {
            "\\[" => "[".to_string(),
            "\\]" => "]".to_string(),
            _ => s,
        }),
    )
    .parse(input)
}

/// 按照类PosixShell的规则解析单个参数
///
/// *参考：* https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html?spm=a2ty_o01.29997173.0.0.488051715w53V1#tag_18_02_02
fn arg(input: &str) -> IResult<&str, String, ParserError<'_>> {
    fold_many1(
        alt((
            map(normal_part, |string| Cow::Owned(string)),
            map(double_quota_part, |string| Cow::Owned(string)),
            map(single_quota_part, |string| Cow::Borrowed(string)),
        )),
        String::new,
        |mut acc, item| {
            match item {
                Cow::Borrowed(string) => acc.push_str(string),
                Cow::Owned(string) => acc.push_str(&string),
            }
            acc
        },
    )
    .parse(input)
}

fn normal_part(input: &str) -> IResult<&str, String, ParserError<'_>> {
    verify(
        escaped_trans(
            // 首先持续匹配直到遇到转义字符，
            //    这里要考虑引号空格等可能导致解析中断的字符
            //    转义字符在后续处理，不能直接捕获，所以也需要排除
            none_of(r#"\ "'"#),
            '\\',          // 定义转义字符前缀字符
            normal_escape, // 转换转义字符，然后继续使用第一个普通解析器继续解析
        ),
        |s: &String| !s.is_empty(), // 确保escaped_transform不会返回空字符串，导致外层fold_many1直接失败
    )
    .parse(input)
}

fn double_quota_part(input: &str) -> IResult<&str, String, ParserError<'_>> {
    map(
        delimited(
            char('"'), // 左双引号
            // 双引号内部允许转义，使用opt避免escaped_transform在遇到`""`时解析失败
            opt(escaped_trans(none_of(r#"\""#), '\\', normal_escape)),
            char('"'), // 右双引号
        ),
        |s| s.unwrap_or_default(),
    )
    .parse(input)
}

fn single_quota_part(input: &str) -> IResult<&str, &str, ParserError<'_>> {
    delimited(
        char('\''),                // 左单引号
        take_while(|c| c != '\''), // 中间可以是任意字符，除了单引号
        char('\''),                // 右单引号
    )
    .parse(input)
}

fn normal_escape(c: char) -> Option<&'static str> {
    match c {
        '\\' => Some("\\"),
        ' ' => Some(" "),
        '"' => Some("\""),
        'r' => Some("\r"),
        'n' => Some("\n"),
        't' => Some("\t"),
        _ => None,
    }
}

/// `nom::bytes::complete::escaped_transform`的优化版本，escaped_transform处理不在转义范围内的反斜杠字符时如果需要
/// 保留原本的字符内容，第三个参数需要返回String类型，导致许多零碎的String碎片，此优化版本一次性解析后手动替换转义
/// 字符，原样保留非转义内容，并且一次性申请String对象。
fn escaped_trans<'a, F, FO, ExtendItem>(
    normal: F, control_char: char, mut escape: impl FnMut(char) -> Option<&'static str> + 'static,
) -> impl Parser<&'a str, Output = String, Error = ParserError<'a>>
where
    FO: ExtendInto<Item = ExtendItem, Extender = String>,
    F: Parser<&'a str, Output = FO, Error = ParserError<'a>>,
{
    map(escaped(normal, control_char, anychar), move |s| {
        let mut result = String::with_capacity(s.len());
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some(escape_char) => {
                        match escape(escape_char) {
                            Some(replacement) => {
                                result.push_str(replacement);
                            }
                            None => {
                                // 不在转义范围内，原样保留
                                result.push('\\');
                                result.push(escape_char);
                            }
                        }
                    }
                    None => result.push('\\'), // 结尾是 \，按原样保留
                }
            } else {
                result.push(c);
            }
        }
        result
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arg() {
        assert_eq!(arg("''"), Ok(("", "".to_string())));
        assert_eq!(arg(r#""""#), Ok(("", "".to_string())));
        assert_eq!(arg("hello"), Ok(("", "hello".to_string())));
        assert_eq!(arg(r#""hello""#), Ok(("", "hello".to_string())));
        assert_eq!(arg(r#"'hello'"#), Ok(("", "hello".to_string())));
        assert_eq!(arg(r#"'"hello"'"#), Ok(("", r#""hello""#.to_string())));
        assert_eq!(arg(r#""'hello'""#), Ok(("", "'hello'".to_string())));
        assert_eq!(arg("hello world"), Ok((" world", "hello".to_string())));
        assert_eq!(arg(r#"\\hello\ world\nhello\tworld\""#), Ok(("", "\\hello world\nhello\tworld\"".to_string())));
        assert_eq!(arg(r#"hello\ "world"\ and\ 'greet'"#), Ok(("", "hello world and greet".to_string())));
        assert_eq!(arg(r#"he""llo\ "world"\ a''nd\ 'greet'"#), Ok(("", "hello world and greet".to_string())));
        assert_eq!(arg(r#"\\a\\b\\c"#), Ok(("", r#"\a\b\c"#.to_string())));
    }

    #[test]
    fn test_cmd_arg_or_args1() {
        assert_eq!(cmd_arg_or_args1("cmd").parse("cmd arg "), Ok(("", vec!["arg".to_string()])));
        assert_eq!(cmd_arg_or_args1("cmd").parse("cmd [ arg ] "), Ok(("", vec!["arg".to_string()])));
        assert_eq!(
            cmd_arg_or_args1("cmd").parse("cmd [ arg arg1 ] "),
            Ok(("", vec!["arg".to_string(), "arg1".to_string()]))
        );
        assert_eq!(
            cmd_arg_or_args1("cmd").parse(r#"cmd [ arg "arg 1" ] "#),
            Ok(("", vec!["arg".to_string(), "arg 1".to_string()]))
        );
        assert!(cmd_arg_or_args1("cmd").parse("cmd").is_err());
        assert!(cmd_arg_or_args1("cmd").parse("cmd ").is_err());
        assert!(cmd_arg_or_args1("cmd").parse("cmd [ arg ").is_err());
        assert!(cmd_arg_or_args1("cmd").parse("cmd [ ] ").is_err());
        assert!(cmd_arg_or_args1("cmd").parse("cmd [ [ ] ").is_err());
        assert!(cmd_arg_or_args1("cmd").parse("cmd ] ").is_err());
    }

    #[test]
    fn test_cmd_arg() {
        assert_eq!(cmd_arg("cmd").parse("cmd arg "), Ok(("", "arg".to_string())));
        assert_eq!(cmd_arg("cmd").parse(r#"cmd "ar g" "#), Ok(("", "ar g".to_string())));
        assert_eq!(cmd_arg("cmd").parse("cmd \\[ "), Ok(("", "[".to_string())));
        assert_eq!(cmd_arg("cmd").parse("cmd \\] "), Ok(("", "]".to_string())));
        assert!(cmd_arg("cmd1").parse("cmd arg ").is_err());
        assert!(cmd_arg("cmd").parse("cmd [ ").is_err());
        assert!(cmd_arg("cmd").parse("cmd ] ").is_err());
    }

    #[test]
    fn test_cmd_args1() {
        assert_eq!(cmd_args1("cmd").parse("cmd [ arg ] "), Ok(("", vec!["arg".to_string()])));
        assert_eq!(
            cmd_args1("cmd").parse("cmd [ arg1 arg2 ] "),
            Ok(("", vec!["arg1".to_string(), "arg2".to_string()]))
        );
        assert_eq!(
            cmd_args1("cmd").parse(r#"cmd [ arg1 arg2 "arg 3" ] "#),
            Ok(("", vec!["arg1".to_string(), "arg2".to_string(), "arg 3".to_string()]))
        );
        assert!(cmd_args1("cmd").parse(r#"cmd [ ] "#).is_err());
        assert!(cmd_args1("cmd").parse(r#"cmd [  ] "#).is_err());
    }
}
