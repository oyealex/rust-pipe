use crate::{Float, Integer};
use rt_format::{Format, FormatArgument, NoPositionalArguments, Specifier};
use std::fmt::Formatter;

#[derive(Debug, PartialEq)]
pub(crate) enum FmtArg {
    String(String),
    Integer(Integer),
    Float(Float),
}

impl From<&str> for FmtArg {
    fn from(value: &str) -> Self {
        FmtArg::String(value.to_string())
    }
}

impl From<Integer> for FmtArg {
    fn from(value: Integer) -> Self {
        FmtArg::Integer(value)
    }
}

impl From<Float> for FmtArg {
    fn from(value: Float) -> Self {
        FmtArg::Float(value)
    }
}

impl FormatArgument for FmtArg {
    fn supports_format(&self, specifier: &Specifier) -> bool {
        match self {
            Self::String(_) => match specifier.format {
                Format::Display | Format::Debug => true,
                _ => false,
            },
            Self::Integer(_) => true,
            Self::Float(_) => match specifier.format {
                Format::Display | Format::Debug | Format::LowerExp | Format::UpperExp => true,
                _ => false,
            },
        }
    }

    fn fmt_display(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            FmtArg::String(string) => std::fmt::Display::fmt(&string, f),
            FmtArg::Integer(integer) => std::fmt::Display::fmt(&integer, f),
            FmtArg::Float(float) => std::fmt::Display::fmt(&float, f),
        }
    }

    fn fmt_debug(&self, f: &mut Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }

    fn fmt_octal(&self, f: &mut Formatter) -> std::fmt::Result {
        if let FmtArg::Integer(integer) = self { std::fmt::Octal::fmt(&integer, f) } else { Err(std::fmt::Error) }
    }

    fn fmt_lower_hex(&self, f: &mut Formatter) -> std::fmt::Result {
        if let FmtArg::Integer(integer) = self { std::fmt::LowerHex::fmt(&integer, f) } else { Err(std::fmt::Error) }
    }

    fn fmt_upper_hex(&self, f: &mut Formatter) -> std::fmt::Result {
        if let FmtArg::Integer(integer) = self { std::fmt::UpperHex::fmt(&integer, f) } else { Err(std::fmt::Error) }
    }

    fn fmt_binary(&self, f: &mut Formatter) -> std::fmt::Result {
        if let FmtArg::Integer(integer) = self { std::fmt::Binary::fmt(&integer, f) } else { Err(std::fmt::Error) }
    }

    fn fmt_lower_exp(&self, f: &mut Formatter) -> std::fmt::Result {
        if let FmtArg::Integer(integer) = self { std::fmt::LowerExp::fmt(&integer, f) } else { Err(std::fmt::Error) }
    }

    fn fmt_upper_exp(&self, f: &mut Formatter) -> std::fmt::Result {
        if let FmtArg::Integer(integer) = self { std::fmt::UpperExp::fmt(&integer, f) } else { Err(std::fmt::Error) }
    }
}

impl NamedArguments<FmtArg> for &[(&str, FmtArg)] {
    fn get(&self, key: &str) -> Option<&FmtArg> {
        for (k, v) in *self {
            if *k == key {
                return Some(v);
            }
        }
        None
    }
}

use crate::err::RpErr;
use rt_format::argument::NamedArguments;
use rt_format::ParsedFormat;

pub(crate) fn fmt_args(fmt: &str, args: &[(&str, FmtArg)]) -> Result<String, RpErr> {
    match ParsedFormat::parse(fmt, &NoPositionalArguments, &args) {
        Ok(string) => Ok(format!("{}", string)),
        Err(err_pos) => Err(RpErr::FormatStringErr { fmt: format!("{fmt:?}"), value: format!("{args:?}"), err_pos }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_args() {
        assert_eq!(
            Ok(format!("{:<7} is {:>7} year's old", "Jack", 12)),
            fmt_args(
                "{name:<7} is {age:>7} year's old",
                &vec![("name", FmtArg::from("Jack")), ("age", FmtArg::from(12))]
            )
        );
        assert_eq!(Ok("".to_string()), fmt_args("", &vec![("name", FmtArg::from("Jack")), ("age", FmtArg::from(12))]));
        assert_eq!(
            Ok("{Jack}".to_string()),
            fmt_args("{{{name}}}", &vec![("name", FmtArg::from("Jack")), ("age", FmtArg::from(12))])
        );
    }
}
