use crate::config::Config;
use std::iter::Peekable;

pub fn parse_configs(args: &mut Peekable<impl Iterator<Item = String>>) -> Vec<Config> {
    let mut configs = Vec::new();
    while let Some(config) = parse_config(args.peek()) {
        args.next();
        configs.push(config);
    }
    configs
}

fn parse_config(arg: Option<&String>) -> Option<Config> {
    match arg {
        Some(arg) => match arg.as_str() {
            "-h" | "--help" => Some(Config::Help),
            "-V" | "--version" => Some(Config::Version),
            "-v" | "--verbose" => Some(Config::Verbose),
            "-d" | "--dry-run" => Some(Config::DryRun),
            "-n" | "--nocase" => Some(Config::Nocase),
            "-t" | "--token" => Some(Config::Token),
            _ => None, // 遇到未知参数，停止解析
        },
        None => None,
    }
}
