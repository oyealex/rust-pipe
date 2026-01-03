use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum RpErr {
    #[error("[Input Token] Invalid args: {0}")]
    ParseInputTokenErr(String),

    #[error("[Op Token] Invalid args: {0}")]
    ParseOpTokenErr(String),

    #[error("[Output Token] Invalid args: {0}")]
    ParseOutputTokenErr(String),

    #[error("[Arg Parse Err] Unable to parse `{arg_value}` in argument `{arg}` of cmd `{cmd}`, error: {error}")]
    ArgParseErr { cmd: &'static str, arg: &'static str, arg_value: String, error: String },

    #[error("[Bad Arg] Bad value `{arg_value}` in argument `{arg}` of cmd `{cmd}`")]
    BadArg { cmd: &'static str, arg: &'static str, arg_value: String },

    #[error("[Bad Arg] Unexpected remaining value `{remaining}` in argument `{arg}` of cmd `{cmd}`")]
    UnexpectedRemaining { cmd: &'static str, arg: &'static str, remaining: String },

    #[error("[Missing Arg] Missing argument `{arg}` of cmd `{cmd}`")]
    MissingArg { cmd: &'static str, arg: &'static str },

    #[error("[Missing Arg] At least one value for argument `{arg}` is required for cmd `{cmd}`")]
    ArgNotEnough { cmd: &'static str, arg: &'static str },

    #[error("[Bad Arg] Closing bracket (`]`) for argument `{arg}` is required for cmd `{cmd}`")]
    UnclosingMultiArg { cmd: &'static str, arg: &'static str },

    #[error("[Bad Arg] Unexpected closing bracket of argument `{arg}` for cmd `{cmd}`")]
    UnexpectedClosingBracket { cmd: &'static str, arg: &'static str },
    
    #[error("[Bad Arg] Unknown arguments: {args:?}")]
    UnknownArgs{args: Vec<String>},
}
