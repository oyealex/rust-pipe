use crate::err::RpErr;
use crate::input::Input;
use crate::parse::args::{consume_if_some, parse_arg_or_arg1};
use std::iter::Peekable;

pub(in crate::parse::args) fn parse_input(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Input, RpErr> {
    match args.peek() {
        Some(cmd) => {
            if cmd.eq_ignore_ascii_case("in") {
                parse_std_in(args)
            } else if cmd.eq_ignore_ascii_case("file") {
                parse_file(args)
            } else if cmd.eq_ignore_ascii_case("clip") {
                parse_clip(args)
            } else if cmd.eq_ignore_ascii_case("of") {
                parse_of(args)
            } else if cmd.eq_ignore_ascii_case("gen") {
                parse_gen(args)
            } else if cmd.eq_ignore_ascii_case("repeat") {
                parse_repeat(args)
            } else {
                Ok(Input::StdIn)
            }
        }
        None => Ok(Input::StdIn),
    }
}

fn parse_std_in(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Input, RpErr> {
    args.next(); // 消耗`in`
    Ok(Input::StdIn)
}

fn parse_file(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Input, RpErr> {
    args.next(); // 消耗`file`
    Ok(Input::File { files: parse_arg_or_arg1(args, "file", "file_name")? })
}

fn parse_clip(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Input, RpErr> {
    args.next(); // 消耗`clip`
    Ok(Input::Clip)
}

fn parse_of(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Input, RpErr> {
    args.next(); // 消耗`of`
    Ok(Input::Of { values: parse_arg_or_arg1(args, "of", "text")? })
}

fn parse_gen(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Input, RpErr> {
    args.next(); // 消耗`gen`
    let range = args.next().ok_or_else(|| RpErr::MissingArg { cmd: "gen", arg: "range" })?;
    match crate::parse::token::input::parse_range_in_gen(&range) {
        Ok((remaining, input)) => {
            if !remaining.is_empty() {
                Err(RpErr::UnexpectedRemaining { cmd: "gen", arg: "range", remaining: remaining.to_string() })
            } else {
                Ok(input)
            }
        }
        Err(e) => {
            Err(RpErr::ArgParseErr { cmd: "gen", arg: "range", arg_value: range.to_string(), error: e.to_string() })
        }
    }
}

fn parse_repeat(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Input, RpErr> {
    args.next(); // 消耗`repeat`
    let value = args.next().ok_or(RpErr::MissingArg { cmd: "repeat", arg: "value" })?;
    let count = consume_if_some(args, |s| s.parse::<usize>().ok());
    Ok(Input::Repeat { value, count })
}
