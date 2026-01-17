use crate::err::RpErr;
use crate::input::Input;
use crate::op::Op;
use crate::output::Output;
use crate::parse;
use cmd_help::CmdHelp;
use itertools::Itertools;
use std::iter::Peekable;

#[derive(Debug, Eq, PartialEq, CmdHelp)]
pub(crate) enum Config {
    /// -V,--version    打印版本信息。
    Version,
    /// -h,--help       打印帮助信息。
    ///                 -h|-help[ <topic>]
    ///                     <topic>     帮助主题：
    ///                         opt|options     打印选项帮助信息。
    ///                         in|input        打印数据输入命令帮助信息。
    ///                         op              打印数据操作命令帮助信息。
    ///                         out|output      打印数据输出命令帮助信息。
    ///                         code            打印退出码帮助信息。
    ///                         fmt             打印格式化帮助信息。
    ///                         cond|condition  打印条件表达式帮助信息。
    ///                                 未指定则打印全部帮助信息。
    Help,
    /// -v,--verbose    执行之前打印流水线详情。
    Verbose,
    /// -d,--dry-run    仅解析流水线，不执行。
    DryRun,
    /// -n,--nocase     全局忽略大小写。
    Nocase,
    /// -s,--skip-err 全局忽略错误。
    SkipErr,
    /// -t,--token      以Token模式解析下一个参数。
    ///                 除了紧跟的第一个参数外，其他参数会被忽略。
    ///                 -t|--token <token>
    ///                     <token> 需要解析的文本参数，必选。
    ///                 例如：
    ///                     -e ':in :uniq :to out'
    Token,
}

#[inline]
pub(crate) fn is_nocase(nocase: bool, configs: &[Config]) -> bool {
    nocase || configs.contains(&Config::Nocase)
}

#[inline]
pub(crate) fn skip_err(configs: &[Config]) -> bool {
    configs.contains(&Config::SkipErr)
}

pub(crate) fn print_pipe_info(input: &Input, ops: &Vec<Op>, output: &Output) {
    println!("Input:");
    println!("    {:?}", input);
    println!("Op:");
    println!("{}", ops.iter().map(|op| format!("    {:?}", op)).join("\n"));
    println!("Output:");
    println!("    {:?}", output);
}

pub(crate) fn parse_eval_token(
    args: &mut Peekable<impl Iterator<Item = String>>,
) -> Result<(Input, Vec<Op>, Output), RpErr> {
    if let Some(mut token) = args.next() {
        token.push(' ');
        match parse::token::parse_without_configs(&token.trim_start()) {
            Ok((remaining, res)) => {
                if !remaining.is_empty() {
                    Err(RpErr::UnexpectedRemaining { cmd: "--token", arg: "<token>", remaining: remaining.to_owned() })?
                }
                Ok(res)
            }
            Err(err) => {
                Err(RpErr::ArgParseErr { cmd: "--token", arg: "<token>", arg_value: token, error: err.to_string() })?
            }
        }
    } else {
        Err(RpErr::MissingArg { cmd: "--token", arg: "<token>" })?
    }
}
