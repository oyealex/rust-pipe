use cmd_help::CmdHelp;
use std::process::{ExitCode, Termination};
use thiserror::Error;

#[derive(Error, Debug, Eq, PartialEq, CmdHelp)]
pub(crate) enum RpErr {
    /// 1       解析配置Token失败。
    #[error("[ParseConfigTokenErr] Invalid args: {0}")]
    ParseConfigTokenErr(String),

    /// 2       解析输入Token失败。
    #[error("[ParseInputTokenErr] Invalid args: {0}")]
    ParseInputTokenErr(String),

    /// 3       解析操作Token失败。
    #[error("[ParseOpTokenErr] Invalid args: {0}")]
    ParseOpTokenErr(String),

    /// 4       解析输出Token失败。
    #[error("[ParseOutputTokenErr] Invalid args: {0}")]
    ParseOutputTokenErr(String),

    /// 5       参数解析失败。
    #[error("[ArgParseErr] Unable to parse `{arg_value}` in argument `{arg}` of cmd `{cmd}`, error: {error}")]
    ArgParseErr { cmd: &'static str, arg: &'static str, arg_value: String, error: String },

    /// 6       参数内容无法完全解析，存在剩余无法解析的内容。
    #[error("[UnexpectedRemaining] Unexpected remaining value `{remaining}` in argument `{arg}` of cmd `{cmd}`")]
    UnexpectedRemaining { cmd: &'static str, arg: &'static str, remaining: String },

    /// 7       命令缺少参数。
    #[error("[MissingArg] Missing argument `{arg}` of cmd `{cmd}`")]
    MissingArg { cmd: &'static str, arg: &'static str },

    /// 8       未知参数。
    #[error("[UnknownArgs] Unknown arguments: {args:?}")]
    UnknownArgs { args: Vec<String> },

    /// 9       从剪切板读取数据失败。
    #[error("[ReadClipboardTextErr] Read text from clipboard error: {0}")]
    ReadClipboardTextErr(String),

    /// 10      从文件读取数据失败。
    #[error("[ReadFromFileErr] Read line `{line_no}` of file `{file}` error: {err}")]
    ReadFromFileErr { file: String, line_no: usize, err: String },

    /// 11      写入数据到剪切板失败。
    #[error("[WriteToClipboardErr] Write result to clipboard error: {0}")]
    WriteToClipboardErr(String),

    /// 12      打开文件失败。
    #[error("[OpenFileErr] Open output file `{file}` error: {err}")]
    OpenFileErr { file: String, err: String },

    /// 13      写入数据到文件失败。
    #[error("[WriteToFileErr] Write item `{item}` to file `{file}` error: {err}")]
    WriteToFileErr { file: String, item: String, err: String },

    /// 14      格式化字符串失败。
    #[error("[FormatStringErr] Format string by {fmt} with `{value}` error at: {err_pos}")]
    FormatStringErr { fmt: String, value: String, err_pos: usize },

    /// 15      无效的正则表达式。
    #[error("[ParseRegexErr] Parse regex {reg:?} err: {err}")]
    ParseRegexErr { reg: String, err: String},
}

impl Termination for RpErr {
    fn report(self) -> ExitCode {
        eprintln!("{}", self);
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
        let mut code = 0u8..;
        match self {
            RpErr::ParseConfigTokenErr(_) => code.next().unwrap(),
            RpErr::ParseInputTokenErr(_) => code.next().unwrap(),
            RpErr::ParseOpTokenErr(_) => code.next().unwrap(),
            RpErr::ParseOutputTokenErr(_) => code.next().unwrap(),
            RpErr::ArgParseErr { .. } => code.next().unwrap(),
            RpErr::UnexpectedRemaining { .. } => code.next().unwrap(),
            RpErr::MissingArg { .. } => code.next().unwrap(),
            RpErr::UnknownArgs { .. } => code.next().unwrap(),
            RpErr::ReadClipboardTextErr(_) => code.next().unwrap(),
            RpErr::ReadFromFileErr { .. } => code.next().unwrap(),
            RpErr::WriteToClipboardErr(_) => code.next().unwrap(),
            RpErr::OpenFileErr { .. } => code.next().unwrap(),
            RpErr::WriteToFileErr { .. } => code.next().unwrap(),
            RpErr::FormatStringErr { .. } => code.next().unwrap(),
            RpErr::ParseRegexErr { .. } => code.next().unwrap(),
        }
    }
}
