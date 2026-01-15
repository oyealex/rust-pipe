use crate::err::RpErr;
use crate::input::Input;
use crate::parse::args::{parse_arg, parse_arg1, parse_opt_arg, parse_positive_usize};
use std::iter::Peekable;

pub(in crate::parse::args) fn parse_input(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Input, RpErr> {
    match args.peek() {
        Some(input) => {
            let lower_input = input.to_ascii_lowercase();
            match lower_input.as_str() {
                ":in" => parse_std_in(args),
                ":file" => parse_file(args),
                #[cfg(windows)]
                ":clip" => parse_clip(args),
                ":of" => parse_of(args),
                ":gen" => parse_gen(args),
                ":repeat" => parse_repeat(args),
                _ => Ok(Input::new_std_in()),
            }
        }
        None => Ok(Input::new_std_in()),
    }
}

fn parse_std_in(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Input, RpErr> {
    args.next(); // 消耗命令文本
    Ok(Input::new_std_in())
}

fn parse_file(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Input, RpErr> {
    args.next(); // 消耗命令文本
    Ok(Input::new_file(parse_arg1(args, ":file", "file_name")?))
}

#[cfg(windows)]
fn parse_clip(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Input, RpErr> {
    args.next(); // 消耗命令文本
    Ok(Input::new_clip())
}

fn parse_of(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Input, RpErr> {
    args.next(); // 消耗命令文本
    Ok(Input::new_of(parse_arg1(args, ":of", "value")?))
}

fn parse_gen(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Input, RpErr> {
    args.next(); // 消耗命令文本
    let range = args.next().ok_or_else(|| RpErr::MissingArg { cmd: ":gen", arg: "range" })?;
    match crate::parse::token::input::parse_range_in_gen(&range) {
        Ok((remaining, (start, end, step))) => {
            if !remaining.is_empty() {
                Err(RpErr::UnexpectedRemaining { cmd: ":gen", arg: "range", remaining: remaining.to_string() })
            } else {
                Ok(Input::new_gen(start, end, step, parse_opt_arg(args)))
            }
        }
        Err(e) => {
            Err(RpErr::ArgParseErr { cmd: ":gen", arg: "range", arg_value: range.to_string(), error: e.to_string() })
        }
    }
}

fn parse_repeat(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Input, RpErr> {
    args.next(); // 消耗命令文本
    let value = parse_arg(args).ok_or(RpErr::MissingArg { cmd: ":repeat", arg: "value" })?;
    let count = parse_positive_usize(args);
    Ok(Input::new_repeat(value, count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::args::build_args;
    use crate::Integer;

    #[test]
    fn test_parse_std_in() {
        let mut args = build_args(":in");
        assert_eq!(Ok(Input::new_std_in()), parse_input(&mut args));
        assert!(args.next().is_none());

        let mut args = build_args(":in 123");
        assert_eq!(Ok(Input::new_std_in()), parse_input(&mut args));
        assert_eq!(Some("123".to_string()), args.next());

        let mut args = build_args("");
        assert_eq!(Ok(Input::new_std_in()), parse_input(&mut args));
        assert_eq!(Some("".to_string()), args.next());
    }

    #[test]
    fn test_parse_file() {
        let mut args = build_args(":file name");
        assert_eq!(Ok(Input::new_file(vec!["name".to_string()])), parse_input(&mut args));
        assert!(args.next().is_none());

        let mut args = build_args(":file name1 name2 \\] :123");
        assert_eq!(
            Ok(Input::new_file(vec!["name1".to_string(), "name2".to_string(), "\\]".to_string()])),
            parse_input(&mut args)
        );
        assert_eq!(Some(":123".to_string()), args.next());

        let mut args = build_args(":file");
        assert_eq!(Err(RpErr::MissingArg { cmd: ":file", arg: "file_name" }), parse_input(&mut args));
        assert!(args.next().is_none());

        let mut args = build_args(":file [ ]");
        assert_eq!(Ok(Input::new_file(vec!["[".to_string(), "]".to_string()])), parse_input(&mut args));
        assert!(args.next().is_none());
    }

    #[test]
    #[cfg(windows)]
    fn test_parse_clip() {
        let mut args = build_args(":clip");
        assert_eq!(Ok(Input::new_clip()), parse_input(&mut args));
        assert!(args.next().is_none());
    }

    #[test]
    fn test_parse_of() {
        let mut args = build_args(":of text");
        assert_eq!(Ok(Input::new_of(vec!["text".to_string()])), parse_input(&mut args));
        assert!(args.next().is_none());

        let mut args = build_args(":of text1 text2 \\] :123");
        assert_eq!(
            Ok(Input::new_of(vec!["text1".to_string(), "text2".to_string(), "\\]".to_string()])),
            parse_input(&mut args)
        );
        assert_eq!(Some(":123".to_string()), args.next());

        let mut args = build_args(":of");
        assert_eq!(Err(RpErr::MissingArg { cmd: ":of", arg: "value" }), parse_input(&mut args));
        assert!(args.next().is_none());

        let mut args = build_args(":of [ ]");
        assert_eq!(Ok(Input::new_of(vec!["[".to_string(), "]".to_string()])), parse_input(&mut args));
        assert!(args.next().is_none());
    }

    #[test]
    fn test_parse_gen() {
        let mut args = build_args(":gen 0");
        assert_eq!(Ok(Input::new_gen(0, Integer::MAX, 1, None)), parse_input(&mut args));
        assert!(args.next().is_none());

        let mut args = build_args(":gen 0,10");
        assert_eq!(Ok(Input::new_gen(0, 10, 1, None)), parse_input(&mut args));
        assert!(args.next().is_none());

        let mut args = build_args(":gen 0,10,2");
        assert_eq!(Ok(Input::new_gen(0, 10, 2, None)), parse_input(&mut args));
        assert!(args.next().is_none());

        let mut args = build_args(":gen 0,,2");
        assert_eq!(Ok(Input::new_gen(0, Integer::MAX, 2, None)), parse_input(&mut args));
        assert!(args.next().is_none());

        let mut args = build_args(":gen");
        assert_eq!(Err(RpErr::MissingArg { cmd: ":gen", arg: "range" }), parse_input(&mut args));
        assert!(args.next().is_none());

        let mut args = build_args(":gen 0,10,2abc");
        assert_eq!(
            Err(RpErr::UnexpectedRemaining { cmd: ":gen", arg: "range", remaining: "abc".to_string() }),
            parse_input(&mut args)
        );
        assert!(args.next().is_none());

        let mut args = build_args(":gen abc");
        assert!(if let Err(err) = parse_input(&mut args) {
            match err {
                RpErr::ArgParseErr { cmd, arg, arg_value, .. } => {
                    ":gen".eq(cmd) && "range".eq(arg) && "abc".eq(&arg_value)
                }
                _ => false,
            }
        } else {
            false
        });
        assert!(args.next().is_none());
    }

    #[test]
    fn test_parse_repeat() {
        let mut args = build_args(":repeat 123");
        assert_eq!(Ok(Input::new_repeat("123".to_string(), None)), parse_input(&mut args));
        assert!(args.next().is_none());

        let mut args = build_args(":repeat 123 10");
        assert_eq!(Ok(Input::new_repeat("123".to_string(), Some(10))), parse_input(&mut args));
        assert!(args.next().is_none());

        let mut args = build_args(":repeat");
        assert_eq!(Err(RpErr::MissingArg { cmd: ":repeat", arg: "value" }), parse_input(&mut args));
        assert!(args.next().is_none());
    }
}
