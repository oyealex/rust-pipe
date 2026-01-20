use crate::condition::{Condition, Select, TextSelectMode};
use crate::err::RpErr;
use crate::parse::args::parse_tag_nocase;
use crate::parse::token::parse_num;
use nom::character::complete::usize;
use nom::Parser;
use std::iter::Peekable;

pub(in crate::parse::args) fn parse_cond(
    args: &mut Peekable<impl Iterator<Item = String>>, cmd: &'static str,
) -> Result<Condition, RpErr> {
    let not = parse_tag_nocase(args, "not");
    match args.peek() {
        Some(arg) => match arg.to_ascii_lowercase().as_str() {
            "len" => {
                args.next();
                match args.next() {
                    Some(cond_range_or_spec) => {
                        if let Ok((remaining, (min, max))) =
                            crate::parse::token::condition::parse_cond_range(usize).parse(&cond_range_or_spec)
                            && remaining.is_empty()
                        {
                            Ok(Condition::new(Select::new_text_len_range(min, max), not))
                        } else if let Ok((remaining, spec)) =
                            crate::parse::token::condition::parse_cond_spec(usize).parse(&cond_range_or_spec)
                            && remaining.is_empty()
                        {
                            Ok(Condition::new(Select::TextLenSpec { spec }, not))
                        } else {
                            Err(RpErr::ArgParseErr {
                                cmd,
                                arg: "len range or spec",
                                arg_value: cond_range_or_spec,
                                error: "can not parse as range or spec arg".to_string(),
                            })
                        }
                    }
                    None => Err(RpErr::MissingArg { cmd, arg: "len range or spec" }),
                }
            }
            "num" => {
                args.next();
                match args.peek() {
                    Some(cond_range_or_spec) => {
                        let (res, should_consume_next) = if let Ok((remaining, (min, max))) =
                            crate::parse::token::condition::parse_cond_range(parse_num).parse(cond_range_or_spec)
                            && remaining.is_empty()
                        {
                            (Select::new_num_range(min, max), true)
                        } else if let Ok((remaining, spec)) =
                            crate::parse::token::condition::parse_cond_spec(parse_num).parse(cond_range_or_spec)
                            && remaining.is_empty()
                        {
                            (Select::NumSpec { spec }, true)
                        } else if let Ok((remaining, integer)) =
                            crate::parse::token::condition::parse_cond_num(cond_range_or_spec)
                            && remaining.is_empty()
                        {
                            (Select::Num { integer: Some(integer) }, true)
                        } else {
                            (Select::Num { integer: None }, false)
                        };
                        if should_consume_next {
                            args.next();
                        };
                        Ok(Condition::new(res, not))
                    }
                    None => Ok(Condition::new(Select::Num { integer: None }, not)),
                }
            }
            "reg" => {
                args.next();
                if let Some(regex) = args.next() {
                    Select::new_reg_match(&regex).map(|regex| Condition::new(regex, not))
                } else {
                    Err(RpErr::MissingArg { cmd, arg: "reg regex" })
                }
            }
            "upper" => {
                args.next();
                Ok(Condition::new(Select::Text { mode: TextSelectMode::Upper }, not))
            }
            "lower" => {
                args.next();
                Ok(Condition::new(Select::Text { mode: TextSelectMode::Lower }, not))
            }
            "ascii" => {
                args.next();
                Ok(Condition::new(Select::Text { mode: TextSelectMode::Ascii }, not))
            }
            "nonascii" => {
                args.next();
                Ok(Condition::new(Select::Text { mode: TextSelectMode::NonAscii }, not))
            }
            "empty" => {
                args.next();
                Ok(Condition::new(Select::Text { mode: TextSelectMode::Empty }, not))
            }
            "blank" => {
                args.next();
                Ok(Condition::new(Select::Text { mode: TextSelectMode::Blank }, not))
            }
            _ => Err(RpErr::MissingArg { cmd, arg: "condition" }),
        },
        None => Err(RpErr::MissingArg { cmd, arg: "condition" }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::args::build_args;
    use crate::Num;

    #[test]
    fn test_parse_cond_text_len_range() {
        assert_eq!(
            parse_cond(&mut build_args("len 1,3 "), ""),
            Ok(Condition::new(Select::new_text_len_range(Some(1), Some(3)), false))
        );
        assert_eq!(
            parse_cond(&mut build_args("len ,3 "), ""),
            Ok(Condition::new(Select::new_text_len_range(None, Some(3)), false))
        );
        assert_eq!(
            parse_cond(&mut build_args("len 1, "), ""),
            Ok(Condition::new(Select::new_text_len_range(Some(1), None), false))
        );
        assert_eq!(
            parse_cond(&mut build_args("not len 1,3 "), ""),
            Ok(Condition::new(Select::new_text_len_range(Some(1), Some(3)), true))
        );
        assert_eq!(
            parse_cond(&mut build_args("not len ,3 "), ""),
            Ok(Condition::new(Select::new_text_len_range(None, Some(3)), true))
        );
        assert_eq!(
            parse_cond(&mut build_args("not len 1, "), ""),
            Ok(Condition::new(Select::new_text_len_range(Some(1), None), true))
        );
        assert_eq!(
            parse_cond(&mut build_args("len , "), ""),
            Ok(Condition::new(Select::new_text_len_range(None, None), false))
        );
        assert_eq!(
            parse_cond(&mut build_args("not len , "), ""),
            Ok(Condition::new(Select::new_text_len_range(None, None), true))
        );
        assert!(parse_cond(&mut build_args("len 1.2,3.0 "), "").is_err());
    }

    #[test]
    fn test_parse_cond_text_len_spec() {
        assert_eq!(
            parse_cond(&mut build_args("len 3 "), ""),
            Ok(Condition::new(Select::TextLenSpec { spec: 3 }, false))
        );
        assert_eq!(
            parse_cond(&mut build_args("not len 3 "), ""),
            Ok(Condition::new(Select::TextLenSpec { spec: 3 }, true))
        );
    }

    #[test]
    fn test_parse_cond_num_range() {
        assert_eq!(
            parse_cond(&mut build_args("num 1,3 "), ""),
            Ok(Condition::new(Select::new_num_range(Some(Num::from(1)), Some(Num::from(3))), false))
        );
        assert_eq!(
            parse_cond(&mut build_args("num ,3 "), ""),
            Ok(Condition::new(Select::new_num_range(None, Some(Num::from(3))), false))
        );
        assert_eq!(
            parse_cond(&mut build_args("num 1, "), ""),
            Ok(Condition::new(Select::new_num_range(Some(Num::from(1)), None), false))
        );
        assert_eq!(
            parse_cond(&mut build_args("num , "), ""),
            Ok(Condition::new(Select::new_num_range(None, None), false))
        );
        assert_eq!(
            parse_cond(&mut build_args("not num 1,3 "), ""),
            Ok(Condition::new(Select::new_num_range(Some(Num::from(1)), Some(Num::from(3))), true))
        );
        assert_eq!(
            parse_cond(&mut build_args("not num ,3 "), ""),
            Ok(Condition::new(Select::new_num_range(None, Some(Num::from(3))), true))
        );
        assert_eq!(
            parse_cond(&mut build_args("not num 1, "), ""),
            Ok(Condition::new(Select::new_num_range(Some(Num::from(1)), None), true))
        );
        assert_eq!(
            parse_cond(&mut build_args("num 1.0,3 "), ""),
            Ok(Condition::new(Select::new_num_range(Some(Num::from(1.0)), Some(Num::from(3))), false))
        );
        assert_eq!(
            parse_cond(&mut build_args("num ,3.0 "), ""),
            Ok(Condition::new(Select::new_num_range(None, Some(Num::from(3.0))), false))
        );
        assert_eq!(
            parse_cond(&mut build_args("num 1.1, "), ""),
            Ok(Condition::new(Select::new_num_range(Some(Num::from(1.1)), None), false))
        );
        assert_eq!(
            parse_cond(&mut build_args("not num 1.0,3 "), ""),
            Ok(Condition::new(Select::new_num_range(Some(Num::from(1.0)), Some(Num::from(3))), true))
        );
        assert_eq!(
            parse_cond(&mut build_args("not num ,3.0 "), ""),
            Ok(Condition::new(Select::new_num_range(None, Some(Num::from(3.0))), true))
        );
        assert_eq!(
            parse_cond(&mut build_args("not num 1.1, "), ""),
            Ok(Condition::new(Select::new_num_range(Some(Num::from(1.1)), None), true))
        );
        assert_eq!(
            parse_cond(&mut build_args("not num , "), ""),
            Ok(Condition::new(Select::new_num_range(None, None), true))
        );
    }

    #[test]
    fn test_parse_cond_num_spec() {
        assert_eq!(
            parse_cond(&mut build_args("num 3 "), ""),
            Ok(Condition::new(Select::NumSpec { spec: Num::from(3) }, false))
        );
        assert_eq!(
            parse_cond(&mut build_args("not num 3 "), ""),
            Ok(Condition::new(Select::NumSpec { spec: Num::from(3) }, true))
        );
        assert_eq!(
            parse_cond(&mut build_args("num 3.1 "), ""),
            Ok(Condition::new(Select::NumSpec { spec: Num::from(3.1) }, false))
        );
        assert_eq!(
            parse_cond(&mut build_args("not num 3.1 "), ""),
            Ok(Condition::new(Select::NumSpec { spec: Num::from(3.1) }, true))
        );
    }

    #[test]
    fn test_parse_cond_number() {
        assert_eq!(parse_cond(&mut build_args("num "), ""), Ok(Condition::new(Select::Num { integer: None }, false)));
        assert_eq!(
            parse_cond(&mut build_args("num integer "), ""),
            Ok(Condition::new(Select::Num { integer: Some(true) }, false))
        );
        assert_eq!(
            parse_cond(&mut build_args("num float "), ""),
            Ok(Condition::new(Select::Num { integer: Some(false) }, false))
        );
        assert_eq!(
            parse_cond(&mut build_args("not num  "), ""),
            Ok(Condition::new(Select::Num { integer: None }, true))
        );
        assert_eq!(
            parse_cond(&mut build_args("not num integer "), ""),
            Ok(Condition::new(Select::Num { integer: Some(true) }, true))
        );
        assert_eq!(
            parse_cond(&mut build_args("not num float "), ""),
            Ok(Condition::new(Select::Num { integer: Some(false) }, true))
        );
    }

    #[test]
    fn test_parse_cond_text_all_case() {
        assert_eq!(
            parse_cond(&mut build_args("upper "), ""),
            Ok(Condition::new(Select::Text { mode: TextSelectMode::Upper }, false))
        );
        assert_eq!(
            parse_cond(&mut build_args("not upper "), ""),
            Ok(Condition::new(Select::Text { mode: TextSelectMode::Upper }, true))
        );
        assert_eq!(
            parse_cond(&mut build_args("lower "), ""),
            Ok(Condition::new(Select::Text { mode: TextSelectMode::Lower }, false))
        );
        assert_eq!(
            parse_cond(&mut build_args("not lower "), ""),
            Ok(Condition::new(Select::Text { mode: TextSelectMode::Lower }, true))
        );
        assert!(parse_cond(&mut build_args(" "), "").is_err());
    }

    #[test]
    fn test_parse_cond_ascii() {
        assert_eq!(parse_cond(&mut build_args("ascii "), ""), Ok(Condition::new(Select::Text { mode: TextSelectMode::Ascii }, false)));
        assert_eq!(
            parse_cond(&mut build_args("not ascii "), ""),
            Ok(Condition::new(Select::Text { mode: TextSelectMode::Ascii }, true))
        );
        assert_eq!(
            parse_cond(&mut build_args("nonascii "), ""),
            Ok(Condition::new(Select::Text { mode: TextSelectMode::NonAscii }, false))
        );
        assert_eq!(
            parse_cond(&mut build_args("not nonascii "), ""),
            Ok(Condition::new(Select::Text { mode: TextSelectMode::NonAscii }, true))
        );
    }

    #[test]
    fn test_parse_cond_text_empty_or_blank() {
        assert_eq!(
            parse_cond(&mut build_args("empty "), ""),
            Ok(Condition::new(Select::Text { mode: TextSelectMode::Empty }, false))
        );
        assert_eq!(
            parse_cond(&mut build_args("not empty "), ""),
            Ok(Condition::new(Select::Text { mode: TextSelectMode::Empty }, true))
        );
        assert_eq!(
            parse_cond(&mut build_args("blank "), ""),
            Ok(Condition::new(Select::Text { mode: TextSelectMode::Blank }, false))
        );
        assert_eq!(
            parse_cond(&mut build_args("not blank "), ""),
            Ok(Condition::new(Select::Text { mode: TextSelectMode::Blank }, true))
        );
    }

    #[test]
    fn test_parse_cond_reg_match() {
        assert_eq!(
            parse_cond(&mut build_args(r"reg '\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}' "), ""),
            Ok(Condition::new(Select::new_reg_match(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap(), false))
        );
        assert_eq!(
            parse_cond(&mut build_args(r"not reg '\d+' "), ""),
            Ok(Condition::new(Select::new_reg_match(r"\d+").unwrap(), true))
        );
    }
}
