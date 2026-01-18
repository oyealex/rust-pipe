pub(in crate::parse) mod condition;
mod config;
pub(in crate::parse) mod input;
pub(in crate::parse) mod op;
pub(in crate::parse) mod output;

use crate::err::RpErr;
use crate::input::Input;
use crate::op::Op;
use crate::output::Output;
use crate::parse::token::input::parse_input;
use crate::parse::token::op::parse_ops;
use crate::parse::token::output::parse_out;
use nom::branch::alt;
use nom::bytes::complete::{escaped, take_while1};
use nom::bytes::complete::{tag_no_case, take_while};
use nom::character::complete::{anychar, char};
use nom::character::complete::{none_of, space1};
use nom::combinator::{eof, map, map_res, opt, peek, recognize, value, verify};
use nom::error::context;
use nom::multi::{fold_many1, many_till};
use nom::sequence::{delimited, preceded};
use nom::{ExtendInto, IResult, Parser};
use nom_language::error::VerboseError;
use std::borrow::Cow;
use std::str::FromStr;

/// 解析错误的类型
pub(crate) type ParserError<'a> = VerboseError<&'a str>;

use crate::config::Config;
use crate::parse::token::config::parse_configs;
use crate::Num;
/// 重新导出解析整数的函数
pub(in crate::parse) use nom::character::complete::i64 as parse_integer;
pub(in crate::parse) use nom::number::complete::double as parse_float;

pub(in crate::parse) fn parse_num(input: &str) -> IResult<&str, Num, ParserError<'_>> {
    map_res(
        recognize(alt((
            // alt要求所有子解析器返回类型相同，所以使用value转换，最终会在外层完成Num转换。
            value((), verify(parse_float, |f| f.is_finite())), // 优先匹配浮点数
            value((), parse_integer),                          // 再匹配整数，如果优先匹配整数，则可能遗漏浮点数字符串
        ))),
        |s: &str| s.parse::<Num>(),
    )
    .parse(input)
}

// TODO 2026-01-10 02:24 完善上下文
#[allow(unused)]
pub(crate) fn parse(token: &str) -> Result<(&str, (Vec<Config>, Input, Vec<Op>, Output)), RpErr> {
    let (token, configs) = parse_configs(token).map_err(|err| RpErr::ParseConfigTokenErr(err.to_string()))?;
    let (token, input) = parse_input(token).map_err(|err| RpErr::ParseInputTokenErr(err.to_string()))?;
    let (token, ops) = parse_ops(token).map_err(|err| RpErr::ParseOpTokenErr(err.to_string()))?;
    let (token, output) = parse_out(token).map_err(|err| RpErr::ParseOutputTokenErr(err.to_string()))?;
    Ok((token, (configs, input, ops, output)))
}

pub(crate) fn parse_without_configs(token: &str) -> Result<(&str, (Input, Vec<Op>, Output)), RpErr> {
    let (token, input) = parse_input(token).map_err(|err| RpErr::ParseInputTokenErr(err.to_string()))?;
    let (token, ops) = parse_ops(token).map_err(|err| RpErr::ParseOpTokenErr(err.to_string()))?;
    let (token, output) = parse_out(token).map_err(|err| RpErr::ParseOutputTokenErr(err.to_string()))?;
    Ok((token, (input, ops, output)))
}

fn general_file_info<'a>(
    optional: bool,
) -> impl Parser<&'a str, Output = (String, Option<&'a str>, Option<&'a str>), Error = ParserError<'a>> {
    (
        context("<file>", if optional { arg_exclude_cmd } else { arg }), // 文件
        opt(preceded(space1, tag_no_case("append"))),                    // 是否追加
        opt(preceded(space1, alt((tag_no_case("lf"), tag_no_case("crlf"))))), // 换行符
    )
}
/// 构造一个解析器，解析`cmd arg [arg ][arg ][...]`，即解析至少一个参数直到遇到下一个冒号命令，
/// 如果参数以冒号开头需要使用`\:`代替开头的`:`。
fn cmd_arg1<'a>(
    cmd_name: &'a str, arg_name: &'static str,
) -> impl Parser<&'a str, Output = Vec<String>, Error = ParserError<'a>> {
    preceded(
        // 丢弃：命令标记
        tag_no_case(cmd_name),
        map(
            context(
                arg_name,
                verify(
                    many_till(
                        preceded(space1, arg_exclude_cmd), // 空格、参数
                        peek((space1, alt((cmd, eof)))),   // 直到下一个命令，但不消耗此命令，或达到结尾，忽略结果
                    ),
                    |(args, _)| !args.is_empty(), // 验证：参数非空
                ),
            ),
            |(args, _)| args,
        ),
    )
}

fn arg_exclude_cmd(input: &str) -> IResult<&str, String, ParserError<'_>> {
    context(
        "arg_exclude_cmd",
        map(verify(arg, |s: &String| whole_cmd_token(s).is_err()), |s| {
            if let Some(stripped) = s.strip_prefix("\\:") { format!(":{}", stripped) } else { s.to_owned() }
        }),
    )
    .parse(input)
}

fn cmd(input: &str) -> IResult<&str, &str, ParserError<'_>> {
    recognize((char(':'), take_while1(|c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')))
        .parse(input)
}

fn arg_end(input: &str) -> IResult<&str, &str, ParserError<'_>> {
    peek(alt((space1, eof))).parse(input)
}

/// 判断是否整个token为命令格式。
pub(in crate::parse) fn whole_cmd_token(input: &str) -> IResult<&str, &str, ParserError<'_>> {
    recognize((cmd, eof)).parse(input)
}

pub(in crate::parse) fn parse_arg_as<T>(input: &str) -> IResult<&str, T, ParserError<'_>>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    map(verify(arg, |s: &String| s.parse::<T>().is_ok()), |s| s.parse::<T>().unwrap()).parse(input)
}

pub(in crate::parse) fn parse_2_choice<'a>(
    primary: &'static str, second: &'static str,
) -> impl Parser<&'a str, Output = bool, Error = ParserError<'a>> {
    alt((value(true, tag_no_case(primary)), value(false, tag_no_case(second))))
}

/// 按照类PosixShell的规则解析单个参数
///
/// *参考：* https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html?spm=a2ty_o01.29997173.0.0.488051715w53V1#tag_18_02_02
pub(in crate::parse) fn arg(input: &str) -> IResult<&str, String, ParserError<'_>> {
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
    context(
        "arg_normal_part",
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
        ),
    )
    .parse(input)
}

fn double_quota_part(input: &str) -> IResult<&str, String, ParserError<'_>> {
    context(
        "arg_double_quota_part",
        map(
            delimited(
                char('"'), // 左双引号
                // 双引号内部允许转义，使用opt避免escaped_transform在遇到`""`时解析失败
                opt(escaped_trans(none_of(r#"\""#), '\\', normal_escape)),
                char('"'), // 右双引号
            ),
            |s| s.unwrap_or_default(),
        ),
    )
    .parse(input)
}

fn single_quota_part(input: &str) -> IResult<&str, &str, ParserError<'_>> {
    context(
        "arg_single_quota_part",
        delimited(
            char('\''),                // 左单引号
            take_while(|c| c != '\''), // 中间可以是任意字符，除了单引号
            char('\''),                // 右单引号
        ),
    )
    .parse(input)
}

fn normal_escape(c: char) -> Option<&'static str> {
    match c {
        '\\' => Some("\\"),
        '0' => Some("\0"),
        ' ' => Some(" "),
        '"' => Some("\""),
        '\'' => Some("\'"),
        'r' => Some("\r"),
        'n' => Some("\n"),
        't' => Some("\t"),
        _ => None,
    }
}

pub(in crate::parse) fn escape(input: &str) -> IResult<&str, String, ParserError<'_>> {
    escaped_trans(none_of("\\"), '\\', normal_escape).parse(input)
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
        let mut chars = s.chars();
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
    fn test_arg_non_cmd() {
        assert_eq!(arg_exclude_cmd("arg"), Ok(("", "arg".to_string())));
        assert_eq!(arg_exclude_cmd("arg1 arg2"), Ok((" arg2", "arg1".to_string())));
        assert!(arg_exclude_cmd(":arg1 arg2").is_err());
        assert_eq!(arg_exclude_cmd("\\:arg1 arg2"), Ok((" arg2", ":arg1".to_string())));
        assert_eq!(arg_exclude_cmd("'\\:arg1 arg2'"), Ok(("", ":arg1 arg2".to_string())));
        assert_eq!(arg_exclude_cmd("'arg a' :cmd"), Ok((" :cmd", "arg a".to_string())));
    }

    #[test]
    fn test_cmd_args1() {
        assert_eq!(cmd_arg1(":cmd", "arg").parse(":cmd arg "), Ok((" ", vec!["arg".to_string()])));
        assert_eq!(cmd_arg1(":cmd", "arg").parse(":cmd arg :cmd1"), Ok((" :cmd1", vec!["arg".to_string()])));
        assert_eq!(
            cmd_arg1(":cmd", "arg").parse(":cmd arg1 'arg2' :cmd1"),
            Ok((" :cmd1", vec!["arg1".to_string(), "arg2".to_string()]))
        );
        assert_eq!(
            cmd_arg1(":cmd", "arg").parse(":cmd \\:arg1 \\::arg2 \\:::arg3 :cmd4"),
            Ok((" :cmd4", vec![":arg1".to_string(), "::arg2".to_string(), ":::arg3".to_string()]))
        );
    }

    #[test]
    fn test_escape_string() {
        assert_eq!(escape(""), Ok(("", "".to_owned())));
        assert_eq!(escape("''"), Ok(("", "''".to_owned())));
        assert_eq!(escape("abc"), Ok(("", "abc".to_owned())));
        assert_eq!(escape("\\n abc"), Ok(("", "\n abc".to_owned())));
        assert_eq!(escape("\\m abc"), Ok(("", "\\m abc".to_owned())));
    }
}
