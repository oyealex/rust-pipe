use crate::err::RpErr;
use crate::fmt::{fmt_args, FmtArg};
use crate::pipe::Pipe;
use crate::{Integer, PipeRes};
use cmd_help::CmdHelp;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::iter::repeat;
use std::rc::Rc;

#[derive(Debug, Eq, PartialEq, CmdHelp)]
pub(crate) enum Input {
    /// :in         从标准输入读取输入。
    ///             未指定元素输入时的默认输入。
    StdIn,
    /// :file       从文件读取输入。
    ///             :file <file_name>[ <file_name>][...]
    ///                 <file_name> 文件路径，至少指定一个。
    ///             例如：
    ///                 :file input.txt
    ///                 :file input1.txt input2.txt input3.txt
    File { files: Vec<String> },
    /// :clip       从剪切板读取输入。
    Clip,
    /// :of         使用直接字面值作为输入。
    ///             :of <text>[ <text][...]
    ///                 <text>  字面值，至少指定一个，如果以':'开头，需要使用'::'转义。
    ///             例如：
    ///                 :of line
    ///                 :of line1 "line 2" 'line 3'
    Of { values: Vec<String> },
    /// :gen        生成指定范围内的整数作为输入，支持进一步格式化。
    ///             :gen <start>[,[<end>][,<step>]][ <fmt>]
    ///                 <start> 起始值，包含，必须。
    ///                 <end>   结束值，包含，可选。
    ///                         未指定时生成到整数最大值（取决于构建版本）。
    ///                         如果范围为空（起始值大于结束值），则无数据生成。
    ///                 <step>  步长，不能为0，可选，未指定时取步长为1。
    ///                         如果步长为正值，表示正序生成；
    ///                         如果步长为负值，表示逆序生成。
    ///                 <fmt>   格式化字符串，以{v}表示生成的整数值。
    ///                         更多格式化信息参考`-h fmt`。
    ///             例如：
    ///                 :gen 0          生成：0 1 2 3 4 5 ...
    ///                 :gen 0,10       生成：0 1 2 3 4 5 6 7 8 9
    ///                 :gen 0,10,2     生成：0 2 4 6 8
    ///                 :gen 0,,2       生成：0 2 4 6 8 10 12 14 ...
    ///                 :gen 10,0       无数据生成
    ///                 :gen 0,10,-1    生成：9 8 7 6 5 4 3 2 1
    ///                 :gen 0,10 n{v}  生成：n0 n1 n2 n3 n4 n5 n6 n7 n8 n9
    ///                 :gen 0,10 "Hex of {v} is {v:#04x}" 生成：
    ///                                 "Hex of 0 is 0x00"
    ///                                 "Hex of 1 is 0x01"
    ///                                 ...
    Gen { start: Integer, end: Integer, step: Integer, fmt: Option<String> },
    /// :repeat     重复字面值作为输入。
    ///             :repeat <value>[ <count>]
    ///                 <value> 需要重复的字面值，必选。
    ///                 <count> 需要重复的次数，必须为非负数，可选，未指定时重复无限次数。
    Repeat { value: String, count: Option<usize> },
}

impl Input {
    pub(crate) fn new_std_in() -> Input {
        Input::StdIn
    }
    pub(crate) fn new_file(files: Vec<String>) -> Input {
        Input::File { files }
    }
    pub(crate) fn new_clip() -> Input {
        Input::Clip
    }
    pub(crate) fn new_of(values: Vec<String>) -> Input {
        Input::Of { values }
    }
    pub(crate) fn new_gen(start: Integer, end: Integer, step: Integer, fmt: Option<String>) -> Input {
        Input::Gen { start, end, step, fmt }
    }
    pub(crate) fn new_repeat(value: String, count: Option<usize>) -> Input {
        Input::Repeat { value, count }
    }
}

impl TryInto<Pipe> for Input {
    type Error = RpErr;

    fn try_into(self) -> PipeRes {
        match self {
            Input::StdIn => Ok(Pipe {
                iter: Box::new(
                    io::stdin().lock().lines().into_iter().take_while(Result::is_ok).map(|line| line.unwrap()),
                ),
            }),
            Input::File { files } => Ok(Pipe {
                iter: Box::new(
                    files
                        .into_iter()
                        .map(|f| (File::open(&f), f))
                        .map(|(r, f)| {
                            match r {
                                Ok(fin) => (fin, f),
                                Err(err) => {
                                    // TODO 2026-01-05 01:18 根据全局配置选择跳过
                                    RpErr::OpenFileErr { file: f, err: err.to_string() }.termination();
                                }
                            }
                        })
                        .map(|(fin, f)| (BufReader::new(fin), Rc::new(f)))
                        .flat_map(|(reader, f)| {
                            BufRead::lines(reader).into_iter().enumerate().map(move |l| (l, f.clone()))
                        })
                        .map(|((line, lr), f)| {
                            match lr {
                                Ok(line) => line,
                                Err(err) => {
                                    // TODO 2026-01-05 01:18 根据全局配置选择跳过
                                    RpErr::ReadFromFileErr { file: (*f).clone(), line_no: line, err: err.to_string() }
                                        .termination();
                                }
                            }
                        }),
                ),
            }),
            Input::Clip => match clipboard_win::get_clipboard_string() {
                // TODO 2026-01-05 01:02 尝试leak text，然后使用Item::Str省略内存分配
                Ok(text) => {
                    Ok(Pipe { iter: Box::new(text.lines().map(|s| s.to_string()).collect::<Vec<_>>().into_iter()) })
                }
                Err(err) => Err(RpErr::ReadClipboardTextErr(err.to_string())),
            },
            Input::Of { values } => Ok(Pipe { iter: Box::new(values.into_iter()) }),
            // TODO 2025-12-28 21:59 如果gen没有指定end，设定为Unbounded。
            Input::Gen { start, end, step, fmt } => {
                if let Some(fmt) = fmt {
                    Ok(Pipe {
                        iter: Box::new(range_to_iter(start, end, step).map(move |x| {
                            match fmt_args(&fmt, &[("v", FmtArg::from(x))]) {
                                Ok(string) => string,
                                Err(err) => err.termination(),
                            }
                        })),
                    })
                } else {
                    Ok(Pipe { iter: Box::new(range_to_iter(start, end, step).map(|s| s.to_string())) })
                }
            }
            Input::Repeat { value, count } => Ok(if count.is_none() {
                Pipe { iter: Box::new(repeat(value)) }
            } else {
                Pipe { iter: Box::new(repeat(value).take(count.unwrap())) }
            }),
        }
    }
}

fn range_to_iter(start: Integer, end: Integer, step: Integer) -> Box<dyn DoubleEndedIterator<Item = Integer>> {
    let iter = RangeIter { start, end, step: Integer::abs(step), next: start, next_back: end };
    if step < 0 { Box::new(iter.rev()) } else { Box::new(iter) }
}

#[derive(Debug, Eq, PartialEq)]
struct RangeIter {
    start: Integer,
    end: Integer,
    step: Integer,
    next: Integer,
    next_back: Integer,
}

impl Iterator for RangeIter {
    type Item = Integer;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next >= self.start && self.next <= self.end && self.next <= self.next_back {
            let res = Some(self.next);
            self.next += self.step;
            res
        } else {
            None
        }
    }
}

impl DoubleEndedIterator for RangeIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.next_back >= self.start && self.next_back <= self.end && self.next_back >= self.next {
            let res = Some(self.next_back);
            self.next_back -= self.step;
            res
        } else {
            None
        }
    }
}

#[cfg(test)]
mod iter_tests {
    use super::*;

    #[test]
    fn test_range_to_iter_positive() {
        assert_eq!(range_to_iter(0, 10, 1).collect::<Vec<_>>(), (0..=10).collect::<Vec<_>>());
        assert_eq!(range_to_iter(0, 10, 2).collect::<Vec<_>>(), (0..=10).step_by(2).collect::<Vec<_>>());
    }

    #[test]
    fn test_range_to_iter_negative() {
        assert_eq!(range_to_iter(0, 10, -1).collect::<Vec<_>>(), (0..=10).rev().collect::<Vec<_>>());
        assert_eq!(range_to_iter(0, 10, -2).collect::<Vec<_>>(), (0..=10).rev().step_by(2).collect::<Vec<_>>());
    }

    #[test]
    fn test_range_to_iter_empty() {
        assert_eq!(range_to_iter(0, 0, 1).collect::<Vec<_>>(), (0..=0).collect::<Vec<_>>());
        assert_eq!(range_to_iter(0, 0, 2).collect::<Vec<_>>(), (0..=0).step_by(2).collect::<Vec<_>>());
    }

    #[test]
    fn test_range_to_iter_reverted_range_and_positive() {
        assert_eq!(range_to_iter(10, 0, 1).collect::<Vec<_>>(), (10..=0).collect::<Vec<_>>());
        assert_eq!(range_to_iter(10, 0, 2).collect::<Vec<_>>(), (10..=0).step_by(2).collect::<Vec<_>>());
    }

    #[test]
    fn test_range_to_iter_reverted_range_and_negative() {
        assert_eq!(range_to_iter(10, 0, -1).collect::<Vec<_>>(), (10..=0).rev().collect::<Vec<_>>());
        assert_eq!(range_to_iter(10, 0, -2).collect::<Vec<_>>(), (10..=0).rev().step_by(2).collect::<Vec<_>>());
    }

    #[test]
    fn test_range_to_iter_zero_step() {
        assert_eq!(Some(0), range_to_iter(0, 0, 0).next());
        assert_eq!(range_to_iter(0, 1, 0).take(10).collect::<Vec<_>>(), vec![0; 10].into_iter().collect::<Vec<_>>());
        assert_eq!(range_to_iter(0, 1, 0).take(100).collect::<Vec<_>>(), vec![0; 100].into_iter().collect::<Vec<_>>());
    }
}
