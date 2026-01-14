use crate::err::RpErr;
use crate::input::Input;
use crate::op::Op;
use crate::output::Output;
use crate::parse::args::input::parse_input;
use crate::parse::args::op::parse_ops;
use crate::parse::args::output::parse_output;
use std::iter::Peekable;
use std::str::FromStr;

mod config;
mod input;
mod op;
mod output;
mod condition;

pub use config::parse_configs;

pub(crate) fn parse(mut args: Peekable<impl Iterator<Item = String>>) -> Result<(Input, Vec<Op>, Output), RpErr> {
    let input = parse_input(&mut args)?;
    let ops = parse_ops(&mut args)?;
    let output = parse_output(&mut args)?;
    let remaining = args.collect::<Vec<_>>();
    if !remaining.is_empty() { Err(RpErr::UnknownArgs { args: remaining }) } else { Ok((input, ops, output)) }
}

/// 解析一个或多个参数，参数不为命令格式，处理转义
fn parse_arg1(
    args: &mut Peekable<impl Iterator<Item = String>>, cmd: &'static str, arg: &'static str,
) -> Result<Vec<String>, RpErr> {
    let res = parse_arg0(args);
    if res.is_empty() { Err(RpErr::MissingArg { cmd, arg }) } else { Ok(res) }
}

/// 解析零个或多个参数，参数不为命令格式，处理转义
fn parse_arg0(args: &mut Peekable<impl Iterator<Item = String>>) -> Vec<String> {
    let mut res = Vec::new();
    while let Some(arg) = parse_opt_arg(args) {
        res.push(arg);
    }
    res
}

/// 解析一个可选的参数，参数不为命令格式，处理转义
fn parse_opt_arg(args: &mut Peekable<impl Iterator<Item = String>>) -> Option<String> {
    if let Some(value) = args.peek()
        && crate::parse::token::whole_cmd_token(value).is_err()
    {
        parse_arg(args)
    } else {
        None
    }
}

/// 解析一个必选参数，处理转义
fn parse_arg(args: &mut Peekable<impl Iterator<Item = String>>) -> Option<String> {
    args.next().map(escape)
}

fn escape<T: AsRef<str>>(arg: T) -> String {
    let arg = crate::parse::token::escape_string(arg.as_ref());
    if let Some(stripped) = arg.strip_prefix("::") { format!(":{}", stripped) } else { arg }
}

fn parse_tag_nocase(args: &mut Peekable<impl Iterator<Item = String>>, tag: &'static str) -> bool {
    if let Some(value) = args.peek()
        && value.eq_ignore_ascii_case(tag)
    {
        args.next();
        true
    } else {
        false
    }
}

fn parse_positive_usize(args: &mut Peekable<impl Iterator<Item = String>>) -> Option<usize> {
    if let Some(value) = args.peek() {
        let option = value.parse::<usize>().ok();
        if option.map_or(false, |u| u > 0) {
            args.next();
            option
        } else {
            None
        }
    } else {
        None
    }
}

fn parse_as<T: FromStr>(args: &mut Peekable<impl Iterator<Item = String>>) -> Option<T> {
    if let Some(value) = args.peek() {
        let option = value.parse::<T>().ok();
        if option.is_some() {
            args.next();
            option
        } else {
            None
        }
    } else {
        None
    }
}

/// 解析一般的文件信息`file[ append][ <crlf|lf>]`
/// 如果`optional`为`false`，则file参数必须非命令格式
fn parse_general_file_info(
    args: &mut Peekable<impl Iterator<Item = String>>, optional: bool,
) -> Option<(String, bool, Option<bool>)> {
    if let Some(file) = if optional { parse_opt_arg(args) } else { parse_arg(args) } {
        let (append, crlf) = if let Some(append_or_postfix) = args.peek() {
            if append_or_postfix.eq_ignore_ascii_case("append") {
                args.next(); // 消耗`append`
                if let Some(crlf) = args.peek() {
                    if crlf.eq_ignore_ascii_case("crlf") {
                        args.next(); // 消耗`crlf`
                        (true, Some(true))
                    } else if crlf.eq_ignore_ascii_case("lf") {
                        args.next(); // 消耗`lf`
                        (true, Some(false))
                    } else {
                        (true, None)
                    }
                } else {
                    (true, None)
                }
            } else if append_or_postfix.eq_ignore_ascii_case("crlf") {
                args.next(); // 消耗`crlf`
                (false, Some(true))
            } else if append_or_postfix.eq_ignore_ascii_case("lf") {
                args.next(); // 消耗`lf`
                (false, Some(false))
            } else {
                (false, None)
            }
        } else {
            (false, None)
        };
        Some((file, append, crlf))
    } else {
        None
    }
}

#[cfg(test)]
fn build_args(args_line: &'static str) -> Peekable<impl Iterator<Item = String>> {
    args_line
        .split(' ')
        .into_iter()
        .map(|s| crate::parse::token::arg(s).unwrap_or_default().1)
        .map(String::from)
        .peekable()
}
