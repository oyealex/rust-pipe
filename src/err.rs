use cmd_help::CmdHelp;
use std::process::{ExitCode, Termination};
use thiserror::Error;

#[derive(Error, Debug, Eq, PartialEq, CmdHelp)]
pub enum RpErr {
    /// 1       解析配置Token失败。
    #[error("[ParseConfigTokenErr:1] Invalid args: {0}")]
    ParseConfigTokenErr(String),

    /// 2       解析输入Token失败。
    #[error("[ParseInputTokenErr:2] Invalid args: {0}")]
    ParseInputTokenErr(String),

    /// 3       解析操作Token失败。
    #[error("[ParseOpTokenErr:3] Invalid args: {0}")]
    ParseOpTokenErr(String),

    /// 4       解析输出Token失败。
    #[error("[ParseOutputTokenErr:4] Invalid args: {0}")]
    ParseOutputTokenErr(String),

    /// 5       参数解析失败。
    #[error("[ArgParseErr:5] Unable to parse `{arg_value}` in argument `{arg}` of cmd `{cmd}`, error: {error}")]
    ArgParseErr { cmd: &'static str, arg: &'static str, arg_value: String, error: String },

    /// 6       命令缺少参数。
    #[error("[MissingArg:6] Missing argument `{arg}` of cmd `{cmd}`")]
    MissingArg { cmd: &'static str, arg: &'static str },

    /// 7       参数内容无法完全解析，存在剩余无法解析的内容。
    #[error("[UnexpectedRemaining:7] Unexpected remaining value `{remaining}` in argument `{arg}` of cmd `{cmd}`")]
    UnexpectedRemaining { cmd: &'static str, arg: &'static str, remaining: String },

    /// 8       未知参数。
    #[error("[UnknownArgs:8] Unknown arguments: {args:?}")]
    UnknownArgs { args: Vec<String> },

    /// 9       从剪切板读取数据失败。
    #[error("[ReadClipboardTextErr:9] Read text from clipboard error: {0}")]
    ReadClipboardTextErr(String),

    /// 10      从文件读取数据失败。
    #[error("[ReadFromFileErr:10] Read line `{line_no}` of file `{file}` error: {err}")]
    ReadFromFileErr { file: String, line_no: usize, err: String },

    /// 11      写入数据到剪切板失败。
    #[error("[WriteToClipboardErr:11] Write result to clipboard error: {0}")]
    WriteToClipboardErr(String),

    /// 12      打开文件失败。
    #[error("[OpenFileErr:12] Open output file `{file}` error: {err}")]
    OpenFileErr { file: String, err: String },

    /// 13      写入数据到文件失败。
    #[error("[WriteToFileErr:13] Write item `{item}` to file `{file}` error: {err}")]
    WriteToFileErr { file: String, item: String, err: String },

    /// 14      格式化字符串失败。
    #[error("[FormatStringErr:14] Format string by {fmt} with `{value}` error at: {err_pos}")]
    FormatStringErr { fmt: String, value: String, err_pos: usize },

    /// 15      解析正则表达式失败。
    #[error("[ParseRegexErr:15] Parse regex from {reg:?} err: {err}")]
    ParseRegexErr { reg: String, err: String },

    /// 16      解析数值失败。
    #[error("[ParseRegexErr:16] Parse number from {0:?} err")]
    ParseNumErr(String),
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
            RpErr::ParseConfigTokenErr(_) => 1,
            RpErr::ParseInputTokenErr(_) => 2,
            RpErr::ParseOpTokenErr(_) => 3,
            RpErr::ParseOutputTokenErr(_) => 4,
            RpErr::ArgParseErr { .. } => 5,
            RpErr::MissingArg { .. } => 6,
            RpErr::UnexpectedRemaining { .. } => 7,
            RpErr::UnknownArgs { .. } => 8,
            RpErr::ReadClipboardTextErr(_) => 9,
            RpErr::ReadFromFileErr { .. } => 10,
            RpErr::WriteToClipboardErr(_) => 11,
            RpErr::OpenFileErr { .. } => 12,
            RpErr::WriteToFileErr { .. } => 13,
            RpErr::FormatStringErr { .. } => 14,
            RpErr::ParseRegexErr { .. } => 15,
            RpErr::ParseNumErr { .. } => 16,
        }
    }
}
