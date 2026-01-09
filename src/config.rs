use crate::err::RpErr;
use crate::input::Input;
use crate::op::Op;
use crate::output::Output;
use crate::parse;
use cmd_help::CmdHelp;
use itertools::Itertools;
use std::env::Args;
use std::iter::{Peekable, Skip};

#[derive(Debug, Eq, PartialEq, CmdHelp)]
pub(crate) enum Config {
    /// -V,--version    打印版本信息。
    Version,
    /// -h,--help       打印帮助信息。
    ///                 -h|-help[ options|input|op|output|code]
    ///                     options 打印选项帮助信息。
    ///                     input   打印数据输入命令帮助信息。
    ///                     op      打印数据操作命令帮助信息。
    ///                     output  打印数据输出命令帮助信息。
    ///                     code    打印退出码帮助信息。
    ///                     未指定则打印全部帮助信息。
    Help,
    /// -v,--verbose    执行之前打印流水线详情。
    Verbose,
    /// -d,--dry-run    仅解析流水线，不执行。
    DryRun,
    /// -n,--nocase     全局忽略大小写。
    Nocase,
    /// -e,--eval       以Token模式解析下一个参数。
    ///                 除了紧跟的第一个参数外，其他参数会被忽略。
    ///                 -e|--eval <token>
    ///                     <token> 需要解析的文本参数，必选。
    ///                 例如：
    ///                     -e ':in :uniq :to out'
    Eval,
}

#[inline]
pub(crate) fn is_nocase(nocase: bool, configs: &[Config]) -> bool {
    nocase || configs.contains(&Config::Nocase)
}

pub(crate) fn print_pipe_info(input: &Input, ops: &Vec<Op>, output: &Output) {
    println!("Input:");
    println!("    {:?}", input);
    println!("Op:");
    println!("{}", ops.iter().map(|op| format!("    {:?}", op)).join("\n"));
    println!("Output:");
    println!("    {:?}", output);
}

pub(crate) fn parse_eval_token(args: &mut Peekable<Skip<Args>>) -> Result<(Input, Vec<Op>, Output), RpErr> {
    if let Some(mut token) = args.next() {
        token.push(' ');
        match parse::token::parse_without_configs(&token.trim_start()) {
            Ok((remaining, res)) => {
                if !remaining.is_empty() {
                    Err(RpErr::UnexpectedRemaining { cmd: "--eval", arg: "token", remaining: remaining.to_owned() })?
                }
                Ok(res)
            }
            Err(err) => {
                Err(RpErr::ArgParseErr { cmd: "--eval", arg: "token", arg_value: token, error: err.to_string() })?
            }
        }
    } else {
        Err(RpErr::MissingArg { cmd: "--eval", arg: "token" })?
    }
}
