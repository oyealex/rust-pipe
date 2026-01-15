use crate::config::Config;
use crate::parse::token::ParserError;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::space1;
use nom::combinator::map;
use nom::error::context;
use nom::multi::many0;
use nom::sequence::terminated;
use nom::{IResult, Parser};

pub(crate) fn parse_configs(input: &str) -> IResult<&str, Vec<Config>, ParserError<'_>> {
    many0(parse_config).parse(input)
}

fn parse_config(input: &str) -> IResult<&str, Config, ParserError<'_>> {
    context(
        "Config",
        terminated(
            alt((
                map(alt((tag("-h"), tag("--help"))), |_| Config::Help),
                map(alt((tag("-V"), tag("--version"))), |_| Config::Version),
                map(alt((tag("-v"), tag("--verbose"))), |_| Config::Verbose),
                map(alt((tag("-d"), tag("--dry-run"))), |_| Config::DryRun),
                map(alt((tag("-n"), tag("--nocase"))), |_| Config::Nocase),
            )),
            space1,
        ),
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        assert_eq!(parse_config("-h "), Ok(("", Config::Help)));
        assert_eq!(parse_config("--help "), Ok(("", Config::Help)));
        assert_eq!(parse_config("-V "), Ok(("", Config::Version)));
        assert_eq!(parse_config("--version "), Ok(("", Config::Version)));
        assert_eq!(parse_config("-v "), Ok(("", Config::Verbose)));
        assert_eq!(parse_config("--verbose "), Ok(("", Config::Verbose)));
        assert_eq!(parse_config("-d "), Ok(("", Config::DryRun)));
        assert_eq!(parse_config("--dry-run "), Ok(("", Config::DryRun)));
        assert_eq!(parse_config("-n "), Ok(("", Config::Nocase)));
        assert_eq!(parse_config("--nocase "), Ok(("", Config::Nocase)));
        assert!(parse_config("-h").is_err());
        assert!(parse_config("abc ").is_err());
    }
    #[test]
    fn test_parse_configs() {
        assert_eq!(
            parse_configs("-h -V -v -d "),
            Ok(("", vec![Config::Help, Config::Version, Config::Verbose, Config::DryRun]))
        );
    }
}
