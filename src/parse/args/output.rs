use crate::err::RpErr;
use crate::output::Output;
use crate::parse::{args, OutputResult};
use args::parse_general_file_info;
use std::iter::Peekable;

pub(in crate::parse::args) fn parse_output(args: &mut Peekable<impl Iterator<Item = String>>) -> OutputResult {
    if let Some(to_cmd) = args.peek()
        && to_cmd.eq_ignore_ascii_case(":to")
    {
        args.next(); // 消耗`to`
        match args.peek() {
            Some(output) => {
                let lower_output = output.to_ascii_lowercase();
                match lower_output.as_str() {
                    "file" => parse_file(args),
                    #[cfg(windows)]
                    "clip" => parse_clip(args),
                    "out" => parse_std_out(args),
                    _ => Ok(Output::new_std_out()),
                }
            }
            None => Ok(Output::new_std_out()),
        }
    } else {
        Ok(Output::new_std_out())
    }
}

fn parse_file(args: &mut Peekable<impl Iterator<Item = String>>) -> OutputResult {
    args.next(); // 消耗`file`
    if let Some((file, append, crlf)) = parse_general_file_info(args, false) {
        Ok(Output::new_file(file, append, crlf))
    } else {
        Err(RpErr::MissingArg { cmd: ":to file", arg: "file" })
    }
}

#[cfg(windows)]
fn parse_clip(args: &mut Peekable<impl Iterator<Item = String>>) -> OutputResult {
    args.next(); // 消耗`clip`
    let postfix = if let Some(crlf) = args.peek() {
        if crlf.eq_ignore_ascii_case("crlf") {
            args.next(); // 消耗`crlf`
            Some(true)
        } else if crlf.eq_ignore_ascii_case("lf") {
            args.next(); // 消耗`lf`
            Some(false)
        } else {
            None
        }
    } else {
        None
    };
    Ok(Output::new_clip(postfix))
}

fn parse_std_out(args: &mut Peekable<impl Iterator<Item = String>>) -> OutputResult {
    args.next(); // 消耗`out`
    Ok(Output::new_std_out())
}
