#![allow(unused)] // TODO 2025-12-26 22:47 移除告警禁用

mod input;
mod op;
mod output;
mod parse;

use crate::input::Item;
use itertools::Itertools;
use nom::Finish;
use std::io::BufRead;

/// 整数类型
pub(crate) type Integer = i64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use nom::Parser;
    let input = "gen -10,10,-10 ";

    let (remaining, (input, output)) = parse::parse(input)?;
    println!("remaining: {:?}", remaining);
    println!("input: {:?}", input);
    println!("output: {:?}", output);
    input.iter().for_each(|item| match item {
        Item::Integer(value) => println!(">> int: {:?}", value),
        Item::String(value) => println!(">> str: {:?}", value),
    });
    Ok(())
}
