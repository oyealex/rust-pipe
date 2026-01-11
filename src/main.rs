use crate::config::Config;
use crate::err::RpErr;
use crate::pipe::Pipe;

mod config;
mod err;
mod fmt;
mod help;
mod input;
mod op;
mod output;
mod parse;
mod pipe;
mod condition;

pub(crate) type Integer = i64;
pub(crate) type Float = f64;

pub(crate) type PipeRes = Result<Pipe, RpErr>;

fn main() {
    if let Err(e) = run() {
        e.termination();
    }
}

fn run() -> Result<(), RpErr> {
    let mut args = std::env::args().skip(1).peekable();
    let configs = parse::args::parse_configs(&mut args);
    if configs.contains(&Config::Help) {
        help::print_help(args.next());
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
    let mut pipe = input.try_into()?;
    for op in ops {
        pipe = op.wrap(pipe, configs)?;
    }
    if configs.contains(&Config::DryRun) { Ok(()) } else { output.handle(pipe) }
}
