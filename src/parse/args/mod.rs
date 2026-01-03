use crate::err::RpErr;
use crate::input::Input;
use crate::op::Op;
use crate::output::Output;
use crate::parse::args::input::parse_input;
use crate::parse::args::op::parse_ops;
use crate::parse::args::output::parse_output;
use std::iter::Peekable;
use std::str::FromStr;

mod input;
mod op;
mod output;

pub(crate) fn parse(mut args: Peekable<impl Iterator<Item = String>>) -> Result<(Input, Vec<Op>, Output), RpErr> {
    let input = parse_input(&mut args)?;
    let ops = parse_ops(&mut args)?;
    let output = parse_output(&mut args)?;
    let remaining = args.collect::<Vec<_>>();
    if !remaining.is_empty() { Err(RpErr::UnknownArgs { args: remaining }) } else { Ok((input, ops, output)) }
}

fn parse_arg_or_arg1(
    args: &mut Peekable<impl Iterator<Item = String>>, cmd: &'static str, arg: &'static str,
) -> Result<Vec<String>, RpErr> {
    match args.next() {
        // 至少有一个值，直接消耗
        Some(value) => {
            if value == "[" {
                // 多值开始
                let mut values = Vec::new();
                while let Some(value) = args.next() {
                    if value == "]" {
                        // 多值结束
                        return if values.is_empty() { Err(RpErr::ArgNotEnough { cmd, arg }) } else { Ok(values) };
                    } else {
                        values.push(escaped(value))
                    }
                }
                Err(RpErr::UnclosingMultiArg { cmd, arg })
            } else if value == "]" {
                // 未开启的多值结束
                Err(RpErr::UnexpectedClosingBracket { cmd, arg })
            } else {
                Ok(vec![escaped(value)])
            }
        }
        None => Err(RpErr::MissingArg { cmd, arg }),
    }
}

fn escaped(arg: String) -> String {
    if arg == "\\[" || arg == "\\]" { arg[1..].to_string() } else { arg }
}

fn consume_if<F>(args: &mut Peekable<impl Iterator<Item = String>>, f: F) -> Option<String>
where
    F: FnOnce(&String) -> bool,
{
    if let Some(value) = args.peek()
        && f(value)
    {
        args.next();
        Some(args.next().unwrap())
    } else {
        None
    }
}

fn consume_if_then_map<F, M, U>(args: &mut Peekable<impl Iterator<Item = String>>, f: F, m: M) -> Option<U>
where
    F: FnOnce(&String) -> bool,
    M: FnOnce(String) -> U,
{
    if let Some(value) = args.peek()
        && f(value)
    {
        args.next();
        Some(args.next().unwrap()).map(m)
    } else {
        None
    }
}

fn consume_if_some<M, U>(args: &mut Peekable<impl Iterator<Item = String>>, m: M) -> Option<U>
where
    M: FnOnce(&String) -> Option<U>,
{
    if let Some(value) = args.peek() {
        let option = m(value);
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
