use crate::condition::{Cond, CondRangeArg, CondSpecArg};
use crate::parse::token::{arg, arg_end, parse_2_choice, parse_num, ParserError};
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{char, space1, usize};
use nom::combinator::{map, opt, success, value, verify};
use nom::error::context;
use nom::sequence::{preceded, terminated};
use nom::{IResult, Parser};

pub(in crate::parse) fn parse_cond(input: &str) -> IResult<&str, Cond, ParserError<'_>> {
    terminated(
        alt((
            context(
                "Cond::TextLenRange",
                preceded((tag_no_case("len"), space1), map(parse_cond_range(usize), |arg| Cond::TextLenRange(arg))),
            ),
            context(
                "Cond::TextLenSpec",
                preceded((tag_no_case("len"), space1), map(parse_cond_spec(usize), |arg| Cond::TextLenSpec(arg))),
            ),
            context(
                "Cond::NumRange",
                preceded((tag_no_case("num"), space1), map(parse_cond_range(parse_num), |arg| Cond::NumRange(arg))),
            ),
            context(
                "Cond::NumSpec",
                preceded((tag_no_case("num"), space1), map(parse_cond_spec(parse_num), |arg| Cond::NumSpec(arg))),
            ),
            preceded(
                tag_no_case("num"),
                alt((preceded(space1, parse_cond_number), success(Cond::new_number(None, false)))),
            ),
            parse_cond_text_all_case,
            parse_cond_text_empty_or_blank,
            preceded((tag_no_case("reg"), space1), parse_cond_reg_match),
        )),
        context("(trailing_space1)", space1),
    )
    .parse(input)
}

pub(in crate::parse) fn parse_cond_range<'a, T, F>(
    range_arg: F,
) -> impl Parser<&'a str, Output = CondRangeArg<T>, Error = ParserError<'a>>
where
    F: Parser<&'a str, Output = T, Error = ParserError<'a>> + Clone,
{
    context(
        "CondRangeArg",
        map(
            verify(
                (
                    context("[!]", opt(char('!'))),
                    context("[<min>]", opt(range_arg.clone())),
                    char(','),
                    context("[<max>]", terminated(opt(range_arg), arg_end)),
                ),
                |(_, min, _, max)| min.is_some() || max.is_some(),
            ),
            |(not, min, _, max)| CondRangeArg::new(min, max, not.is_some()),
        ),
    )
}

pub(in crate::parse) fn parse_cond_spec<'a, T, F>(
    spec_arg: F,
) -> impl Parser<&'a str, Output = CondSpecArg<T>, Error = ParserError<'a>>
where
    F: Parser<&'a str, Output = T, Error = ParserError<'a>>,
{
    context(
        "CondSpecArg",
        map(
            (context("[!]", opt(char('!'))), char('='), context("<spec>", terminated(spec_arg, arg_end))),
            |(not, _, spec)| CondSpecArg::new(spec, not.is_some()),
        ),
    )
}

pub(in crate::parse) fn parse_cond_number(input: &str) -> IResult<&str, Cond, ParserError<'_>> {
    context(
        "Cond::Number",
        alt((
            map(
                preceded(
                    char('!'),
                    opt(alt((value(true, tag_no_case("integer")), value(false, tag_no_case("float"))))),
                ), // 必定有!
                |num_type| Cond::new_number(num_type, true),
            ),
            map(
                (opt(char('!')), alt((value(true, tag_no_case("integer")), value(false, tag_no_case("float"))))), // 必定有integer|float
                |(not, num_type)| Cond::new_number(Some(num_type), not.is_some()),
            ),
        )),
    )
    .parse(input)
}

pub(in crate::parse) fn parse_cond_text_all_case(input: &str) -> IResult<&str, Cond, ParserError<'_>> {
    context("Cond::TextAllCase", map(parse_2_choice("upper", "lower"), |is_upper| Cond::TextAllCase(is_upper)))
        .parse(input)
}

pub(in crate::parse) fn parse_cond_text_empty_or_blank(input: &str) -> IResult<&str, Cond, ParserError<'_>> {
    context(
        "Cond::TextEmptyOrBlank",
        map(parse_2_choice("empty", "blank"), |is_empty| Cond::TextEmptyOrBlank(is_empty)),
    )
    .parse(input)
}
pub(in crate::parse) fn parse_cond_reg_match(input: &str) -> IResult<&str, Cond, ParserError<'_>> {
    context(
        "Cond::RegMatch",
        map(context("<exp>", arg), |regex| match Cond::new_reg_match(&regex) {
            Ok(cond) => cond,
            Err(rp_err) => rp_err.termination(),
        }),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Num;

    #[test]
    fn test_parse_cond_text_len_range() {
        assert_eq!(parse_cond("len 1,3 "), Ok(("", Cond::TextLenRange(CondRangeArg::new(Some(1), Some(3), false)))));
        assert_eq!(parse_cond("len ,3 "), Ok(("", Cond::TextLenRange(CondRangeArg::new(None, Some(3), false)))));
        assert_eq!(parse_cond("len 1, "), Ok(("", Cond::TextLenRange(CondRangeArg::new(Some(1), None, false)))));
        assert_eq!(parse_cond("len !1,3 "), Ok(("", Cond::TextLenRange(CondRangeArg::new(Some(1), Some(3), true)))));
        assert_eq!(parse_cond("len !,3 "), Ok(("", Cond::TextLenRange(CondRangeArg::new(None, Some(3), true)))));
        assert_eq!(parse_cond("len !1, "), Ok(("", Cond::TextLenRange(CondRangeArg::new(Some(1), None, true)))));
        assert!(parse_cond("len !, ").is_err());
        assert!(parse_cond("len , ").is_err());
        assert!(parse_cond("len 1.2,3.0 ").is_err());
    }

    #[test]
    fn test_parse_cond_text_len_spec() {
        assert_eq!(parse_cond("len =3 "), Ok(("", Cond::TextLenSpec(CondSpecArg::new(3, false)))));
        assert_eq!(parse_cond("len !=3 "), Ok(("", Cond::TextLenSpec(CondSpecArg::new(3, true)))));
    }

    #[test]
    fn test_parse_cond_num_range() {
        assert_eq!(
            parse_cond("num 1,3 "),
            Ok(("", Cond::NumRange(CondRangeArg::new(Some(Num::from(1)), Some(Num::from(3)), false))))
        );
        assert_eq!(parse_cond("num ,3 "), Ok(("", Cond::NumRange(CondRangeArg::new(None, Some(Num::from(3)), false)))));
        assert_eq!(parse_cond("num 1, "), Ok(("", Cond::NumRange(CondRangeArg::new(Some(Num::from(1)), None, false)))));
        assert_eq!(
            parse_cond("num !1,3 "),
            Ok(("", Cond::NumRange(CondRangeArg::new(Some(Num::from(1)), Some(Num::from(3)), true))))
        );
        assert_eq!(parse_cond("num !,3 "), Ok(("", Cond::NumRange(CondRangeArg::new(None, Some(Num::from(3)), true)))));
        assert_eq!(parse_cond("num !1, "), Ok(("", Cond::NumRange(CondRangeArg::new(Some(Num::from(1)), None, true)))));
        assert_eq!(
            parse_cond("num 1.0,3 "),
            Ok(("", Cond::NumRange(CondRangeArg::new(Some(Num::from(1.0)), Some(Num::from(3)), false))))
        );
        assert_eq!(
            parse_cond("num ,3.0 "),
            Ok(("", Cond::NumRange(CondRangeArg::new(None, Some(Num::from(3.0)), false))))
        );
        assert_eq!(
            parse_cond("num 1.1, "),
            Ok(("", Cond::NumRange(CondRangeArg::new(Some(Num::from(1.1)), None, false))))
        );
        assert_eq!(
            parse_cond("num !1.0,3 "),
            Ok(("", Cond::NumRange(CondRangeArg::new(Some(Num::from(1.0)), Some(Num::from(3)), true))))
        );
        assert_eq!(
            parse_cond("num !,3.0 "),
            Ok(("", Cond::NumRange(CondRangeArg::new(None, Some(Num::from(3.0)), true))))
        );
        assert_eq!(
            parse_cond("num !1.1, "),
            Ok(("", Cond::NumRange(CondRangeArg::new(Some(Num::from(1.1)), None, true))))
        );
        assert!(parse_cond("num !, ").is_err());
    }

    #[test]
    fn test_parse_cond_num_spec() {
        assert_eq!(parse_cond("num =3 "), Ok(("", Cond::NumSpec(CondSpecArg::new(Num::from(3), false)))));
        assert_eq!(parse_cond("num !=3 "), Ok(("", Cond::NumSpec(CondSpecArg::new(Num::from(3), true)))));
        assert_eq!(parse_cond("num =3.1 "), Ok(("", Cond::NumSpec(CondSpecArg::new(Num::from(3.1), false)))));
        assert_eq!(parse_cond("num !=3.1 "), Ok(("", Cond::NumSpec(CondSpecArg::new(Num::from(3.1), true)))));
    }

    #[test]
    fn test_parse_cond_number() {
        assert_eq!(parse_cond("num "), Ok(("", Cond::new_number(None, false))));
        assert_eq!(parse_cond("num integer "), Ok(("", Cond::new_number(Some(true), false))));
        assert_eq!(parse_cond("num float "), Ok(("", Cond::new_number(Some(false), false))));
        assert_eq!(parse_cond("num ! "), Ok(("", Cond::new_number(None, true))));
        assert_eq!(parse_cond("num !integer "), Ok(("", Cond::new_number(Some(true), true))));
        assert_eq!(parse_cond("num !float "), Ok(("", Cond::new_number(Some(false), true))));
    }

    #[test]
    fn test_parse_cond_text_all_case() {
        assert_eq!(parse_cond("upper "), Ok(("", Cond::TextAllCase(true))));
        assert_eq!(parse_cond("lower "), Ok(("", Cond::TextAllCase(false))));
        assert!(parse_cond(" ").is_err());
    }

    #[test]
    fn test_parse_cond_text_empty_or_blank() {
        assert_eq!(parse_cond("empty "), Ok(("", Cond::TextEmptyOrBlank(true))));
        assert_eq!(parse_cond("blank "), Ok(("", Cond::TextEmptyOrBlank(false))));
        assert!(parse_cond(" ").is_err());
    }

    #[test]
    fn test_parse_cond_reg_match() {
        assert_eq!(
            parse_cond(r"reg '\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}' "),
            Ok(("", Cond::new_reg_match(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap()))
        );
    }
}
