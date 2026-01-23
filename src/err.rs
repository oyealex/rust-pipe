use cmd_help::CmdHelp;
use std::process::{ExitCode, Termination};
use thiserror::Error;

#[derive(Error, Clone, Debug, Eq, PartialEq, CmdHelp)]
pub enum RpErr {
    ///  1      解析配置Token失败。
    #[error("[ParseTokenErr:1] Parse token err: {0}")]
    ParseTokenErr(String),

    ///  2      参数解析失败。
    #[error("[ArgParseErr:2] Unable to parse {arg_value:?} in argument `{arg}` of cmd `{cmd}`, error: {error}")]
    ArgParseErr { cmd: &'static str, arg: &'static str, arg_value: String, error: String },

    ///  3      命令缺少有效参数。
    #[error("[MissingArg:3] Missing valid argument `{arg}` of cmd `{cmd}`")]
    MissingArg { cmd: &'static str, arg: &'static str },

    ///  4      参数内容无法完全解析，存在剩余无法解析的内容。
    #[error("[UnexpectedRemaining:4] Unexpected remaining value {remaining:?} in argument `{arg}` of cmd `{cmd}`")]
    UnexpectedRemaining { cmd: &'static str, arg: &'static str, remaining: String },

    ///  5      未知参数。
    #[error("[UnknownArgs:5] Unknown arguments: {args:?}")]
    UnknownArgs { args: Vec<String> },

    ///  6      从剪切板读取数据失败。
    #[error("[ReadClipboardTextErr:6] Read text from clipboard error: {0}")]
    ReadClipboardTextErr(String),

    /// 7      从文件读取数据失败。
    #[error("[ReadFromFileErr:7] Read line {line_no} of file {file:?} error: {err}")]
    ReadFromFileErr { file: String, line_no: usize, err: String },

    /// 8      写入数据到剪切板失败。
    #[error("[WriteToClipboardErr:8] Write result to clipboard error: {0}")]
    WriteToClipboardErr(String),

    /// 9      打开文件失败。
    #[error("[OpenFileErr:9] Open output file {file:?} error: {err}")]
    OpenFileErr { file: String, err: String },

    /// 10      写入数据到文件失败。
    #[error("[WriteToFileErr:10] Write item {item:?} to file {file:?} error: {err}")]
    WriteToFileErr { file: String, item: String, err: String },

    /// 11      格式化字符串失败。
    #[error("[FormatStringErr:11] Format string by {fmt:?} with {value} error at: {err_pos}")]
    FormatStringErr { fmt: String, value: String, err_pos: usize },

    /// 12      解析正则表达式失败。
    #[error("[ParseRegexErr:12] Parse regex from {reg:?} err: {err}")]
    ParseRegexErr { reg: String, err: String },

    /// 13      解析数值失败。
    #[error("[ParseRegexErr:13] Parse number from {0:?} err")]
    ParseNumErr(String),

    /// 14      无效的非负整数参数。
    #[error(
        "[InvalidNonNegativeIntArg:14] Positive integer or zero is required by argument `{arg}` of cmd `{cmd}`, but it is {arg_value:?}"
    )]
    InvalidNonNegativeIntArg { cmd: &'static str, arg: &'static str, arg_value: String },
}

impl Termination for RpErr {
    fn report(self) -> ExitCode {
        crate::println_err!("{}", self);
        ExitCode::from(self.exit_code())
    }
}

impl RpErr {
    pub fn termination(self) -> ! {
        let exit_code = self.exit_code();
        self.report();
        std::process::exit(exit_code as i32);
    }

    fn exit_code(&self) -> u8 {
        match self {
            RpErr::ParseTokenErr(_) => 1,
            RpErr::ArgParseErr { .. } => 2,
            RpErr::MissingArg { .. } => 3,
            RpErr::UnexpectedRemaining { .. } => 4,
            RpErr::UnknownArgs { .. } => 5,
            RpErr::ReadClipboardTextErr(_) => 6,
            RpErr::ReadFromFileErr { .. } => 7,
            RpErr::WriteToClipboardErr(_) => 8,
            RpErr::OpenFileErr { .. } => 9,
            RpErr::WriteToFileErr { .. } => 10,
            RpErr::FormatStringErr { .. } => 11,
            RpErr::ParseRegexErr { .. } => 12,
            RpErr::ParseNumErr { .. } => 13,
            RpErr::InvalidNonNegativeIntArg { .. } => 14,
        }
    }
}
