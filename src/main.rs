#![allow(unused)] // TODO 2025-12-26 22:47 移除告警禁用

use crate::err::RpErr;
use crate::input::Input;
use crate::op::Op;
use crate::output::Output;

mod input;
mod op;
mod output;
mod parse;
mod err;

/// 整数类型
pub(crate) type Integer = i64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1).peekable();
    match parse::args::parse(args) {
        Ok((input, ops, output)) => {
            println!("input: {:?}", input);
            println!("ops: {:?}", ops);
            println!("output: {:?}", output);
            output.handle(ops.into_iter().fold(input.pipe(), |pipe, op| op.wrap(pipe)));
        }
        Err(err) => eprintln!("{}", err),
    }
    Ok(())
}
