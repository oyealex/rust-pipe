use crate::err::RpErr;
use crate::op::Op;
use crate::parse::args::{consume_if, consume_if_some};
use std::iter::Peekable;

pub(in crate::parse::args) fn parse_ops(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Vec<Op>, RpErr> {
    let mut ops = vec![];
    while let Some(op) = parse_op(args)? {
        ops.push(op);
    }
    Ok(ops)
}

fn parse_op(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Option<Op>, RpErr> {
    match args.peek() {
        Some(cmd) => {
            if cmd.eq_ignore_ascii_case("upper") {
                parse_upper(args)
            } else if cmd.eq_ignore_ascii_case("lower") {
                parse_lower(args)
            } else if cmd.eq_ignore_ascii_case("replace") {
                parse_replace(args)
            } else if cmd.eq_ignore_ascii_case("uniq") {
                parse_uniq(args)
            } else {
                Ok(None)
            }
        }
        None => Ok(None),
    }
}

fn parse_upper(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Option<Op>, RpErr> {
    args.next();
    Ok(Some(Op::new_upper()))
}

fn parse_lower(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Option<Op>, RpErr> {
    args.next();
    Ok(Some(Op::new_lower()))
}

fn parse_replace(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Option<Op>, RpErr> {
    args.next();
    // 被替换字符串必选，直接消耗
    if let Some(from) = args.next() {
        // 替换目标字符串必选，直接消耗
        if let Some(to) = args.next() {
            let count_opt = consume_if_some(args, |s| s.parse::<usize>().ok());
            let nocase = consume_if(args, |s| s.eq_ignore_ascii_case("nocase")).is_some();
            Ok(Some(Op::new_replace(from, to, count_opt, nocase)))
        } else {
            Err(RpErr::MissingArg { cmd: "replace", arg: "to" })
        }
    } else {
        Err(RpErr::MissingArg { cmd: "replace", arg: "from" })
    }
}

fn parse_uniq(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Option<Op>, RpErr> {
    args.next();
    let nocase = args.peek().map(|nocase| nocase.eq_ignore_ascii_case("nocase")).unwrap_or(false);
    if nocase {
        args.next();
    }
    Ok(Some(Op::new_uniq(nocase)))
}
