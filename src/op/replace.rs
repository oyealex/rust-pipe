use crate::config::{is_nocase, Config};
use std::borrow::Cow;

#[derive(Debug, PartialEq)]
pub(crate) struct ReplaceArg {
    from: String, /*nocase时需要转为小写*/
    to: String,
    pub(in crate::op) count: Option<usize>,
    nocase: bool,
}

impl ReplaceArg {
    pub(crate) fn new(from: String, to: String, count: Option<usize>, nocase: bool) -> Self {
        Self { from: if nocase { from.to_ascii_lowercase() } else { from }, to, count, nocase }
    }

    /// 替换字符串
    ///
    /// # Arguments
    /// * `token` - 原始字符串
    /// * `from` - 要被替换的子串
    ///
    /// # Returns
    /// 返回替换后的字符串（如果无替换发生，返回原字符串的引用以避免分配）
    pub(crate) fn replace<'a>(&self, text: &'a str, configs: &[Config]) -> Cow<'a, str> {
        let mut result = String::new();
        let mut last_end = 0;
        let mut replaced_count = 0;
        let max_replacements = self.count.unwrap_or(usize::MAX);

        let lower_text_holder; // 保持下方的&str引用有效
        // 根据是否忽略大小写选择匹配函数
        let actual_text = if is_nocase(self.nocase, configs) {
            lower_text_holder = text.to_ascii_lowercase();
            &lower_text_holder as &str
        } else {
            text
        };

        let matches = actual_text.match_indices(&self.from);
        for (start, end) in matches {
            if replaced_count >= max_replacements {
                break;
            }
            result.push_str(&text[last_end..start]); // 添加从上一个结束位置到当前匹配开始位置的文本
            result.push_str(&self.to); // 添加替换文本
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_with_count_and_nocase() {
        let config = vec![];
        assert_eq!(
            ReplaceArg::new("abc".to_owned(), "1234".to_owned(), None, false).replace("abc ABC abc abc", &config),
            "1234 ABC 1234 1234"
        );
        assert_eq!(
            ReplaceArg::new("AbC".to_owned(), "1234".to_owned(), None, true).replace("abc ABC abc abc", &config),
            "1234 1234 1234 1234"
        );
        assert_eq!(
            ReplaceArg::new("abc".to_owned(), "1234".to_owned(), Some(0), false).replace("abc ABC abc abc", &config),
            "abc ABC abc abc"
        );
        assert_eq!(
            ReplaceArg::new("aBc".to_owned(), "1234".to_owned(), Some(0), true).replace("abc ABC abc abc", &config),
            "abc ABC abc abc"
        );
        assert_eq!(
            ReplaceArg::new("abc".to_owned(), "1234".to_owned(), Some(2), false).replace("abc ABC abc abc", &config),
            "1234 ABC 1234 abc"
        );
        assert_eq!(
            ReplaceArg::new("abc".to_owned(), "1234".to_owned(), Some(2), true).replace("abc ABC abc abc", &config),
            "1234 1234 abc abc"
        );
        assert_eq!(
            ReplaceArg::new("".to_owned(), "1234".to_owned(), Some(2), true).replace("abc ABC abc abc", &config),
            "1234a1234bc ABC abc abc"
        );
        assert_eq!(ReplaceArg::new("".to_owned(), "_".to_owned(), None, true).replace("abc", &config), "_a_b_c_");
        assert_eq!(
            ReplaceArg::new("你".to_owned(), "_".to_owned(), None, true).replace("abc你好世界，你好！", &config),
            "abc_好世界，_好！"
        );
    }
}
