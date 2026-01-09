use crate::err::RpErr;
use crate::input::Pipe;
use cmd_help::CmdHelp;
use itertools::Itertools;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Debug, Eq, PartialEq, CmdHelp)]
pub(crate) enum Output {
    /// :to out     输出到标准输出。
    ///             未指定元素输出时的默认输出。
    StdOut,
    /// :to file    输出到文件。
    ///             :to file <file_name>[ append][ lf|crlf]
    ///                 <file_name> 文件路径，必选。
    ///                 append      追加输出而不是覆盖，可选，如果未指定则覆盖源文件。
    ///                 lf|crlf     指定换行符为'LF'或'CRLF'，可选，如果未指定则默认使用'LF'。
    ///             例如：
    ///                 :to file out.txt
    ///                 :to file out.txt append
    ///                 :to file out.txt crlf
    ///                 :to file out.txt lf
    ///                 :to file out.txt append crlf
    ///                 :to file out.txt append lf
    File { file: String, append: bool, crlf: Option<bool> },
    /// :to clip    输出到剪切板。
    ///             :to clip[ lf|crlf]
    ///                 lf|crlf 指定换行符为'LF'或'CRLF'，可选，如果未指定则默认使用'LF'。
    ///             例如：
    ///                 :to clip
    ///                 :to clip lf
    ///                 :to clip crlf
    Clip { crlf: Option<bool> },
}

impl Output {
    pub(crate) fn new_std_out() -> Self {
        Output::StdOut
    }
    pub(crate) fn new_file(file: String, append: bool, crlf: Option<bool>) -> Self {
        Output::File { file, append, crlf }
    }
    pub(crate) fn new_clip(crlf: Option<bool>) -> Self {
        Output::Clip { crlf }
    }

    pub(crate) fn handle(self, pipe: Pipe) -> Result<(), RpErr> {
        match self {
            Output::StdOut => {
                for item in pipe {
                    println!("{item}");
                }
                Ok(())
            }
            Output::File { file, append, crlf } => {
                match OpenOptions::new().write(true).truncate(!append).append(append).create(true).open(&file) {
                    Ok(mut writer) => {
                        let ending = if crlf.unwrap_or(false) { "\r\n" } else { "\n" };
                        for item in pipe {
                            write!(writer, "{item}{ending}").map_err(|err| RpErr::WriteToFileErr {
                                file: file.clone(),
                                item: item.to_string(),
                                err: err.to_string(),
                            })?
                        }
                        Ok(())
                    }
                    Err(err) => Err(RpErr::OpenFileErr { file, err: err.to_string() }),
                }
            }
            Output::Clip { crlf } => {
                let text = pipe.map(String::from).join(if crlf.unwrap_or(false) { "\r\n" } else { "\n" });
                clipboard_win::set_clipboard_string(&text).map_err(|err| RpErr::WriteToClipboardErr(err.to_string()))
            }
        }
    }
}
