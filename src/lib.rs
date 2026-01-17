use crate::config::Config;
use crate::err::RpErr;
use crate::pipe::Pipe;
use std::iter::Peekable;
use std::str::FromStr;

mod condition;
mod config;
mod err;
mod fmt;
mod help;
mod input;
pub(crate) mod op;
mod output;
mod parse;
mod pipe;
pub(crate) mod print;

pub(crate) type Integer = i64;
pub(crate) type Float = f64;

#[derive(Debug, Copy, Clone)]
pub(crate) enum Num {
    Integer(Integer),
    Float(Float),
}

impl From<Integer> for Num {
    fn from(i: Integer) -> Num {
        Num::Integer(i)
    }
}

impl From<Float> for Num {
    fn from(f: Float) -> Num {
        Num::Float(f)
    }
}

impl FromStr for Num {
    type Err = RpErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(i) = s.parse::<Integer>() {
            Ok(Num::Integer(i))
        } else if let Ok(f) = s.parse::<Float>()
            && f.is_finite()
        {
            Ok(Num::Float(f))
        } else {
            Err(RpErr::ParseNumErr(s.to_owned()))
        }
    }
}

impl PartialOrd for Num {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Num::Integer(a), Num::Integer(b)) => a.partial_cmp(b),
            (Num::Float(a), Num::Float(b)) => a.partial_cmp(b),
            (Num::Integer(a), Num::Float(b)) => (*a as Float).partial_cmp(b),
            (Num::Float(a), Num::Integer(b)) => a.partial_cmp(&(*b as Float)),
        }
    }
}

impl PartialEq for Num {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Num::Integer(a), Num::Integer(b)) => a == b,
            (Num::Float(a), Num::Float(b)) => a == b,
            (Num::Integer(a), Num::Float(b)) => (*a as Float) == *b,
            (Num::Float(a), Num::Integer(b)) => *a == (*b as Float),
        }
    }
}

pub(crate) type PipeRes = Result<Pipe, RpErr>;

pub fn run(mut args: Peekable<impl Iterator<Item = String>>) -> Result<(), RpErr> {
    let configs = parse::args::parse_configs(&mut args);
    if configs.contains(&Config::Help) {
        help::print_help(&mut args);
        return Ok(());
    } else if configs.contains(&Config::Version) {
        help::print_version();
        return Ok(());
    }
    let (input, ops, output) =
        if configs.contains(&Config::Token) { config::parse_eval_token(&mut args)? } else { parse::args::parse(args)? };
    if configs.contains(&Config::Verbose) {
        config::print_pipe_info(&input, &ops, &output);
    }
    let configs: &'static mut [Config] = configs.leak();
    let mut pipe = input.try_into(configs)?;
    for op in ops {
        pipe = op.wrap(pipe, configs)?;
    }
    if configs.contains(&Config::DryRun) { Ok(()) } else { output.handle(pipe) }
}
