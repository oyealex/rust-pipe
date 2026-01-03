use crate::err::RpErr;
use crate::output::Output;
use std::iter::Peekable;

pub(in crate::parse::args) fn parse_output(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Output, RpErr> {
    if let Some(to_cmd) = args.peek()
        && to_cmd.eq_ignore_ascii_case("to")
    {
        args.next(); // 消耗`to`
        match args.peek() {
            Some(output) => {
                if output.eq_ignore_ascii_case("file") {
                    parse_file(args)
                } else if output.eq_ignore_ascii_case("clip") {
                    parse_clip(args)
                } else if output.eq_ignore_ascii_case("out") {
                    parse_std_out(args)
                } else {
                    Ok(Output::new_std_out())
                }
            }
            None => Ok(Output::new_std_out()),
        }
    } else {
        Ok(Output::new_std_out())
    }
}

fn parse_file(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Output, RpErr> {
    args.next(); // 消耗`file`
    if let Some(file) = args.next() {
        // 必须文件名，直接消耗
        let (append, crlf) = if let Some(append_or_ending) = args.peek() {
            if append_or_ending.eq_ignore_ascii_case("append") {
                args.next(); // 消耗`append`
                if let Some(crlf) = args.peek() {
                    if crlf.eq_ignore_ascii_case("crlf") {
                        args.next(); // 消耗`crlf`
                        (true, Some(true))
                    } else if crlf.eq_ignore_ascii_case("lf") {
                        args.next(); // 消耗`lf`
                        (true, Some(false))
                    } else {
                        (true, None)
                    }
                } else {
                    (true, None)
                }
            } else if append_or_ending.eq_ignore_ascii_case("crlf") {
                args.next(); // 消耗`crlf`
                (false, Some(true))
            } else if append_or_ending.eq_ignore_ascii_case("lf") {
                args.next(); // 消耗`lf`
                (false, Some(false))
            } else {
                (false, None)
            }
        } else {
            (false, None)
        };
        Ok(Output::new_file(file, append, crlf))
    } else {
        Err(RpErr::MissingArg { cmd: "to file", arg: "file" })
    }
}

fn parse_clip(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Output, RpErr> {
    args.next(); // 消耗`clip`
    Ok(Output::new_clip())
}

fn parse_std_out(args: &mut Peekable<impl Iterator<Item = String>>) -> Result<Output, RpErr> {
    args.next(); // 消耗`out`
    Ok(Output::new_std_out())
}
