use crate::condition::Cond;
use crate::err::RpErr;
use crate::parse::token::parse_num;
use nom::character::complete::usize;
use nom::Parser;
use std::iter::Peekable;

pub(in crate::parse::args) fn parse_cond(
    args: &mut Peekable<impl Iterator<Item = String>>, cmd: &'static str,
) -> Result<Cond, RpErr> {
    match args.peek() {
        Some(arg) => {
            let lower_arg = arg.to_ascii_lowercase();
            match lower_arg.as_str() {
                "len" => {
                    args.next();
                    match args.next() {
                        Some(cond_range_or_spec) => {
                            if let Ok((remaining, cond_range_arg)) =
                                crate::parse::token::condition::parse_cond_range(usize).parse(&cond_range_or_spec)
                                && remaining.is_empty()
                            {
                                Ok(Cond::TextLenRange(cond_range_arg))
                            } else if let Ok((remaining, cond_spec_arg)) =
                                crate::parse::token::condition::parse_cond_spec(usize).parse(&cond_range_or_spec)
                                && remaining.is_empty()
                            {
                                Ok(Cond::TextLenSpec(cond_spec_arg))
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
                            let (res, should_consume_next) = if let Ok((remaining, cond_range_arg)) =
                                crate::parse::token::condition::parse_cond_range(parse_num).parse(cond_range_or_spec)
                                && remaining.is_empty()
                            {
                                (Cond::NumRange(cond_range_arg), true)
                            } else if let Ok((remaining, cond_spec_arg)) =
                                crate::parse::token::condition::parse_cond_spec(parse_num).parse(cond_range_or_spec)
                                && remaining.is_empty()
                            {
                                (Cond::NumSpec(cond_spec_arg), true)
                            } else if let Ok((remaining, cond_number_arg)) =
                                crate::parse::token::condition::parse_cond_number(cond_range_or_spec)
                                && remaining.is_empty()
                            {
                                (cond_number_arg, true)
                            } else {
                                (Cond::new_number(None, false), false)
                            };
                            if should_consume_next {
                                args.next();
                            };
                            Ok(res)
                        }
                        None => Ok(Cond::new_number(None, false)),
                    }
                }
                "reg" => {
                    args.next();
                    if let Some(regex) = args.next() {
                        Cond::new_reg_match(&regex)
                    } else {
                        Err(RpErr::MissingArg { cmd, arg: "reg regex" })
                    }
                }
                "upper" => {
                    args.next();
                    Ok(Cond::TextAllCase(true))
                }
                "lower" => {
                    args.next();
                    Ok(Cond::TextAllCase(false))
                }
                "empty" => {
                    args.next();
                    Ok(Cond::TextEmptyOrBlank(true))
                }
                "blank" => {
                    args.next();
                    Ok(Cond::TextEmptyOrBlank(false))
                }
                _ => Err(RpErr::MissingArg { cmd, arg: "condition" }),
            }
        }
        None => Err(RpErr::MissingArg { cmd, arg: "condition" }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::{CondRangeArg, CondSpecArg};
    use crate::parse::args::build_args;
    use crate::Num;

    #[test]
    fn test_parse_cond_text_len_range() {
        assert_eq!(
            Ok(Cond::TextLenRange(CondRangeArg::new(Some(1), Some(3), false))),
            parse_cond(&mut build_args("len 1,3 "), "")
        );
        assert_eq!(
            Ok(Cond::TextLenRange(CondRangeArg::new(None, Some(3), false))),
            parse_cond(&mut build_args("len ,3 "), "")
        );
        assert_eq!(
            Ok(Cond::TextLenRange(CondRangeArg::new(Some(1), None, false))),
            parse_cond(&mut build_args("len 1, "), "")
        );
        assert_eq!(
            Ok(Cond::TextLenRange(CondRangeArg::new(Some(1), Some(3), true))),
            parse_cond(&mut build_args("len !1,3 "), "")
        );
        assert_eq!(
            Ok(Cond::TextLenRange(CondRangeArg::new(None, Some(3), true))),
            parse_cond(&mut build_args("len !,3 "), "")
        );
        assert_eq!(
            Ok(Cond::TextLenRange(CondRangeArg::new(Some(1), None, true))),
            parse_cond(&mut build_args("len !1, "), "")
        );
        assert!(parse_cond(&mut build_args("len !, "), "").is_err());
        assert!(parse_cond(&mut build_args("len , "), "").is_err());
        assert!(parse_cond(&mut build_args("len 1.2,3.0 "), "").is_err());
    }

    #[test]
    fn test_parse_cond_text_len_spec() {
        assert_eq!(Ok(Cond::TextLenSpec(CondSpecArg::new(3, false))), parse_cond(&mut build_args("len =3 "), ""));
        assert_eq!(Ok(Cond::TextLenSpec(CondSpecArg::new(3, true))), parse_cond(&mut build_args("len !=3 "), ""));
    }

    #[test]
    fn test_parse_cond_num_range() {
        assert_eq!(
            Ok(Cond::NumRange(CondRangeArg::new(Some(Num::from(1)), Some(Num::from(3)), false))),
            parse_cond(&mut build_args("num 1,3 "), "")
        );
        assert_eq!(
            Ok(Cond::NumRange(CondRangeArg::new(None, Some(Num::from(3)), false))),
            parse_cond(&mut build_args("num ,3 "), "")
        );
        assert_eq!(
            Ok(Cond::NumRange(CondRangeArg::new(Some(Num::from(1)), None, false))),
            parse_cond(&mut build_args("num 1, "), "")
        );
        assert_eq!(
            Ok(Cond::NumRange(CondRangeArg::new(Some(Num::from(1)), Some(Num::from(3)), true))),
            parse_cond(&mut build_args("num !1,3 "), "")
        );
        assert_eq!(
            Ok(Cond::NumRange(CondRangeArg::new(None, Some(Num::from(3)), true))),
            parse_cond(&mut build_args("num !,3 "), "")
        );
        assert_eq!(
            Ok(Cond::NumRange(CondRangeArg::new(Some(Num::from(1)), None, true))),
            parse_cond(&mut build_args("num !1, "), "")
        );
        assert_eq!(
            Ok(Cond::NumRange(CondRangeArg::new(Some(Num::from(1.0)), Some(Num::from(3)), false))),
            parse_cond(&mut build_args("num 1.0,3 "), "")
        );
        assert_eq!(
            Ok(Cond::NumRange(CondRangeArg::new(None, Some(Num::from(3.0)), false))),
            parse_cond(&mut build_args("num ,3.0 "), "")
        );
        assert_eq!(
            Ok(Cond::NumRange(CondRangeArg::new(Some(Num::from(1.1)), None, false))),
            parse_cond(&mut build_args("num 1.1, "), "")
        );
        assert_eq!(
            Ok(Cond::NumRange(CondRangeArg::new(Some(Num::from(1.0)), Some(Num::from(3)), true))),
            parse_cond(&mut build_args("num !1.0,3 "), "")
        );
        assert_eq!(
            Ok(Cond::NumRange(CondRangeArg::new(None, Some(Num::from(3.0)), true))),
            parse_cond(&mut build_args("num !,3.0 "), "")
        );
        assert_eq!(
            Ok(Cond::NumRange(CondRangeArg::new(Some(Num::from(1.1)), None, true))),
            parse_cond(&mut build_args("num !1.1, "), "")
        );
        let mut args = build_args("num !, ");
        assert_eq!(Ok(Cond::new_number(None, false)), parse_cond(&mut args, ""));
        assert_eq!(Some("!,".to_string()), args.next());
    }

    #[test]
    fn test_parse_cond_num_spec() {
        assert_eq!(
            Ok(Cond::NumSpec(CondSpecArg::new(Num::from(3), false))),
            parse_cond(&mut build_args("num =3 "), "")
        );
        assert_eq!(
            Ok(Cond::NumSpec(CondSpecArg::new(Num::from(3), true))),
            parse_cond(&mut build_args("num !=3 "), "")
        );
        assert_eq!(
            Ok(Cond::NumSpec(CondSpecArg::new(Num::from(3.1), false))),
            parse_cond(&mut build_args("num =3.1 "), "")
        );
        assert_eq!(
            Ok(Cond::NumSpec(CondSpecArg::new(Num::from(3.1), true))),
            parse_cond(&mut build_args("num !=3.1 "), "")
        );
    }

    #[test]
    fn test_parse_cond_number() {
        assert_eq!(Ok(Cond::new_number(None, false)), parse_cond(&mut build_args("num "), ""));
        assert_eq!(Ok(Cond::new_number(Some(true), false)), parse_cond(&mut build_args("num integer "), ""));
        assert_eq!(Ok(Cond::new_number(Some(false), false)), parse_cond(&mut build_args("num float "), ""));
        assert_eq!(Ok(Cond::new_number(None, true)), parse_cond(&mut build_args("num ! "), ""));
        assert_eq!(Ok(Cond::new_number(Some(true), true)), parse_cond(&mut build_args("num !integer "), ""));
        assert_eq!(Ok(Cond::new_number(Some(false), true)), parse_cond(&mut build_args("num !float "), ""));
    }

    #[test]
    fn test_parse_cond_text_all_case() {
        assert_eq!(Ok(Cond::TextAllCase(true)), parse_cond(&mut build_args("upper "), ""));
        assert_eq!(Ok(Cond::TextAllCase(false)), parse_cond(&mut build_args("lower "), ""));
        assert!(parse_cond(&mut build_args(" "), "").is_err());
    }

    #[test]
    fn test_parse_cond_text_empty_or_blank() {
        assert_eq!(Ok(Cond::TextEmptyOrBlank(true)), parse_cond(&mut build_args("empty "), ""));
        assert_eq!(Ok(Cond::TextEmptyOrBlank(false)), parse_cond(&mut build_args("blank "), ""));
        assert!(parse_cond(&mut build_args(" "), "").is_err());
    }

    #[test]
    fn test_parse_cond_reg_match() {
        assert_eq!(
            Ok(Cond::new_reg_match(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap()),
            parse_cond(&mut build_args(r"reg '\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}' "), "")
        );
        assert!(parse_cond(&mut build_args(r"reg '\d{1,' "), "").is_err());
    }
}
