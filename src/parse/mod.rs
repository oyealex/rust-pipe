use crate::condition::Condition;
use crate::config::Config;
use crate::err::RpErr;
use crate::input::Input;
use crate::op::Op;
use crate::output::Output;
use nom::error::{ContextError, ErrorKind, FromExternalError, ParseError};
use nom::IResult;
use nom_language::error::{VerboseError, VerboseErrorKind};

pub(crate) mod args;
pub(crate) mod token;

type ConfigOptResult = Result<Option<Config>, RpErr>;
type ConfigsResult = Result<Vec<Config>, RpErr>;
type InputResult = Result<Input, RpErr>;
type OpResult = Result<Op, RpErr>;
type OpOptResult = Result<Option<Op>, RpErr>;
type OpsResult = Result<Vec<Op>, RpErr>;
type OutputResult = Result<Output, RpErr>;
type CondResult = Result<Condition, RpErr>;
type OpsIResult<'a> = IResult<&'a str, Vec<Op>, RpParseErr<'a>>;
type OpIResult<'a> = IResult<&'a str, Op, RpParseErr<'a>>;

/// 解析错误的类型
#[derive(Debug, Clone, PartialEq)]
pub(in crate::parse) enum RpParseErr<'a> {
    Nom(VerboseError<&'a str>),
    Rp((&'a str, Option<ErrorKind>, RpErr)),
}

impl<'a> ParseError<&'a str> for RpParseErr<'a> {
    fn from_error_kind(input: &'a str, kind: ErrorKind) -> Self {
        RpParseErr::Nom(ParseError::from_error_kind(input, kind))
    }

    fn append(input: &'a str, kind: ErrorKind, mut other: Self) -> Self {
        match other {
            RpParseErr::Nom(ref mut err) => {
                err.errors.push((input, VerboseErrorKind::Nom(kind)));
                other
            }
            RpParseErr::Rp(_) => other,
        }
    }
}

impl<'a> ContextError<&'a str> for RpParseErr<'a> {
    fn add_context(input: &'a str, ctx: &'static str, mut other: Self) -> Self {
        if let RpParseErr::Nom(err) = &mut other {
            err.errors.push((input, VerboseErrorKind::Context(ctx)))
        }
        other
    }
}

impl<'a> FromExternalError<&'a str, RpErr> for RpParseErr<'a> {
    fn from_external_error(input: &'a str, kind: ErrorKind, e: RpErr) -> Self {
        RpParseErr::Rp((input, Some(kind), e))
    }
}

impl From<nom::Err<RpParseErr<'_>>> for RpErr {
    fn from(err: nom::Err<RpParseErr<'_>>) -> Self {
        match err {
            nom::Err::Error(err) | nom::Err::Failure(err) => match err {
                RpParseErr::Nom(err) => RpErr::ParseTokenErr(format!("{err:?}")),
                RpParseErr::Rp((_, _, err)) => err,
            },
            err => RpErr::ParseTokenErr(format!("{err:?}")),
        }
    }
}
