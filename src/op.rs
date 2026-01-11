use crate::config::{is_nocase, Config};
use crate::err::RpErr;
use crate::input::Item;
use crate::pipe::Pipe;
use crate::{Float, Integer, PipeRes};
use cmd_help::CmdHelp;
use itertools::Itertools;
use ordered_float::OrderedFloat;
use rand::seq::SliceRandom;
use std::borrow::Cow;
use std::cmp::Reverse;
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::Write;
use unicase::UniCase;

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum PeekTo {
    StdOut,
    File { file: String, append: bool, crlf: Option<bool> },
}

#[derive(Debug, PartialEq)]
pub(crate) enum SortBy {
    Num(Option<Integer>, Option<Float>),
    Text(bool /*nocase*/),
    Random,
}

#[derive(Debug, PartialEq, CmdHelp)]
pub(crate) enum Op {
    /* **************************************** 访问 **************************************** */
    /// :peek       打印每个值到标准输出或文件。
    ///             :peek[ <file_name>]
    ///                 <file_name> 文件路径，可选。
    ///             例如：
    ///                 :peek
    ///                 :peek file.txt
    Peek(PeekTo),
    /* **************************************** 转换 **************************************** */
    /// :upper      转为ASCII大写。
    Upper,
    /// :lower      转为ASCII小写。
    Lower,
    /// :case       切换ASCII大小写。
    Case,
    /// :replace    替换字符串。
    ///             数值类型元素当作字符串进行替换，替换后转为字符串。
    ///             :replace <from> <to>[ <count>][ nocase]
    ///                 <from>  待替换的字符串，必选。
    ///                 <to>    待替换为的字符串，必选。
    ///                 <count> 对每个元素需要替换的次数，不能为负数，可选，未指定则替换所有。
    ///                 nocase  替换时忽略大小写，可选，未指定时不忽略大小写。
    ///             例如：
    ///                 :replace abc xyz
    ///                 :replace abc xyz 10
    ///                 :replace abc xyz nocase
    ///                 :replace abc xyz 10 nocase
    Replace { from: String, to: String, count: Option<usize>, nocase: bool },
    /* **************************************** 减少 **************************************** */
    /// :uniq       去重。
    ///             数值类型元素当作字符串处理，但是去重后仍为原数值类型。
    ///             :uniq[ nocase]
    ///                 nocase  去重时忽略大小写，可选，未指定时不忽略大小写。
    ///             例如：
    ///                 :uniq
    ///                 :uniq nocase
    Uniq { nocase: bool },
    /// :join       合并数据。
    ///             数值类型元素当作字符串处理，支持按照数量分组合并。
    ///             :join<[ <delimiter>[ <prefix>[ <postfix>[ <size>]]]]
    ///             <delimiter> 分隔字符串，可选。
    ///             <prefix>    前缀字符串，可选。
    ///                         指定前缀字符串时必须指定分割字符串。
    ///             <postfix>   后缀字符串，可选。
    ///                         指定后缀字符串时必须指定分割字符串和前缀字符串。
    ///             <size>      分组大小，必须为正整数，可选，未指定时所有数据为一组。
    ///                         指定分组大小时必须指定分隔字符串、前缀字符串和后缀字符串。
    ///             例如：
    ///                 :join ,
    ///                 :join , [ ]
    ///                 :join , [ ] 3
    Join { join_info: JoinInfo, count: Option<usize> },
    // /// 丢弃：
    // /// ```
    // /// :drop
    // /// ```
    // Drop,
    // /* **************************************** 增加 **************************************** */
    /* **************************************** 调整位置 **************************************** */
    /// :sort       排序。
    ///             :sort[ num [<default>]][ nocase][ desc][ random]
    ///                 num         按照数值排序，可选，未指定时按照字典序排序。
    ///                             尝试将文本解析为数值后排序，无法解析的按照<default>排序。
    ///                 <default>   仅按照数值排序时生效，无法解析为数值的文本的默认数值，可选，
    ///                             未指定时按照数值最大值处理。
    ///                 nocase      忽略大小写，仅按字典序排序时生效，可选，未指定时不忽略大小写。
    ///                 desc        逆序排序，可选，未指定时正序排序。
    ///                 random      随机排序，与按照数值排序和字典序排序互斥，且不支持逆序。
    ///             例如：
    ///                 :sort
    ///                 :sort desc
    ///                 :sort nocase
    ///                 :sort nocase desc
    ///                 :sort num
    ///                 :sort num desc
    ///                 :sort num 10
    ///                 :sort num 10 desc
    ///                 :sort num 10.5
    ///                 :sort num 10.5 desc
    ///                 :sort random
    Sort { sort_by: SortBy, desc: bool },
}

impl Op {
    pub(crate) fn new_peek(peek_to: PeekTo) -> Op {
        Op::Peek(peek_to)
    }
    pub(crate) fn new_upper() -> Op {
        Op::Upper
    }
    pub(crate) fn new_lower() -> Op {
        Op::Lower
    }
    pub(crate) fn new_case() -> Op {
        Op::Case
    }
    pub(crate) fn new_replace(from: String, to: String, count: Option<usize>, nocase: bool) -> Op {
        Op::Replace { from, to, count, nocase }
    }
    pub(crate) fn new_uniq(nocase: bool) -> Op {
        Op::Uniq { nocase }
    }
    pub(crate) fn new_join(join_info: JoinInfo, count: Option<usize>) -> Op {
        Op::Join { join_info, count }
    }
    pub(crate) fn new_sort(sort_by: SortBy, desc: bool) -> Op {
        Op::Sort { sort_by, desc }
    }

    pub(crate) fn wrap(self, mut pipe: Pipe, configs: &'static [Config]) -> PipeRes {
        match self {
            Op::Peek(peek) => match peek {
                PeekTo::StdOut => Ok(pipe.op_inspect(|item| println!("{item}"))),
                PeekTo::File { file, append, crlf } => {
                    match OpenOptions::new().write(true).truncate(!append).append(append).create(true).open(&file) {
                        Ok(mut writer) => {
                            let postfix = if crlf.unwrap_or(false) { "\r\n" } else { "\n" };
                            Ok(pipe.op_inspect(move |item| {
                                if let Err(err) = write!(writer, "{item}{postfix}") {
                                    RpErr::WriteToFileErr {
                                        file: file.clone(),
                                        item: item.to_string(),
                                        err: err.to_string(),
                                    }
                                    .termination()
                                }
                            }))
                        }
                        Err(err) => RpErr::OpenFileErr { file, err: err.to_string() }.termination(),
                    }
                }
            },
            Op::Upper => Ok(pipe.op_map(|mut item| match &mut item {
                // OPT 2026-12-29 01:24 Pipe增加属性以优化重复大小写。
                Item::String(string) => {
                    if string.chars().all(|c| c.is_ascii_uppercase()) {
                        item
                    } else {
                        string.make_ascii_uppercase();
                        item
                    }
                }
                Item::Integer(_) => item,
            })),
            Op::Lower => Ok(pipe.op_map(|mut item| match &mut item {
                // OPT 2026-12-29 01:24 Pipe增加属性以优化重复大小写。
                Item::String(string) => {
                    if string.chars().all(|c| c.is_ascii_lowercase()) {
                        item
                    } else {
                        string.make_ascii_lowercase();
                        item
                    }
                }
                Item::Integer(_) => item,
            })),
            Op::Case => {
                Ok(pipe.op_map(|mut item| match &mut item {
                    Item::String(string) => {
                        // 只修改ASCII字母（范围A-Z/a-z），而ASCII字符在UTF-8中就是单字节，
                        // 且切换大小写后仍是合法ASCII（从而合法UTF-8）。
                        for b in unsafe { string.as_bytes_mut() } {
                            match b {
                                b'A'..=b'Z' => *b += b'a' - b'A',
                                b'a'..=b'z' => *b -= b'a' - b'A',
                                _ => {}
                            }
                        }
                        item
                    }
                    Item::Integer(_) => item,
                }))
            }
            Op::Replace { from, to, count, nocase } => {
                if count == Some(0) {
                    Ok(pipe)
                } else {
                    Ok(pipe.op_map(move |item| match &item {
                        Item::String(string) => {
                            let cow =
                                replace_with_count_and_nocase(string, &*from, &*to, count, is_nocase(nocase, configs));
                            match cow {
                                Cow::Borrowed(_) => item,
                                Cow::Owned(string) => Item::String(string),
                            }
                        }
                        Item::Integer(_) => {
                            let string = item.to_string();
                            let cow =
                                replace_with_count_and_nocase(&string, &*from, &*to, count, is_nocase(nocase, configs));
                            match cow {
                                Cow::Borrowed(_) => item,
                                Cow::Owned(string) => Item::String(string),
                            }
                        }
                    }))
                }
            }
            Op::Uniq { nocase } => {
                let mut seen = HashSet::new();
                Ok(pipe.op_filter(move |item| {
                    let key = match item {
                        Item::String(s) => {
                            if is_nocase(nocase, configs) {
                                s.to_ascii_uppercase()
                            } else {
                                s.clone()
                            }
                        }
                        Item::Integer(_) => item.to_string(),
                    };
                    seen.insert(key) // 返回 true 表示保留（首次出现）
                }))
            }
            Op::Join { join_info, count } => {
                if let Some(count) = count {
                    if count > 0 {
                        return Ok(Pipe { iter: Box::new(ChunkJoin { source: pipe, group_size: count, join_info }) });
                    } else {
                        unreachable!("join count must be greater than zero");
                    }
                }
                Ok(Pipe {
                    iter: Box::new(std::iter::once(Item::String(format!(
                        "{}{}{}",
                        join_info.prefix,
                        pipe.join(&join_info.delimiter),
                        join_info.postfix
                    )))),
                })
            }
            Op::Sort { sort_by, desc } => match sort_by {
                SortBy::Num(def_integer, def_float) => {
                    if let Some(def) = def_integer {
                        let key_fn = move |item: &Item| Integer::try_from(item).unwrap_or(def);
                        let new_pipe = if desc {
                            pipe.sorted_by_key(|item| Reverse(key_fn(item)))
                        } else {
                            pipe.sorted_by_key(key_fn)
                        };
                        return Ok(Pipe { iter: Box::new(new_pipe) });
                    }
                    let def = def_float.unwrap_or(Float::MAX); // 默认按照浮点最大值
                    let key_fn = move |item: &Item| OrderedFloat(Float::try_from(item).unwrap_or(def));
                    let new_pipe = if desc {
                        pipe.sorted_by_key(|item| Reverse(key_fn(item)))
                    } else {
                        pipe.sorted_by_key(key_fn)
                    };
                    Ok(Pipe { iter: Box::new(new_pipe) })
                }
                SortBy::Text(nocase) => {
                    // TODO 2026-01-08 02:34 使用UniCase优化其他nocase场景
                    let iter = if is_nocase(nocase, configs) {
                        if desc {
                            pipe.sorted_by_key(|item| Reverse(UniCase::new(item.to_string())))
                        } else {
                            pipe.sorted_by_key(|item| UniCase::new(item.to_string()))
                        }
                    } else {
                        if desc {
                            pipe.sorted_by_key(|item| Reverse(item.to_string()))
                        } else {
                            pipe.sorted_by_key(|item| item.to_string())
                        }
                    };
                    Ok(Pipe { iter: Box::new(iter) })
                }
                SortBy::Random => {
                    let mut v = pipe.collect::<Vec<_>>();
                    v.shuffle(&mut rand::rng());
                    Ok(Pipe { iter: Box::new(v.into_iter()) })
                }
            },
        }
    }
}

/// 替换字符串
///
/// # Arguments
/// * `token` - 原始字符串
/// * `from` - 要被替换的子串
/// * `to` - 替换后的字符串
/// * `count` - 替换次数
/// * `nocase` - 是否忽略大小写
///
/// # Returns
/// 返回替换后的字符串（如果无替换发生，返回原字符串的引用以避免分配）
fn replace_with_count_and_nocase<'a>(
    text: &'a str, from: &str, to: &str, count: Option<usize>, nocase: bool,
) -> Cow<'a, str> {
    let mut result = String::new();
    let mut last_end = 0;
    let mut replaced_count = 0;
    let max_replacements = count.unwrap_or(usize::MAX);

    let (lower_text_holder, lower_from_holder); // 保持下方的&str引用有效
    // 根据是否忽略大小写选择匹配函数
    let (actual_text, actual_from) = if nocase {
        lower_text_holder = text.to_lowercase();
        lower_from_holder = from.to_lowercase();
        (&lower_text_holder as &str, &lower_from_holder as &str)
    } else {
        (text, from)
    };

    let matches = actual_text.match_indices(actual_from);
    for (start, end) in matches {
        if replaced_count >= max_replacements {
            break;
        }
        result.push_str(&text[last_end..start]); // 添加从上一个结束位置到当前匹配开始位置的文本
        result.push_str(to); // 添加替换文本
        last_end = start + end.len();
        replaced_count += 1;
    }

    if replaced_count == 0 {
        Cow::Borrowed(text) // 无替换发生，直接返回原字符串
    } else {
        result.push_str(&text[last_end..]); // 添加剩余文本
        Cow::Owned(result)
    }
}

#[derive(Debug, Eq, PartialEq, Default)]
pub(crate) struct JoinInfo {
    pub(crate) delimiter: String,
    pub(crate) prefix: String,
    pub(crate) postfix: String,
}

struct ChunkJoin<I> {
    source: I,
    group_size: usize,
    join_info: JoinInfo,
}

impl<I> Iterator for ChunkJoin<I>
where
    I: Iterator<Item = Item>,
{
    type Item = Item;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chunk = Vec::with_capacity(self.group_size);
        for _ in 0..self.group_size {
            if let Some(item) = self.source.next() {
                chunk.push(String::from(item));
            } else {
                break;
            }
        }
        if chunk.is_empty() {
            None
        } else {
            Some(Item::String(format!(
                "{}{}{}",
                self.join_info.prefix,
                chunk.join(&self.join_info.delimiter),
                self.join_info.postfix
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_with_count_and_nocase() {
        assert_eq!(replace_with_count_and_nocase("abc ABC abc abc", "abc", "1234", None, false), "1234 ABC 1234 1234");
        assert_eq!(replace_with_count_and_nocase("abc ABC abc abc", "AbC", "1234", None, true), "1234 1234 1234 1234");
        assert_eq!(replace_with_count_and_nocase("abc ABC abc abc", "abc", "1234", Some(0), false), "abc ABC abc abc");
        assert_eq!(replace_with_count_and_nocase("abc ABC abc abc", "aBc", "1234", Some(0), true), "abc ABC abc abc");
        assert_eq!(
            replace_with_count_and_nocase("abc ABC abc abc", "abc", "1234", Some(2), false),
            "1234 ABC 1234 abc"
        );
        assert_eq!(replace_with_count_and_nocase("abc ABC abc abc", "abc", "1234", Some(2), true), "1234 1234 abc abc");
        assert_eq!(
            replace_with_count_and_nocase("abc ABC abc abc", "", "1234", Some(2), true),
            "1234a1234bc ABC abc abc"
        );
        assert_eq!(replace_with_count_and_nocase("abc", "", "_", None, true), "_a_b_c_");
    }
}
