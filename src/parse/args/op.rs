use crate::err::RpErr;
use crate::op::trim::{TrimArg, TrimMode};
use crate::op::{CaseArg, JoinInfo, Op, PeekArg, SortBy, TakeDropMode};
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
                ":lower" => Some(parse_case(CaseArg::Lower, args)?),
                ":upper" => Some(parse_case(CaseArg::Upper, args)?),
                ":case" => Some(parse_case(CaseArg::Switch, args)?),
                ":replace" => Some(parse_replace(args)?),
                ":trim" => Some(parse_trim(TrimMode::All, false, args)?),
                ":ltrim" => Some(parse_trim(TrimMode::Left, false, args)?),
                ":rtrim" => Some(parse_trim(TrimMode::Right, false, args)?),
                ":trimc" => Some(parse_trim(TrimMode::All, true, args)?),
                ":ltrimc" => Some(parse_trim(TrimMode::Left, true, args)?),
                ":rtrimc" => Some(parse_trim(TrimMode::Right, true, args)?),
                ":uniq" => Some(parse_uniq(args)?),
                ":join" => Some(parse_join(args)?),
                ":drop" => Some(parse_drop_or_drop_while(args)?),
                ":take" => Some(parse_take_or_take_while(args)?),
                ":count" => Some(parse_count(args)?),
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
        Ok(Op::Peek(PeekArg::File { file, append, crlf }))
    } else {
        Ok(Op::Peek(PeekArg::StdOut))
    }
}

fn parse_case(case_arg: CaseArg, args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Op, RpErr> {
    args.next();
    Ok(Op::Case(case_arg))
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

fn parse_trim(
    trim_mode: TrimMode, char_mode: bool, args: &mut Peekable<impl Iterator<Item = String>>,
) -> Result<Op, RpErr> {
    args.next();
    let pattern = parse_opt_arg(args);
    let nocase = if pattern.is_some() { parse_tag_nocase(args, "nocase") } else { false };
    Ok(Op::Trim(TrimArg::new(trim_mode, pattern, char_mode, nocase)))
}

fn parse_uniq(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Op, RpErr> {
    args.next();
    let nocase = parse_tag_nocase(args, "nocase");
    Ok(Op::Uniq(nocase))
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

fn parse_drop_or_drop_while(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Op, RpErr> {
    args.next();
    if let Some(maybe_while) = args.peek()
        && maybe_while.eq_ignore_ascii_case("while")
    {
        args.next();
        Ok(Op::new_take_drop(TakeDropMode::DropWhile, parse_cond(args, ":drop while")?))
    } else {
        Ok(Op::new_take_drop(TakeDropMode::Drop, parse_cond(args, ":drop")?))
    }
}

fn parse_take_or_take_while(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Op, RpErr> {
    args.next();
    if let Some(maybe_while) = args.peek()
        && maybe_while.eq_ignore_ascii_case("while")
    {
        args.next();
        Ok(Op::new_take_drop(TakeDropMode::TakeWhile, parse_cond(args, ":take while")?))
    } else {
        Ok(Op::new_take_drop(TakeDropMode::Take, parse_cond(args, ":take")?))
    }
}

fn parse_count(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Op, RpErr> {
    args.next();
    Ok(Op::Count)
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
    use crate::op::CaseArg;
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
        assert_eq!(Ok(Some(Op::Uniq(false))), parse_op(&mut args));
        assert!(args.next().is_none());

        let mut args = build_args(":uniq nocase");
        assert_eq!(Ok(Some(Op::Uniq(true))), parse_op(&mut args));
        assert!(args.next().is_none());
    }

    #[test]
    fn test_parse_case() {
        let mut args = build_args(":lower");
        assert_eq!(Ok(Some(Op::Case(CaseArg::Lower))), parse_op(&mut args));
        assert!(args.next().is_none());
        let mut args = build_args(":upper");
        assert_eq!(Ok(Some(Op::Case(CaseArg::Upper))), parse_op(&mut args));
        assert!(args.next().is_none());
        let mut args = build_args(":case");
        assert_eq!(Ok(Some(Op::Case(CaseArg::Switch))), parse_op(&mut args));
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
    fn test_parse_trim() {
        // trim
        assert_eq!(
            Ok(Some(Op::Trim(TrimArg::new(TrimMode::All, None, false, false)))),
            parse_op(&mut build_args(":trim"))
        );
        assert_eq!(
            Ok(Some(Op::Trim(TrimArg::new(TrimMode::All, Some("abc".to_string()), false, false)))),
            parse_op(&mut build_args(":trim abc"))
        );
        assert_eq!(
            Ok(Some(Op::Trim(TrimArg::new(TrimMode::All, Some("abc".to_string()), false, true)))),
            parse_op(&mut build_args(":trim abc nocase"))
        );
        let mut args = build_args(":trim :abc");
        assert_eq!(Ok(Some(Op::Trim(TrimArg::new(TrimMode::All, None, false, false)))), parse_op(&mut args));
        assert_eq!(vec![":abc"], args.collect::<Vec<_>>());
        assert_eq!(
            Ok(Some(Op::Trim(TrimArg::new(TrimMode::All, Some(":abc".to_string()), false, false)))),
            parse_op(&mut build_args(":trim \\:abc"))
        );
        // ltrim
        assert_eq!(
            Ok(Some(Op::Trim(TrimArg::new(TrimMode::Left, None, false, false)))),
            parse_op(&mut build_args(":ltrim"))
        );
        assert_eq!(
            Ok(Some(Op::Trim(TrimArg::new(TrimMode::Left, Some("abc".to_string()), false, false)))),
            parse_op(&mut build_args(":ltrim abc"))
        );
        assert_eq!(
            Ok(Some(Op::Trim(TrimArg::new(TrimMode::Left, Some("abc".to_string()), false, true)))),
            parse_op(&mut build_args(":ltrim abc nocase"))
        );
        let mut args = build_args(":ltrim :abc");
        assert_eq!(Ok(Some(Op::Trim(TrimArg::new(TrimMode::Left, None, false, false)))), parse_op(&mut args));
        assert_eq!(vec![":abc"], args.collect::<Vec<_>>());
        assert_eq!(
            Ok(Some(Op::Trim(TrimArg::new(TrimMode::Left, Some(":abc".to_string()), false, false)))),
            parse_op(&mut build_args(":ltrim \\:abc"))
        );
        // rtrim
        assert_eq!(
            Ok(Some(Op::Trim(TrimArg::new(TrimMode::Right, None, false, false)))),
            parse_op(&mut build_args(":rtrim"))
        );
        assert_eq!(
            Ok(Some(Op::Trim(TrimArg::new(TrimMode::Right, Some("abc".to_string()), false, false)))),
            parse_op(&mut build_args(":rtrim abc"))
        );
        assert_eq!(
            Ok(Some(Op::Trim(TrimArg::new(TrimMode::Right, Some("abc".to_string()), false, true)))),
            parse_op(&mut build_args(":rtrim abc nocase"))
        );
        let mut args = build_args(":rtrim :abc");
        assert_eq!(Ok(Some(Op::Trim(TrimArg::new(TrimMode::Right, None, false, false)))), parse_op(&mut args));
        assert_eq!(vec![":abc"], args.collect::<Vec<_>>());
        assert_eq!(
            Ok(Some(Op::Trim(TrimArg::new(TrimMode::Right, Some(":abc".to_string()), false, false)))),
            parse_op(&mut build_args(":rtrim \\:abc"))
        );
        // trimc
        assert_eq!(
            Ok(Some(Op::Trim(TrimArg::new(TrimMode::All, None, true, false)))),
            parse_op(&mut build_args(":trimc"))
        );
        assert_eq!(
            Ok(Some(Op::Trim(TrimArg::new(TrimMode::All, Some("abc".to_string()), true, false)))),
            parse_op(&mut build_args(":trimc abc"))
        );
        assert_eq!(
            Ok(Some(Op::Trim(TrimArg::new(TrimMode::All, Some("abc".to_string()), true, true)))),
            parse_op(&mut build_args(":trimc abc nocase"))
        );
        let mut args = build_args(":trimc :abc");
        assert_eq!(Ok(Some(Op::Trim(TrimArg::new(TrimMode::All, None, true, false)))), parse_op(&mut args));
        assert_eq!(vec![":abc"], args.collect::<Vec<_>>());
        assert_eq!(
            Ok(Some(Op::Trim(TrimArg::new(TrimMode::All, Some(":abc".to_string()), true, false)))),
            parse_op(&mut build_args(":trimc \\:abc"))
        );
        // ltrimc
        assert_eq!(
            Ok(Some(Op::Trim(TrimArg::new(TrimMode::Left, None, true, false)))),
            parse_op(&mut build_args(":ltrimc"))
        );
        assert_eq!(
            Ok(Some(Op::Trim(TrimArg::new(TrimMode::Left, Some("abc".to_string()), true, false)))),
            parse_op(&mut build_args(":ltrimc abc"))
        );
        assert_eq!(
            Ok(Some(Op::Trim(TrimArg::new(TrimMode::Left, Some("abc".to_string()), true, true)))),
            parse_op(&mut build_args(":ltrimc abc nocase"))
        );
        let mut args = build_args(":ltrimc :abc");
        assert_eq!(Ok(Some(Op::Trim(TrimArg::new(TrimMode::Left, None, true, false)))), parse_op(&mut args));
        assert_eq!(vec![":abc"], args.collect::<Vec<_>>());
        assert_eq!(
            Ok(Some(Op::Trim(TrimArg::new(TrimMode::Left, Some(":abc".to_string()), true, false)))),
            parse_op(&mut build_args(":ltrimc \\:abc"))
        );
        // rtrimc
        assert_eq!(
            Ok(Some(Op::Trim(TrimArg::new(TrimMode::Right, None, true, false)))),
            parse_op(&mut build_args(":rtrimc"))
        );
        assert_eq!(
            Ok(Some(Op::Trim(TrimArg::new(TrimMode::Right, Some("abc".to_string()), true, false)))),
            parse_op(&mut build_args(":rtrimc abc"))
        );
        assert_eq!(
            Ok(Some(Op::Trim(TrimArg::new(TrimMode::Right, Some("abc".to_string()), true, true)))),
            parse_op(&mut build_args(":rtrimc abc nocase"))
        );
        let mut args = build_args(":rtrimc :abc");
        assert_eq!(Ok(Some(Op::Trim(TrimArg::new(TrimMode::Right, None, true, false)))), parse_op(&mut args));
        assert_eq!(vec![":abc"], args.collect::<Vec<_>>());
        assert_eq!(
            Ok(Some(Op::Trim(TrimArg::new(TrimMode::Right, Some(":abc".to_string()), true, false)))),
            parse_op(&mut build_args(":rtrimc \\:abc"))
        );
    }

    #[test]
    fn test_parse_uniq() {
        let mut args = build_args(":uniq");
        assert_eq!(Ok(Some(Op::Uniq(false))), parse_op(&mut args));
        assert!(args.next().is_none());

        let mut args = build_args(":uniq nocase");
        assert_eq!(Ok(Some(Op::Uniq(true))), parse_op(&mut args));
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
