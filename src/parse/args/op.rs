use crate::err::RpErr;
use crate::op::{JoinInfo, Op, PeekTo, SortBy};
use crate::parse::args::condition::parse_cond;
use crate::parse::args::{
    parse_arg, parse_as, parse_general_file_info, parse_opt_arg, parse_positive_usize, parse_tag_nocase,
};
use crate::{Float, Integer};
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
        Some(op) => {
            let lower_op = op.to_ascii_lowercase();
            Ok(match lower_op.as_str() {
                ":peek" => Some(parse_peek(args)?),
                ":upper" => Some(parse_upper(args)?),
                ":lower" => Some(parse_lower(args)?),
                ":case" => Some(parse_case(args)?),
                ":replace" => Some(parse_replace(args)?),
                ":uniq" => Some(parse_uniq(args)?),
                ":join" => Some(parse_join(args)?),
                ":drop" => Some(parse_drop(args)?),
                ":keep" => Some(parse_keep(args)?),
                ":sort" => Some(parse_sort(args)?),
                _ => None,
            })
        }
        None => Ok(None),
    }
}

fn parse_peek(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Op, RpErr> {
    args.next();
    if let Some((file, append, crlf)) = parse_general_file_info(args, true) {
        Ok(Op::new_peek(PeekTo::File { file, append, crlf }))
    } else {
        Ok(Op::new_peek(PeekTo::StdOut))
    }
}

fn parse_upper(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Op, RpErr> {
    args.next();
    Ok(Op::new_upper())
}

fn parse_lower(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Op, RpErr> {
    args.next();
    Ok(Op::new_lower())
}

fn parse_case(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Op, RpErr> {
    args.next();
    Ok(Op::new_case())
}

fn parse_replace(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Op, RpErr> {
    args.next();
    // 被替换字符串必选，直接消耗
    if let Some(from) = parse_arg(args) {
        // 替换目标字符串必选，直接消耗
        if let Some(to) = parse_arg(args) {
            let count_opt = parse_positive_usize(args);
            let nocase = parse_tag_nocase(args, "nocase");
            Ok(Op::new_replace(from, to, count_opt, nocase))
        } else {
            Err(RpErr::MissingArg { cmd: ":replace", arg: "to" })
        }
    } else {
        Err(RpErr::MissingArg { cmd: ":replace", arg: "from" })
    }
}

fn parse_uniq(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Op, RpErr> {
    args.next();
    let nocase = parse_tag_nocase(args, "nocase");
    Ok(Op::new_uniq(nocase))
}

fn parse_join(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Op, RpErr> {
    args.next();
    let (join_info, batch) = if let Some(delimiter) = parse_opt_arg(args) {
        if let Some(prefix) = parse_opt_arg(args) {
            if let Some(postfix) = parse_opt_arg(args) {
                if let Some(size) = parse_positive_usize(args) {
                    args.next();
                    (JoinInfo { delimiter, prefix, postfix }, Some(size))
                } else {
                    (JoinInfo { delimiter, prefix, postfix }, None)
                }
            } else {
                (JoinInfo { delimiter, prefix, postfix: String::new() }, None)
            }
        } else {
            (JoinInfo { delimiter, prefix: String::new(), postfix: String::new() }, None)
        }
    } else {
        (JoinInfo::default(), None)
    };
    Ok(Op::new_join(join_info, batch))
}

fn parse_drop(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Op, RpErr> {
    args.next();
    Ok(Op::Drop(parse_cond(args, ":drop")?))
}

fn parse_keep(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Op, RpErr> {
    args.next();
    Ok(Op::Keep(parse_cond(args, ":keep")?))
}

fn parse_sort(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Op, RpErr> {
    args.next();
    let sort_by = if let Some(sort_by) = args.peek() {
        if sort_by.eq_ignore_ascii_case("num") {
            // 按照数值排序
            args.next();
            if let Some(def_integer) = parse_as::<Integer>(args) {
                SortBy::Num(Some(def_integer), None)
            } else if let Some(def_float) = parse_as::<Float>(args) {
                SortBy::Num(None, Some(def_float))
            } else {
                SortBy::Num(None, None)
            }
        } else if sort_by.eq_ignore_ascii_case("nocase") {
            args.next();
            SortBy::Text(true)
        } else if sort_by.eq_ignore_ascii_case("random") {
            args.next();
            SortBy::Random
        } else {
            SortBy::Text(false)
        }
    } else {
        SortBy::Text(false)
    };
    let desc = if sort_by != SortBy::Random
        && let Some(desc) = args.peek()
        && desc.eq_ignore_ascii_case("desc")
    {
        args.next();
        true
    } else {
        false
    };
    Ok(Op::new_sort(sort_by, desc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::args::build_args;

    #[test]
    fn test_non_match() {
        let mut args = build_args("");
        assert_eq!(Ok(None), parse_op(&mut args));
        assert_eq!(Some("".to_string()), args.next());
    }

    #[test]
    fn test_parse_peek() {
        let mut args = build_args(":uniq");
        assert_eq!(Ok(Some(Op::new_uniq(false))), parse_op(&mut args));
        assert!(args.next().is_none());

        let mut args = build_args(":uniq nocase");
        assert_eq!(Ok(Some(Op::new_uniq(true))), parse_op(&mut args));
        assert!(args.next().is_none());
    }

    #[test]
    fn test_parse_upper() {
        let mut args = build_args(":upper");
        assert_eq!(Ok(Some(Op::new_upper())), parse_op(&mut args));
        assert!(args.next().is_none());
    }

    #[test]
    fn test_parse_lower() {
        let mut args = build_args(":lower");
        assert_eq!(Ok(Some(Op::new_lower())), parse_op(&mut args));
        assert!(args.next().is_none());
    }

    #[test]
    fn test_parse_case() {
        let mut args = build_args(":case");
        assert_eq!(Ok(Some(Op::new_case())), parse_op(&mut args));
        assert!(args.next().is_none());
    }

    #[test]
    fn test_parse_replace() {
        let mut args = build_args(":replace 123 abc");
        assert_eq!(Ok(Some(Op::new_replace("123".to_string(), "abc".to_string(), None, false))), parse_op(&mut args));
        assert!(args.next().is_none());

        let mut args = build_args(":replace 123 abc 10");
        assert_eq!(
            Ok(Some(Op::new_replace("123".to_string(), "abc".to_string(), Some(10), false))),
            parse_op(&mut args)
        );
        assert!(args.next().is_none());

        let mut args = build_args(":replace 123 abc nocase");
        assert_eq!(Ok(Some(Op::new_replace("123".to_string(), "abc".to_string(), None, true))), parse_op(&mut args));
        assert!(args.next().is_none());

        let mut args = build_args(":replace 123 abc 10 nocase");
        assert_eq!(
            Ok(Some(Op::new_replace("123".to_string(), "abc".to_string(), Some(10), true))),
            parse_op(&mut args)
        );
        assert!(args.next().is_none());

        let mut args = build_args(":replace 123");
        assert_eq!(Err(RpErr::MissingArg { cmd: ":replace", arg: "to" }), parse_op(&mut args));
        assert!(args.next().is_none());

        let mut args = build_args(":replace");
        assert_eq!(Err(RpErr::MissingArg { cmd: ":replace", arg: "from" }), parse_op(&mut args));
        assert!(args.next().is_none());
    }

    #[test]
    fn test_parse_uniq() {
        let mut args = build_args(":uniq");
        assert_eq!(Ok(Some(Op::new_uniq(false))), parse_op(&mut args));
        assert!(args.next().is_none());

        let mut args = build_args(":uniq nocase");
        assert_eq!(Ok(Some(Op::new_uniq(true))), parse_op(&mut args));
        assert!(args.next().is_none());
    }

    #[test]
    fn test_parse_join() {
        let mut args = build_args(":join");
        assert_eq!(Ok(Some(Op::new_join(JoinInfo::default(), None))), parse_op(&mut args));
        assert!(args.next().is_none());

        let mut args = build_args(":join d");
        assert_eq!(
            Ok(Some(Op::new_join(
                JoinInfo { delimiter: "d".to_string(), prefix: String::new(), postfix: String::new() },
                None
            ))),
            parse_op(&mut args)
        );
        assert!(args.next().is_none());

        let mut args = build_args(":join d l");
        assert_eq!(
            Ok(Some(Op::new_join(
                JoinInfo { delimiter: "d".to_string(), prefix: "l".to_string(), postfix: String::new() },
                None
            ))),
            parse_op(&mut args)
        );
        assert!(args.next().is_none());

        let mut args = build_args(":join d l e");
        assert_eq!(
            Ok(Some(Op::new_join(
                JoinInfo { delimiter: "d".to_string(), prefix: "l".to_string(), postfix: "e".to_string() },
                None
            ))),
            parse_op(&mut args)
        );
        assert!(args.next().is_none());

        let mut args = build_args(":join d l e 10");
        assert_eq!(
            Ok(Some(Op::new_join(
                JoinInfo { delimiter: "d".to_string(), prefix: "l".to_string(), postfix: "e".to_string() },
                Some(10)
            ))),
            parse_op(&mut args)
        );
        assert!(args.next().is_none());

        let mut args = build_args(r#":join d '' "" -10"#);
        assert_eq!(
            Ok(Some(Op::new_join(
                JoinInfo { delimiter: "d".to_string(), prefix: "".to_string(), postfix: "".to_string() },
                None
            ))),
            parse_op(&mut args)
        );
        assert_eq!(Some("-10".to_string()), args.next());
    }

    #[test]
    fn test_parse_sort() {
        let mut args = build_args(":sort abc");
        assert_eq!(Ok(Some(Op::new_sort(SortBy::Text(false), false))), parse_op(&mut args));
        assert_eq!(Some("abc".to_string()), args.next());

        let mut args = build_args(":sort desc abc");
        assert_eq!(Ok(Some(Op::new_sort(SortBy::Text(false), true))), parse_op(&mut args));
        assert_eq!(Some("abc".to_string()), args.next());

        let mut args = build_args(":sort nocase abc");
        assert_eq!(Ok(Some(Op::new_sort(SortBy::Text(true), false))), parse_op(&mut args));
        assert_eq!(Some("abc".to_string()), args.next());

        let mut args = build_args(":sort nocase desc abc");
        assert_eq!(Ok(Some(Op::new_sort(SortBy::Text(true), true))), parse_op(&mut args));
        assert_eq!(Some("abc".to_string()), args.next());

        let mut args = build_args(":sort num abc");
        assert_eq!(Ok(Some(Op::new_sort(SortBy::Num(None, None), false))), parse_op(&mut args));
        assert_eq!(Some("abc".to_string()), args.next());

        let mut args = build_args(":sort num desc abc");
        assert_eq!(Ok(Some(Op::new_sort(SortBy::Num(None, None), true))), parse_op(&mut args));
        assert_eq!(Some("abc".to_string()), args.next());

        let mut args = build_args(":sort num 10 abc");
        assert_eq!(Ok(Some(Op::new_sort(SortBy::Num(Some(10), None), false))), parse_op(&mut args));
        assert_eq!(Some("abc".to_string()), args.next());

        let mut args = build_args(":sort num 10 desc abc");
        assert_eq!(Ok(Some(Op::new_sort(SortBy::Num(Some(10), None), true))), parse_op(&mut args));
        assert_eq!(Some("abc".to_string()), args.next());

        let mut args = build_args(":sort num 10.5 abc");
        assert_eq!(Ok(Some(Op::new_sort(SortBy::Num(None, Some(10.5)), false))), parse_op(&mut args));
        assert_eq!(Some("abc".to_string()), args.next());

        let mut args = build_args(":sort num 10.5 desc abc");
        assert_eq!(Ok(Some(Op::new_sort(SortBy::Num(None, Some(10.5)), true))), parse_op(&mut args));
        assert_eq!(Some("abc".to_string()), args.next());

        let mut args = build_args(":sort num -10 abc");
        assert_eq!(Ok(Some(Op::new_sort(SortBy::Num(Some(-10), None), false))), parse_op(&mut args));
        assert_eq!(Some("abc".to_string()), args.next());

        let mut args = build_args(":sort num -10 desc abc");
        assert_eq!(Ok(Some(Op::new_sort(SortBy::Num(Some(-10), None), true))), parse_op(&mut args));
        assert_eq!(Some("abc".to_string()), args.next());

        let mut args = build_args(":sort num -10.5 abc");
        assert_eq!(Ok(Some(Op::new_sort(SortBy::Num(None, Some(-10.5)), false))), parse_op(&mut args));
        assert_eq!(Some("abc".to_string()), args.next());

        let mut args = build_args(":sort num -10.5 desc abc");
        assert_eq!(Ok(Some(Op::new_sort(SortBy::Num(None, Some(-10.5)), true))), parse_op(&mut args));
        assert_eq!(Some("abc".to_string()), args.next());

        let mut args = build_args(":sort random abc");
        assert_eq!(Ok(Some(Op::new_sort(SortBy::Random, false))), parse_op(&mut args));
        assert_eq!(Some("abc".to_string()), args.next());
    }
}
