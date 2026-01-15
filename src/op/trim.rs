use crate::config::{is_nocase, Config};
use std::collections::HashSet;

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum TrimMode {
    All,
    Left,
    Right,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct TrimArg {
    trim_mode: TrimMode,
    /// 需要去除的内容。
    /// 出于优化目的：如果nocase，则必须为小写；如果char_mode则必须去重。
    pattern: Option<String>,
    /// 去除字串还是字符
    char_mode: bool,
    nocase: bool,
}

impl TrimArg {
    pub(crate) fn new(trim_mode: TrimMode, pattern: Option<String>, char_mode: bool, nocase: bool) -> TrimArg {
        TrimArg {
            trim_mode,
            pattern: {
                let pattern = if nocase {
                    pattern.map(|mut s| {
                        s.make_ascii_lowercase();
                        s
                    })
                } else {
                    pattern
                };
                if char_mode {
                    pattern.map(|s| {
                        let mut seen = HashSet::new();
                        s.chars().filter(|&c| seen.insert(c)).collect()
                    })
                } else {
                    pattern
                }
            },
            char_mode,
            nocase,
        }
    }

    pub(crate) fn trim(&self, to_trim: String, configs: &[Config]) -> String {
        let trimmed = if let Some(pattern) = &self.pattern
            && !pattern.is_empty()
        {
            if self.char_mode {
                if is_nocase(self.nocase, configs) {
                    match self.trim_mode {
                        TrimMode::All => {
                            Self::trim_end_char_nocase(Self::trim_start_char_nocase(&to_trim, pattern), pattern)
                        }
                        TrimMode::Left => Self::trim_start_char_nocase(&to_trim, pattern),
                        TrimMode::Right => Self::trim_end_char_nocase(&to_trim, pattern),
                    }
                } else {
                    match self.trim_mode {
                        TrimMode::All => Self::trim_end_char(Self::trim_start_char(&to_trim, pattern), pattern),
                        TrimMode::Left => Self::trim_start_char(&to_trim, pattern),
                        TrimMode::Right => Self::trim_end_char(&to_trim, pattern),
                    }
                }
            } else {
                if is_nocase(self.nocase, configs) {
                    match self.trim_mode {
                        TrimMode::All => {
                            Self::trim_end_str_nocase(Self::trim_start_str_nocase(&to_trim, pattern), pattern)
                        }
                        TrimMode::Left => Self::trim_start_str_nocase(&to_trim, pattern),
                        TrimMode::Right => Self::trim_end_str_nocase(&to_trim, pattern),
                    }
                } else {
                    match self.trim_mode {
                        TrimMode::All => {
                            let stripped = to_trim.strip_prefix(pattern).unwrap_or(&to_trim);
                            stripped.strip_suffix(pattern).unwrap_or(stripped)
                        }
                        TrimMode::Left => to_trim.strip_prefix(pattern).unwrap_or(&to_trim),
                        TrimMode::Right => to_trim.strip_suffix(pattern).unwrap_or(&to_trim),
                    }
                }
            }
        } else {
            to_trim.trim()
        };
        if trimmed == &to_trim { to_trim } else { trimmed.to_owned() }
    }

    fn trim_start_str_nocase<'a>(to_trim: &'a str, pattern: &'a str) -> &'a str {
        let mut to_trim_chars = to_trim.char_indices();
        let mut pattern_chars = pattern.chars();
        loop {
            match (to_trim_chars.next(), pattern_chars.next()) {
                (Some((_, tc)), Some(pc)) => {
                    if tc.to_ascii_lowercase() != pc {
                        return to_trim; // 匹配失败，不截取
                    }
                }
                (None, Some(_)) => return to_trim,            // to_trim太短，不截取
                (Some((i, _)), None) => return &to_trim[i..], // 匹配完成
                (None, None) => return "",                    // 完全匹配，全部截取
            }
        }
    }

    fn trim_end_str_nocase<'a>(to_trim: &'a str, pattern: &'a str) -> &'a str {
        let mut to_trim_chars = to_trim.char_indices().rev();
        let mut pattern_chars = pattern.chars().rev();
        loop {
            match (to_trim_chars.next(), pattern_chars.next()) {
                (Some((_, tc)), Some(pc)) => {
                    if tc.to_ascii_lowercase() != pc {
                        return to_trim; // 匹配失败，不截取
                    }
                }
                (None, Some(_)) => return to_trim, // to_trim太短，不截取
                (Some((i, tc)), None) => return &to_trim[..(i + tc.len_utf8())], // 匹配完成
                (None, None) => return "",         // 完全匹配，全部截取
            }
        }
    }

    fn trim_start_char_nocase<'a>(to_trim: &'a str, pattern: &str) -> &'a str {
        let mut start_idx = 0;
        for ch in to_trim.chars() {
            if pattern.chars().any(|p| p.eq(&ch.to_ascii_lowercase())) {
                start_idx += ch.len_utf8();
            } else {
                break;
            }
        }
        &to_trim[start_idx..]
    }

    fn trim_end_char_nocase<'a>(to_trim: &'a str, pattern: &str) -> &'a str {
        let mut end_idx = to_trim.len();

        for ch in to_trim.chars().rev() {
            if pattern.chars().any(|p| p.eq(&ch.to_ascii_lowercase())) {
                end_idx -= ch.len_utf8();
            } else {
                break;
            }
        }

        &to_trim[..end_idx]
    }

    fn trim_start_char<'a>(to_trim: &'a str, pattern: &'a str) -> &'a str {
        let start = to_trim.char_indices().find(|(_, c)| !pattern.contains(*c)).map_or(to_trim.len(), |(i, _)| i);
        if start == to_trim.len() { "" } else { &to_trim[start..] }
    }

    fn trim_end_char<'a>(to_trim: &'a str, pattern: &'a str) -> &'a str {
        let end = to_trim.char_indices().rfind(|(_, c)| !pattern.contains(*c)).map_or(0, |(i, c)| i + c.len_utf8());
        if end == 0 { "" } else { &to_trim[..end] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_char_nocase() {
        let configs = vec![];
        // left
        assert_eq!("abc", TrimArg::new(TrimMode::Left, None, true, true).trim("abc".to_owned(), &configs));
        assert_eq!(
            "abc123abc",
            TrimArg::new(TrimMode::Left, Some("_;+-=".to_owned()), true, true).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "23ABC",
            TrimArg::new(TrimMode::Left, Some("cBAa1".to_owned()), true, true).trim("abc123ABC".to_owned(), &configs)
        );
        assert_eq!(
            "啊你好",
            TrimArg::new(TrimMode::Left, Some("你好好".to_owned()), true, true)
                .trim("你好你好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "1c好啊你好",
            TrimArg::new(TrimMode::Left, Some("你好aBc".to_owned()), true, true)
                .trim("你a好b你c1c好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "",
            TrimArg::new(TrimMode::Left, Some("你好啊abc".to_owned()), true, true).trim("a你".to_owned(), &configs)
        );
        // right
        assert_eq!("abc", TrimArg::new(TrimMode::Right, None, true, true).trim("abc".to_owned(), &configs));
        assert_eq!(
            "abc123abc",
            TrimArg::new(TrimMode::Right, Some("_;+-=".to_owned()), true, true).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "abc123",
            TrimArg::new(TrimMode::Right, Some("cBAa1".to_owned()), true, true).trim("abc123ABC".to_owned(), &configs)
        );
        assert_eq!(
            "你好你好啊",
            TrimArg::new(TrimMode::Right, Some("你好好".to_owned()), true, true)
                .trim("你好你好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "你a好b你c1c好啊",
            TrimArg::new(TrimMode::Right, Some("你好aBc".to_owned()), true, true)
                .trim("你a好b你c1c好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "",
            TrimArg::new(TrimMode::Right, Some("你好啊abc".to_owned()), true, true).trim("a你".to_owned(), &configs)
        );
        // all
        assert_eq!("abc", TrimArg::new(TrimMode::All, None, true, true).trim("abc".to_owned(), &configs));
        assert_eq!(
            "abc123abc",
            TrimArg::new(TrimMode::All, Some("_;+-=".to_owned()), true, true).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "23",
            TrimArg::new(TrimMode::All, Some("cBAa1".to_owned()), true, true).trim("abc123ABC".to_owned(), &configs)
        );
        assert_eq!(
            "啊",
            TrimArg::new(TrimMode::All, Some("你好好".to_owned()), true, true)
                .trim("你好你好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "1c好啊",
            TrimArg::new(TrimMode::All, Some("你好aBc".to_owned()), true, true)
                .trim("你a好b你c1c好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "",
            TrimArg::new(TrimMode::All, Some("你好啊abc".to_owned()), true, true).trim("a你".to_owned(), &configs)
        );
    }

    #[test]
    fn test_trim_char() {
        let configs = vec![];
        // left
        assert_eq!("abc", TrimArg::new(TrimMode::Left, None, true, false).trim("abc".to_owned(), &configs));
        assert_eq!(
            "abc123abc",
            TrimArg::new(TrimMode::Left, Some("_;+-=".to_owned()), true, false).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "23aBc",
            TrimArg::new(TrimMode::Left, Some("aBc1".to_owned()), true, false).trim("acB123aBc".to_owned(), &configs)
        );
        assert_eq!(
            "啊你好",
            TrimArg::new(TrimMode::Left, Some("你好好".to_owned()), true, false)
                .trim("你好你好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "b你c1c好啊你好",
            TrimArg::new(TrimMode::Left, Some("你好aBc".to_owned()), true, false)
                .trim("你a好b你c1c好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "",
            TrimArg::new(TrimMode::Left, Some("你好啊abc".to_owned()), true, false).trim("a你".to_owned(), &configs)
        );
        // right
        assert_eq!("abc", TrimArg::new(TrimMode::Right, None, true, false).trim("abc".to_owned(), &configs));
        assert_eq!(
            "abc123abc",
            TrimArg::new(TrimMode::Right, Some("_;+-=".to_owned()), true, false).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "abc123ab",
            TrimArg::new(TrimMode::Right, Some("aBc1".to_owned()), true, false).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "你好你好啊",
            TrimArg::new(TrimMode::Right, Some("你好好".to_owned()), true, false)
                .trim("你好你好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "你a好b你c1c好啊",
            TrimArg::new(TrimMode::Right, Some("你好aBc".to_owned()), true, false)
                .trim("你a好b你c1c好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "",
            TrimArg::new(TrimMode::Right, Some("你好啊abc".to_owned()), true, false).trim("a你".to_owned(), &configs)
        );
        // all
        assert_eq!("abc", TrimArg::new(TrimMode::All, None, true, false).trim("abc".to_owned(), &configs));
        assert_eq!(
            "abc123abc",
            TrimArg::new(TrimMode::All, Some("_;+-=".to_owned()), true, false).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "bc123ab",
            TrimArg::new(TrimMode::All, Some("aBc1".to_owned()), true, false).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "啊",
            TrimArg::new(TrimMode::All, Some("你好好".to_owned()), true, false)
                .trim("你好你好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "b你c1c好啊",
            TrimArg::new(TrimMode::All, Some("你好aBc".to_owned()), true, false)
                .trim("你a好b你c1c好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "",
            TrimArg::new(TrimMode::All, Some("你好啊abc".to_owned()), true, false).trim("a你".to_owned(), &configs)
        );
    }

    #[test]
    fn test_trim_str_nocase() {
        let configs = vec![];
        // left
        assert_eq!("abc", TrimArg::new(TrimMode::Left, None, false, true).trim("abc".to_owned(), &configs));
        assert_eq!(
            "abc123abc",
            TrimArg::new(TrimMode::Left, Some("_;+-=".to_owned()), false, true).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "abc123abCABC",
            TrimArg::new(TrimMode::Left, Some("abc".to_owned()), false, true)
                .trim("abcabc123abCABC".to_owned(), &configs)
        );
        assert_eq!(
            "123aBc",
            TrimArg::new(TrimMode::Left, Some("acB".to_owned()), false, true).trim("acB123aBc".to_owned(), &configs)
        );
        assert_eq!(
            "好啊你好",
            TrimArg::new(TrimMode::Left, Some("你好你".to_owned()), false, true)
                .trim("你好你好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "你a好b你c1c好啊你好",
            TrimArg::new(TrimMode::Left, Some("你好aBc".to_owned()), false, true)
                .trim("你a好b你c1c好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "啊你好你好aBc",
            TrimArg::new(TrimMode::Left, Some("你好aBc".to_owned()), false, true)
                .trim("你好aBc啊你好你好aBc".to_owned(), &configs)
        );
        assert_eq!(
            "a你",
            TrimArg::new(TrimMode::Left, Some("你好啊abc".to_owned()), false, true).trim("a你".to_owned(), &configs)
        );
        // right
        assert_eq!("abc", TrimArg::new(TrimMode::Right, None, false, true).trim("abc".to_owned(), &configs));
        assert_eq!(
            "abc123abc",
            TrimArg::new(TrimMode::Right, Some("_;+-=".to_owned()), false, true).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "abcabc123abC",
            TrimArg::new(TrimMode::Right, Some("abc".to_owned()), false, true)
                .trim("abcabc123abCABC".to_owned(), &configs)
        );
        assert_eq!(
            "abc123abc",
            TrimArg::new(TrimMode::Right, Some("aBc1".to_owned()), false, true).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "你好你好啊你好",
            TrimArg::new(TrimMode::Right, Some("你好你".to_owned()), false, true)
                .trim("你好你好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "你a好b你c1c好啊你好",
            TrimArg::new(TrimMode::Right, Some("你好aBc".to_owned()), false, true)
                .trim("你a好b你c1c好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "你好aBc啊你好",
            TrimArg::new(TrimMode::Right, Some("你好aBc".to_owned()), false, true)
                .trim("你好aBc啊你好你好aBc".to_owned(), &configs)
        );
        assert_eq!(
            "a你",
            TrimArg::new(TrimMode::Right, Some("你好啊abc".to_owned()), false, true).trim("a你".to_owned(), &configs)
        );
        // all
        assert_eq!("abc", TrimArg::new(TrimMode::All, None, false, true).trim("abc".to_owned(), &configs));
        assert_eq!(
            "abc123abc",
            TrimArg::new(TrimMode::All, Some("_;+-=".to_owned()), false, true).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "abc123abC",
            TrimArg::new(TrimMode::All, Some("abc".to_owned()), false, true)
                .trim("abcabc123abCABC".to_owned(), &configs)
        );
        assert_eq!(
            "23abc",
            TrimArg::new(TrimMode::All, Some("aBc1".to_owned()), false, true).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "好啊你好",
            TrimArg::new(TrimMode::All, Some("你好你".to_owned()), false, true)
                .trim("你好你好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "你a好b你c1c好啊你好",
            TrimArg::new(TrimMode::All, Some("你好aBc".to_owned()), false, true)
                .trim("你a好b你c1c好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "啊你好",
            TrimArg::new(TrimMode::All, Some("你好aBc".to_owned()), false, true)
                .trim("你好aBc啊你好你好aBc".to_owned(), &configs)
        );
        assert_eq!(
            "a你",
            TrimArg::new(TrimMode::All, Some("你好啊abc".to_owned()), false, true).trim("a你".to_owned(), &configs)
        );
    }

    #[test]
    fn test_trim_str() {
        let configs = vec![];
        // left
        assert_eq!("abc", TrimArg::new(TrimMode::Left, None, false, false).trim("abc".to_owned(), &configs));
        assert_eq!(
            "abc123abc",
            TrimArg::new(TrimMode::Left, Some("_;+-=".to_owned()), false, false).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "aBcabc123abcabc",
            TrimArg::new(TrimMode::Left, Some("abc".to_owned()), false, false)
                .trim("aBcabc123abcabc".to_owned(), &configs)
        );
        assert_eq!(
            "123acb",
            TrimArg::new(TrimMode::Left, Some("acB".to_owned()), false, false).trim("acB123acb".to_owned(), &configs)
        );
        assert_eq!(
            "好啊你好",
            TrimArg::new(TrimMode::Left, Some("你好你".to_owned()), false, false)
                .trim("你好你好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "你a好b你c1c好啊你好",
            TrimArg::new(TrimMode::Left, Some("你好aBc".to_owned()), false, false)
                .trim("你a好b你c1c好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "啊你好你好abc",
            TrimArg::new(TrimMode::Left, Some("你好aBc".to_owned()), false, false)
                .trim("你好aBc啊你好你好abc".to_owned(), &configs)
        );
        assert_eq!(
            "a你",
            TrimArg::new(TrimMode::Left, Some("你好啊abc".to_owned()), false, false).trim("a你".to_owned(), &configs)
        );
        // right
        assert_eq!("abc", TrimArg::new(TrimMode::Right, None, false, false).trim("abc".to_owned(), &configs));
        assert_eq!(
            "abc123abc",
            TrimArg::new(TrimMode::Right, Some("_;+-=".to_owned()), false, false)
                .trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "aBcabc123abc",
            TrimArg::new(TrimMode::Right, Some("abc".to_owned()), false, false)
                .trim("aBcabc123abcabc".to_owned(), &configs)
        );
        assert_eq!(
            "abc123abc",
            TrimArg::new(TrimMode::Right, Some("aBc1".to_owned()), false, false).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "你好你好啊你好",
            TrimArg::new(TrimMode::Right, Some("你好你".to_owned()), false, false)
                .trim("你好你好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "你a好b你c1c好啊你好",
            TrimArg::new(TrimMode::Right, Some("你好aBc".to_owned()), false, false)
                .trim("你a好b你c1c好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "你好aBc啊你好你好abc",
            TrimArg::new(TrimMode::Right, Some("你好aBc".to_owned()), false, false)
                .trim("你好aBc啊你好你好abc".to_owned(), &configs)
        );
        assert_eq!(
            "a你",
            TrimArg::new(TrimMode::Right, Some("你好啊abc".to_owned()), false, false).trim("a你".to_owned(), &configs)
        );
        // all
        assert_eq!("abc", TrimArg::new(TrimMode::All, None, false, false).trim("abc".to_owned(), &configs));
        assert_eq!(
            "abc123abc",
            TrimArg::new(TrimMode::All, Some("_;+-=".to_owned()), false, false).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "aBcabc123abc",
            TrimArg::new(TrimMode::All, Some("abc".to_owned()), false, false)
                .trim("aBcabc123abcabc".to_owned(), &configs)
        );
        assert_eq!(
            "abc123abc",
            TrimArg::new(TrimMode::All, Some("aBc1".to_owned()), false, false).trim("abc123abc".to_owned(), &configs)
        );
        assert_eq!(
            "好啊你好",
            TrimArg::new(TrimMode::All, Some("你好你".to_owned()), false, false)
                .trim("你好你好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "你a好b你c1c好啊你好",
            TrimArg::new(TrimMode::All, Some("你好aBc".to_owned()), false, false)
                .trim("你a好b你c1c好啊你好".to_owned(), &configs)
        );
        assert_eq!(
            "啊你好你好abc",
            TrimArg::new(TrimMode::All, Some("你好aBc".to_owned()), false, false)
                .trim("你好aBc啊你好你好abc".to_owned(), &configs)
        );
        assert_eq!(
            "a你",
            TrimArg::new(TrimMode::All, Some("你好啊abc".to_owned()), false, false).trim("a你".to_owned(), &configs)
        );
    }
}
