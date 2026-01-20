use crate::condition::{Condition, Select, TextSelectMode};
use crate::parse::token::{arg, arg_end, parse_num, ParserError};
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{char, space1, usize};
use nom::combinator::{map, opt, value};
use nom::error::context;
use nom::sequence::{preceded, terminated};
use nom::{IResult, Parser};

pub(in crate::parse) fn parse_cond(input: &str) -> IResult<&str, Condition, ParserError<'_>> {
    terminated(
        alt((
            context(
                "Cond::TextLenRange",
                map(
                    (
                        terminated(opt((tag_no_case("not"), space1)), (tag_no_case("len"), space1)),
                        parse_cond_range(usize),
                    ),
                    |(not_opt, (min, max))| Condition::new(Select::new_text_len_range(min, max), not_opt.is_some()),
                ),
            ),
            context(
                "Cond::TextLenSpec",
                map(
                    (
                        terminated(opt((tag_no_case("not"), space1)), (tag_no_case("len"), space1)),
                        parse_cond_spec(usize),
                    ),
                    |(not_opt, spec)| Condition::new(Select::TextLenSpec { spec }, not_opt.is_some()),
                ),
            ),
            context(
                "Cond::NumRange",
                map(
                    (
                        terminated(opt((tag_no_case("not"), space1)), (tag_no_case("num"), space1)),
                        parse_cond_range(parse_num),
                    ),
                    |(not_opt, (min, max))| Condition::new(Select::new_num_range(min, max), not_opt.is_some()),
                ),
            ),
            context(
                "Cond::NumSpec",
                map(
                    (
                        terminated(opt((tag_no_case("not"), space1)), (tag_no_case("num"), space1)),
                        parse_cond_spec(parse_num),
                    ),
                    |(not_opt, spec)| Condition::new(Select::NumSpec { spec }, not_opt.is_some()),
                ),
            ),
            context(
                "Cond::Number",
                map(
                    (
                        terminated(opt((tag_no_case("not"), space1)), tag_no_case("num")),
                        opt(preceded(space1, parse_cond_num)),
                    ),
                    |(not_opt, integer)| Condition::new(Select::Num { integer }, not_opt.is_some()),
                ),
            ),
            parse_cond_text,
            context(
                "Cond::RegMatch",
                map(
                    (terminated(opt((tag_no_case("not"), space1)), (tag_no_case("reg"), space1)), parse_cond_reg_match),
                    |(not_opt, regex)| Condition::new(regex, not_opt.is_some()),
                ),
            ),
        )),
        context("(trailing_space1)", space1),
    )
    .parse(input)
}

pub(in crate::parse) fn parse_cond_range<'a, T, F>(
    range_arg: F,
) -> impl Parser<&'a str, Output = (Option<T>, Option<T>), Error = ParserError<'a>>
where
    F: Parser<&'a str, Output = T, Error = ParserError<'a>> + Clone,
{
    map(
        (
            context("[<min>]", opt(range_arg.clone())),
            char(','),
            context("[<max>]", terminated(opt(range_arg), arg_end)),
        ),
        |(min, _, max)| (min, max),
    )
}

pub(in crate::parse) fn parse_cond_spec<'a, T, F>(
    spec_arg: F,
) -> impl Parser<&'a str, Output = T, Error = ParserError<'a>>
where
    F: Parser<&'a str, Output = T, Error = ParserError<'a>>,
{
    map(context("<spec>", terminated(spec_arg, arg_end)), |spec| spec)
}

pub(in crate::parse) fn parse_cond_num(input: &str) -> IResult<&str, bool, ParserError<'_>> {
    alt((value(true, tag_no_case("integer")), value(false, tag_no_case("float")))).parse(input)
}

pub(in crate::parse) fn parse_cond_text(input: &str) -> IResult<&str, Condition, ParserError<'_>> {
    context(
        "Cond::Text",
        map((opt((tag_no_case("not"), space1)), alt((
            value(TextSelectMode::Upper, tag_no_case("upper")),
            value(TextSelectMode::Lower, tag_no_case("lower")),
            value(TextSelectMode::Ascii, tag_no_case("ascii")),
            value(TextSelectMode::NonAscii, tag_no_case("nonascii")),
            value(TextSelectMode::Empty, tag_no_case("empty")),
            value(TextSelectMode::Blank, tag_no_case("blank")),
        ))), |(not_opt, mode)| {
            Condition::new(
                Select::Text { mode },
                not_opt.is_some(),
            )
        }),
    )
    .parse(input)
}

pub(in crate::parse) fn parse_cond_reg_match(input: &str) -> IResult<&str, Select, ParserError<'_>> {
    map(context("<exp>", arg), |regex| match Select::new_reg_match(&regex) {
        Ok(cond) => cond,
        Err(rp_err) => rp_err.termination(),
    })
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Num;

    #[test]
    fn test_parse_cond_text_len_range() {
        assert_eq!(
            parse_cond("len 1,3 "),
            Ok(("", Condition::new(Select::new_text_len_range(Some(1), Some(3)), false)))
        );
        assert_eq!(parse_cond("len ,3 "), Ok(("", Condition::new(Select::new_text_len_range(None, Some(3)), false))));
        assert_eq!(parse_cond("len 1, "), Ok(("", Condition::new(Select::new_text_len_range(Some(1), None), false))));
        assert_eq!(
            parse_cond("not len 1,3 "),
            Ok(("", Condition::new(Select::new_text_len_range(Some(1), Some(3)), true)))
        );
        assert_eq!(
            parse_cond("not len ,3 "),
            Ok(("", Condition::new(Select::new_text_len_range(None, Some(3)), true)))
        );
        assert_eq!(
            parse_cond("not len 1, "),
            Ok(("", Condition::new(Select::new_text_len_range(Some(1), None), true)))
        );
        assert_eq!(parse_cond("len , "), Ok(("", Condition::new(Select::new_text_len_range(None, None), false))));
        assert_eq!(parse_cond("not len , "), Ok(("", Condition::new(Select::new_text_len_range(None, None), true))));
        assert!(parse_cond("len 1.2,3.0 ").is_err());
    }

    #[test]
    fn test_parse_cond_text_len_spec() {
        assert_eq!(parse_cond("len 3 "), Ok(("", Condition::new(Select::TextLenSpec { spec: 3 }, false))));
        assert_eq!(parse_cond("not len 3 "), Ok(("", Condition::new(Select::TextLenSpec { spec: 3 }, true))));
    }

    #[test]
    fn test_parse_cond_num_range() {
        assert_eq!(
            parse_cond("num 1,3 "),
            Ok(("", Condition::new(Select::new_num_range(Some(Num::from(1)), Some(Num::from(3))), false)))
        );
        assert_eq!(
            parse_cond("num ,3 "),
            Ok(("", Condition::new(Select::new_num_range(None, Some(Num::from(3))), false)))
        );
        assert_eq!(
            parse_cond("num 1, "),
            Ok(("", Condition::new(Select::new_num_range(Some(Num::from(1)), None), false)))
        );
        assert_eq!(parse_cond("num , "), Ok(("", Condition::new(Select::new_num_range(None, None), false))));
        assert_eq!(
            parse_cond("not num 1,3 "),
            Ok(("", Condition::new(Select::new_num_range(Some(Num::from(1)), Some(Num::from(3))), true)))
        );
        assert_eq!(
            parse_cond("not num ,3 "),
            Ok(("", Condition::new(Select::new_num_range(None, Some(Num::from(3))), true)))
        );
        assert_eq!(
            parse_cond("not num 1, "),
            Ok(("", Condition::new(Select::new_num_range(Some(Num::from(1)), None), true)))
        );
        assert_eq!(
            parse_cond("num 1.0,3 "),
            Ok(("", Condition::new(Select::new_num_range(Some(Num::from(1.0)), Some(Num::from(3))), false)))
        );
        assert_eq!(
            parse_cond("num ,3.0 "),
            Ok(("", Condition::new(Select::new_num_range(None, Some(Num::from(3.0))), false)))
        );
        assert_eq!(
            parse_cond("num 1.1, "),
            Ok(("", Condition::new(Select::new_num_range(Some(Num::from(1.1)), None), false)))
        );
        assert_eq!(
            parse_cond("not num 1.0,3 "),
            Ok(("", Condition::new(Select::new_num_range(Some(Num::from(1.0)), Some(Num::from(3))), true)))
        );
        assert_eq!(
            parse_cond("not num ,3.0 "),
            Ok(("", Condition::new(Select::new_num_range(None, Some(Num::from(3.0))), true)))
        );
        assert_eq!(
            parse_cond("not num 1.1, "),
            Ok(("", Condition::new(Select::new_num_range(Some(Num::from(1.1)), None), true)))
        );
        assert_eq!(parse_cond("not num "), Ok(("", Condition::new(Select::Num { integer: None }, true))));
    }

    #[test]
    fn test_parse_cond_num_spec() {
        assert_eq!(parse_cond("num 3 "), Ok(("", Condition::new(Select::NumSpec { spec: Num::from(3) }, false))));
        assert_eq!(parse_cond("not num 3 "), Ok(("", Condition::new(Select::NumSpec { spec: Num::from(3) }, true))));
        assert_eq!(parse_cond("num 3.1 "), Ok(("", Condition::new(Select::NumSpec { spec: Num::from(3.1) }, false))));
        assert_eq!(
            parse_cond("not num 3.1 "),
            Ok(("", Condition::new(Select::NumSpec { spec: Num::from(3.1) }, true)))
        );
    }

    #[test]
    fn test_parse_cond_num() {
        assert_eq!(parse_cond("num "), Ok(("", Condition::new(Select::Num { integer: None }, false))));
        assert_eq!(parse_cond("num integer "), Ok(("", Condition::new(Select::Num { integer: Some(true) }, false))));
        assert_eq!(parse_cond("num float "), Ok(("", Condition::new(Select::Num { integer: Some(false) }, false))));
        assert_eq!(parse_cond("not num  "), Ok(("", Condition::new(Select::Num { integer: None }, true))));
        assert_eq!(parse_cond("not num integer "), Ok(("", Condition::new(Select::Num { integer: Some(true) }, true))));
        assert_eq!(parse_cond("not num float "), Ok(("", Condition::new(Select::Num { integer: Some(false) }, true))));
    }

    #[test]
    fn test_parse_cond_text_all_case() {
        assert_eq!(parse_cond("upper "), Ok(("", Condition::new(Select::Text { mode: TextSelectMode::Upper }, false))));
        assert_eq!(
            parse_cond("not upper "),
            Ok(("", Condition::new(Select::Text { mode: TextSelectMode::Upper }, true)))
        );
        assert_eq!(parse_cond("lower "), Ok(("", Condition::new(Select::Text { mode: TextSelectMode::Lower }, false))));
        assert_eq!(
            parse_cond("not lower "),
            Ok(("", Condition::new(Select::Text { mode: TextSelectMode::Lower }, true)))
        );
        assert!(parse_cond(" ").is_err());
    }

    #[test]
    fn test_parse_cond_ascii() {
        assert_eq!(parse_cond("ascii "), Ok(("", Condition::new(Select::Text { mode: TextSelectMode::Ascii }, false))));
        assert_eq!(
            parse_cond("not ascii "),
            Ok(("", Condition::new(Select::Text { mode: TextSelectMode::Ascii }, true)))
        );
        assert_eq!(
            parse_cond("nonascii "),
            Ok(("", Condition::new(Select::Text { mode: TextSelectMode::NonAscii }, false)))
        );
        assert_eq!(
            parse_cond("not nonascii "),
            Ok(("", Condition::new(Select::Text { mode: TextSelectMode::NonAscii }, true)))
        );
    }

    #[test]
    fn test_parse_cond_text_empty_or_blank() {
        assert_eq!(parse_cond("empty "), Ok(("", Condition::new(Select::Text { mode: TextSelectMode::Empty }, false))));
        assert_eq!(
            parse_cond("not empty "),
            Ok(("", Condition::new(Select::Text { mode: TextSelectMode::Empty }, true)))
        );
        assert_eq!(parse_cond("blank "), Ok(("", Condition::new(Select::Text { mode: TextSelectMode::Blank }, false))));
        assert_eq!(
            parse_cond("not blank "),
            Ok(("", Condition::new(Select::Text { mode: TextSelectMode::Blank }, true)))
        );
    }

    #[test]
    fn test_parse_cond_reg_match() {
        assert_eq!(
            parse_cond(r"reg '\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}' "),
            Ok(("", Condition::new(Select::new_reg_match(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap(), false)))
        );
        assert_eq!(
            parse_cond(r"not reg '\d+' "),
            Ok(("", Condition::new(Select::new_reg_match(r"\d+").unwrap(), true)))
        );
    }
}
