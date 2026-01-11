use crate::err::RpErr;
use crate::{Float, Integer};
use cmd_help::CmdHelp;
use regex::Regex;

#[derive(Debug, CmdHelp)]
pub(crate) enum Cond {
    /// len [!][<min_len>],[<max_len>]
    ///     按照字符串长度范围选择，范围表达式最小值和最大值均为包含且可选，支持可选的否定。
    TextLenRange { range: (Option<usize>, Option<usize>), not: bool },
    /// len [!]=<len>
    ///     按照字符串特定长度选择，支持可选的否定。
    TextLenSpec { len: usize, not: bool },
    /// num [!][<min_integer>],[<max_integer>]
    ///     按照整数值范围选择，范围表达式最小值和最大值均为包含且可选，支持可选的否定。
    ///     如果无法解析为整数则不选择。
    IntegerRange { range: (Option<Integer>, Option<Integer>), not: bool },
    /// num [!]=<integer>
    ///     按照整数值特定值选择，支持可选的否定。
    ///     如果无法解析为整数则不选择。
    IntegerSpec { val: Integer, not: bool },
    /// num [!][<min_float>],[<max_float>]
    ///     按照浮点数值范围选择，范围表达式最小值和最大值均为包含且可选，支持可选的否定。
    ///     如果无法解析为浮点数则不选择。
    FloatRange { range: (Option<Float>, Option<Float>), not: bool },
    /// num [!]=<float>
    ///     按照浮点数值特定值选择，支持可选的否定。
    ///     如果无法解析为浮点数则不选择。
    FloatSpec { val: Float, not: bool },
    /// num ![integer|float]
    ///     按照非整数或非浮点数选择，如果不指定则选择非数值数据。
    NumberNot(Option<bool> /*integer*/),
    /// upper|lower
    ///     选择全部为大写或小写字符的数据，不支持大小写的字符总是满足。
    TextAllCase(bool /*upper?*/),
    /// empty|blank
    ///     选择没有任何字符或全部为空白字符的数据。
    TextEmptyOrBlank(bool /*empty*/),
    /// reg <reg_exp>
    ///     选择匹配给定正则表达式的数据。
    RegMatch(Regex),
}

impl Cond {
    pub(crate) fn new_text_len_range(range: (Option<usize>, Option<usize>), not: bool) -> Cond {
        Cond::TextLenRange { range, not }
    }
    pub(crate) fn new_text_len_spec(len: usize, not: bool) -> Cond {
        Cond::TextLenSpec { len, not }
    }
    pub(crate) fn new_integer_range(range: (Option<Integer>, Option<Integer>), not: bool) -> Cond {
        Cond::IntegerRange { range, not }
    }
    pub(crate) fn new_integer_spec(val: Integer, not: bool) -> Cond {
        Cond::IntegerSpec { val, not }
    }
    pub(crate) fn new_float_range(range: (Option<Float>, Option<Float>), not: bool) -> Cond {
        Cond::FloatRange { range, not }
    }
    pub(crate) fn new_float_spec(val: Float, not: bool) -> Cond {
        Cond::FloatSpec { val, not }
    }
    pub(crate) fn new_number_not(integer: Option<bool>) -> Cond {
        Cond::NumberNot(integer)
    }
    pub(crate) fn new_text_all_case(upper: bool) -> Cond {
        Cond::TextAllCase(upper)
    }
    pub(crate) fn new_text_empty_or_blank(empty: bool) -> Cond {
        Cond::TextEmptyOrBlank(empty)
    }
    pub(crate) fn new_reg_match(regex: &str) -> Result<Cond, RpErr> {
        let reg = format!(r"\A(?:{})\z", regex);
        Regex::new(&reg)
            .map(|reg| Cond::RegMatch(reg))
            .map_err(|err| RpErr::ParseRegexErr { reg, err: err.to_string() })
    }

    pub(crate) fn test(&self, input: &str) -> bool {
        match self {
            Cond::TextLenRange { range, not } => {
                let len = *&input.chars().count();
                let res =
                    range.0.map_or(true, |min_len| len >= min_len) && range.1.map_or(true, |max_len| len <= max_len);
                if *not { !res } else { res }
            }
            Cond::TextLenSpec { len, not } => {
                if *not {
                    input.chars().count() != *len
                } else {
                    input.chars().count() == *len
                }
            }
            Cond::IntegerRange { range, not } => input
                .parse::<Integer>()
                .map(|i| {
                    let res =
                        range.0.map_or(true, |min_len| i >= min_len) && range.1.map_or(true, |max_len| i <= max_len);
                    if *not { !res } else { res }
                })
                .unwrap_or(false),
            Cond::IntegerSpec { val, not } => {
                input.parse::<Integer>().ok().map(|i| if *not { &i != val } else { &i == val }).unwrap_or(false)
            }
            Cond::FloatRange { range, not } => input
                .parse::<Float>()
                .map(|f| {
                    if !f.is_finite() {
                        return false;
                    }
                    let res =
                        range.0.map_or(true, |min_len| f >= min_len) && range.1.map_or(true, |max_len| f <= max_len);
                    if *not { !res } else { res }
                })
                .unwrap_or(false),
            Cond::FloatSpec { val, not } => input
                .parse::<Float>()
                .ok()
                .map(|f| {
                    if !f.is_finite() {
                        return false;
                    }
                    if *not { &f != val } else { &f == val }
                })
                .unwrap_or(false),
            Cond::NumberNot(integer) => match integer {
                Some(integer) => {
                    if *integer {
                        !input.parse::<Integer>().is_ok()
                    } else {
                        input.parse::<Integer>().is_ok() || input.parse::<Float>().map_or(true, |v| !v.is_finite())
                    }
                }
                None => input.parse::<Float>().map_or(true, |v| !v.is_finite()),
            },
            Cond::TextAllCase(upper) => {
                if *upper {
                    !input.chars().any(|c| c.is_lowercase())
                } else {
                    !input.chars().any(|c| c.is_uppercase())
                }
            }
            Cond::TextEmptyOrBlank(empty) => {
                if *empty {
                    input.is_empty()
                } else {
                    input.chars().all(|c| c.is_whitespace())
                }
            }
            Cond::RegMatch(regex) => regex.is_match(input),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_len_range() {
        assert!(!Cond::new_text_len_range((Some(3), Some(5)), false).test("12"));
        assert!(Cond::new_text_len_range((Some(3), Some(5)), false).test("123"));
        assert!(Cond::new_text_len_range((Some(3), Some(5)), false).test("1234"));
        assert!(Cond::new_text_len_range((Some(3), Some(5)), false).test("12345"));
        assert!(!Cond::new_text_len_range((Some(3), Some(5)), false).test("123456"));
        assert!(!Cond::new_text_len_range((Some(3), None), false).test("12"));
        assert!(Cond::new_text_len_range((Some(3), None), false).test("123"));
        assert!(Cond::new_text_len_range((Some(3), None), false).test("1234"));
        assert!(Cond::new_text_len_range((None, Some(3)), false).test("12"));
        assert!(Cond::new_text_len_range((None, Some(3)), false).test("123"));
        assert!(!Cond::new_text_len_range((None, Some(3)), false).test("1234"));
        assert!(Cond::new_text_len_range((None, None), false).test("123"));
        // not
        assert!(Cond::new_text_len_range((Some(3), Some(5)), true).test("12"));
        assert!(!Cond::new_text_len_range((Some(3), Some(5)), true).test("123"));
        assert!(!Cond::new_text_len_range((Some(3), Some(5)), true).test("1234"));
        assert!(!Cond::new_text_len_range((Some(3), Some(5)), true).test("12345"));
        assert!(Cond::new_text_len_range((Some(3), Some(5)), true).test("123456"));
        assert!(Cond::new_text_len_range((Some(3), None), true).test("12"));
        assert!(!Cond::new_text_len_range((Some(3), None), true).test("123"));
        assert!(!Cond::new_text_len_range((Some(3), None), true).test("1234"));
        assert!(!Cond::new_text_len_range((None, Some(3)), true).test("12"));
        assert!(!Cond::new_text_len_range((None, Some(3)), true).test("123"));
        assert!(Cond::new_text_len_range((None, Some(3)), true).test("1234"));
        assert!(!Cond::new_text_len_range((None, None), true).test("123"));
    }

    #[test]
    fn test_text_len_spec() {
        assert!(Cond::new_text_len_spec(0, false).test(""));
        assert!(!Cond::new_text_len_spec(0, false).test("1"));
        assert!(!Cond::new_text_len_spec(3, false).test(""));
        assert!(!Cond::new_text_len_spec(3, false).test("12"));
        assert!(Cond::new_text_len_spec(3, false).test("123"));
        assert!(!Cond::new_text_len_spec(3, false).test("1234"));
        // not
        assert!(!Cond::new_text_len_spec(0, true).test(""));
        assert!(Cond::new_text_len_spec(0, true).test("1"));
        assert!(Cond::new_text_len_spec(3, true).test(""));
        assert!(Cond::new_text_len_spec(3, true).test("12"));
        assert!(!Cond::new_text_len_spec(3, true).test("123"));
        assert!(Cond::new_text_len_spec(3, true).test("1234"));
    }

    #[test]
    fn test_integer_range() {
        assert!(!Cond::new_integer_range((Some(3), Some(5)), false).test("2"));
        assert!(Cond::new_integer_range((Some(3), Some(5)), false).test("3"));
        assert!(Cond::new_integer_range((Some(3), Some(5)), false).test("4"));
        assert!(Cond::new_integer_range((Some(3), Some(5)), false).test("5"));
        assert!(!Cond::new_integer_range((Some(3), Some(5)), false).test("6"));
        assert!(!Cond::new_integer_range((Some(3), None), false).test("2"));
        assert!(Cond::new_integer_range((Some(3), None), false).test("3"));
        assert!(Cond::new_integer_range((Some(3), None), false).test("4"));
        assert!(Cond::new_integer_range((None, Some(3)), false).test("2"));
        assert!(Cond::new_integer_range((None, Some(3)), false).test("3"));
        assert!(!Cond::new_integer_range((None, Some(3)), false).test("4"));
        assert!(Cond::new_integer_range((None, None), false).test("3"));
        assert!(!Cond::new_integer_range((None, None), false).test("abc"));
        assert!(!Cond::new_integer_range((None, None), false).test(""));
        // not
        assert!(Cond::new_integer_range((Some(3), Some(5)), true).test("2"));
        assert!(!Cond::new_integer_range((Some(3), Some(5)), true).test("3"));
        assert!(!Cond::new_integer_range((Some(3), Some(5)), true).test("4"));
        assert!(!Cond::new_integer_range((Some(3), Some(5)), true).test("5"));
        assert!(Cond::new_integer_range((Some(3), Some(5)), true).test("6"));
        assert!(Cond::new_integer_range((Some(3), None), true).test("2"));
        assert!(!Cond::new_integer_range((Some(3), None), true).test("3"));
        assert!(!Cond::new_integer_range((Some(3), None), true).test("4"));
        assert!(!Cond::new_integer_range((None, Some(3)), true).test("2"));
        assert!(!Cond::new_integer_range((None, Some(3)), true).test("3"));
        assert!(Cond::new_integer_range((None, Some(3)), true).test("4"));
        assert!(!Cond::new_integer_range((None, None), true).test("3"));
        assert!(!Cond::new_integer_range((None, None), true).test("abc"));
        assert!(!Cond::new_integer_range((None, None), true).test(""));
    }

    #[test]
    fn test_integer_spec() {
        assert!(Cond::new_integer_spec(0, false).test("0"));
        assert!(!Cond::new_integer_spec(0, false).test("1"));
        assert!(!Cond::new_integer_spec(3, false).test("1"));
        assert!(Cond::new_integer_spec(3, false).test("3"));
        assert!(!Cond::new_integer_spec(3, false).test("abc"));
        assert!(!Cond::new_integer_spec(3, false).test(""));
        // not
        assert!(!Cond::new_integer_spec(0, true).test("0"));
        assert!(Cond::new_integer_spec(0, true).test("1"));
        assert!(Cond::new_integer_spec(3, true).test("1"));
        assert!(!Cond::new_integer_spec(3, true).test("3"));
        assert!(!Cond::new_integer_spec(3, true).test("abc"));
        assert!(!Cond::new_integer_spec(3, true).test(""));
    }

    #[test]
    fn test_float_range() {
        assert!(!Cond::new_float_range((Some(3.0), Some(5.0)), false).test("2"));
        assert!(Cond::new_float_range((Some(3.0), Some(5.0)), false).test("3"));
        assert!(Cond::new_float_range((Some(3.0), Some(5.0)), false).test("4"));
        assert!(Cond::new_float_range((Some(3.0), Some(5.0)), false).test("5"));
        assert!(!Cond::new_float_range((Some(3.0), Some(5.0)), false).test("6"));
        assert!(!Cond::new_float_range((Some(3.0), None), false).test("2"));
        assert!(Cond::new_float_range((Some(3.0), None), false).test("3"));
        assert!(Cond::new_float_range((Some(3.0), None), false).test("4"));
        assert!(Cond::new_float_range((None, Some(3.0)), false).test("2"));
        assert!(Cond::new_float_range((None, Some(3.0)), false).test("3"));
        assert!(!Cond::new_float_range((None, Some(3.0)), false).test("4"));
        assert!(Cond::new_float_range((None, None), false).test("3"));
        assert!(!Cond::new_float_range((None, None), false).test("abc"));
        assert!(!Cond::new_float_range((None, None), false).test("NaN"));
        assert!(!Cond::new_float_range((None, None), false).test("nan"));
        assert!(!Cond::new_float_range((None, None), false).test("inf"));
        assert!(!Cond::new_float_range((None, None), false).test("Inf"));
        assert!(!Cond::new_float_range((None, None), false).test("-inf"));
        assert!(!Cond::new_float_range((None, None), false).test("-Inf"));
        assert!(!Cond::new_float_range((None, None), false).test(""));
        // not
        assert!(Cond::new_float_range((Some(3.0), Some(5.0)), true).test("2"));
        assert!(!Cond::new_float_range((Some(3.0), Some(5.0)), true).test("3"));
        assert!(!Cond::new_float_range((Some(3.0), Some(5.0)), true).test("4"));
        assert!(!Cond::new_float_range((Some(3.0), Some(5.0)), true).test("5"));
        assert!(Cond::new_float_range((Some(3.0), Some(5.0)), true).test("6"));
        assert!(Cond::new_float_range((Some(3.0), None), true).test("2"));
        assert!(!Cond::new_float_range((Some(3.0), None), true).test("3"));
        assert!(!Cond::new_float_range((Some(3.0), None), true).test("4"));
        assert!(!Cond::new_float_range((None, Some(3.0)), true).test("2"));
        assert!(!Cond::new_float_range((None, Some(3.0)), true).test("3"));
        assert!(Cond::new_float_range((None, Some(3.0)), true).test("4"));
        assert!(!Cond::new_float_range((None, None), true).test("3"));
        assert!(!Cond::new_float_range((None, None), true).test("abc"));
        assert!(!Cond::new_float_range((None, None), true).test("NaN"));
        assert!(!Cond::new_float_range((None, None), true).test("nan"));
        assert!(!Cond::new_float_range((None, None), true).test("inf"));
        assert!(!Cond::new_float_range((None, None), true).test("Inf"));
        assert!(!Cond::new_float_range((None, None), true).test("-inf"));
        assert!(!Cond::new_float_range((None, None), true).test("-Inf"));
        assert!(!Cond::new_float_range((None, None), true).test(""));
    }

    #[test]
    fn test_float_spec() {
        assert!(Cond::new_float_spec(0.0, false).test("0"));
        assert!(!Cond::new_float_spec(0.0, false).test("1"));
        assert!(!Cond::new_float_spec(3.0, false).test("1"));
        assert!(Cond::new_float_spec(3.0, false).test("3"));
        assert!(!Cond::new_float_spec(3.0, false).test("abc"));
        assert!(!Cond::new_float_spec(3.0, false).test("NaN"));
        assert!(!Cond::new_float_spec(3.0, false).test("nan"));
        assert!(!Cond::new_float_spec(3.0, false).test("inf"));
        assert!(!Cond::new_float_spec(3.0, false).test("Inf"));
        assert!(!Cond::new_float_spec(3.0, false).test("-inf"));
        assert!(!Cond::new_float_spec(3.0, false).test("-Inf"));
        assert!(!Cond::new_float_spec(3.0, false).test(""));
        // not
        assert!(!Cond::new_float_spec(0.0, true).test("0"));
        assert!(Cond::new_float_spec(0.0, true).test("1"));
        assert!(Cond::new_float_spec(3.0, true).test("1"));
        assert!(!Cond::new_float_spec(3.0, true).test("3"));
        assert!(!Cond::new_float_spec(3.0, true).test("abc"));
        assert!(!Cond::new_float_spec(3.0, true).test("NaN"));
        assert!(!Cond::new_float_spec(3.0, true).test("nan"));
        assert!(!Cond::new_float_spec(3.0, true).test("inf"));
        assert!(!Cond::new_float_spec(3.0, true).test("Inf"));
        assert!(!Cond::new_float_spec(3.0, true).test("-inf"));
        assert!(!Cond::new_float_spec(3.0, true).test("-Inf"));
        assert!(!Cond::new_float_spec(3.0, true).test(""));
    }

    #[test]
    fn test_number_not() {
        // integer
        assert!(Cond::new_number_not(Some(true)).test("abc"));
        assert!(!Cond::new_number_not(Some(true)).test("123"));
        assert!(Cond::new_number_not(Some(true)).test("123.1"));
        assert!(Cond::new_number_not(Some(true)).test("123.0"));
        assert!(Cond::new_number_not(Some(true)).test("NaN"));
        assert!(Cond::new_number_not(Some(true)).test("nan"));
        assert!(Cond::new_number_not(Some(true)).test("inf"));
        assert!(Cond::new_number_not(Some(true)).test("Inf"));
        assert!(Cond::new_number_not(Some(true)).test("-inf"));
        assert!(Cond::new_number_not(Some(true)).test("-Inf"));
        assert!(Cond::new_number_not(Some(true)).test(""));
        // float
        assert!(Cond::new_number_not(Some(false)).test("abc"));
        assert!(Cond::new_number_not(Some(false)).test("123"));
        assert!(!Cond::new_number_not(Some(false)).test("123.1"));
        assert!(!Cond::new_number_not(Some(false)).test("123.0"));
        assert!(Cond::new_number_not(Some(false)).test("NaN"));
        assert!(Cond::new_number_not(Some(false)).test("nan"));
        assert!(Cond::new_number_not(Some(false)).test("inf"));
        assert!(Cond::new_number_not(Some(false)).test("Inf"));
        assert!(Cond::new_number_not(Some(false)).test("-inf"));
        assert!(Cond::new_number_not(Some(false)).test("-Inf"));
        assert!(Cond::new_number_not(Some(false)).test(""));
        // number
        assert!(Cond::new_number_not(None).test("abc"));
        assert!(!Cond::new_number_not(None).test("123"));
        assert!(!Cond::new_number_not(None).test("123.1"));
        assert!(!Cond::new_number_not(None).test("123.0"));
        assert!(Cond::new_number_not(None).test("NaN"));
        assert!(Cond::new_number_not(None).test("nan"));
        assert!(Cond::new_number_not(None).test("inf"));
        assert!(Cond::new_number_not(None).test("Inf"));
        assert!(Cond::new_number_not(None).test("-inf"));
        assert!(Cond::new_number_not(None).test("-Inf"));
        assert!(Cond::new_number_not(None).test(""));
    }

    #[test]
    fn test_text_all_case() {
        // upper
        assert!(!Cond::new_text_all_case(true).test("abc"));
        assert!(Cond::new_text_all_case(true).test("ABC"));
        assert!(!Cond::new_text_all_case(true).test("abcABC"));
        assert!(Cond::new_text_all_case(true).test("你好123.#!@"));
        // lower
        assert!(Cond::new_text_all_case(false).test("abc"));
        assert!(!Cond::new_text_all_case(false).test("ABC"));
        assert!(!Cond::new_text_all_case(false).test("abcABC"));
        assert!(Cond::new_text_all_case(false).test("你好123.#!@"));
    }

    #[test]
    fn test_text_empty_or_blank() {
        // empty
        assert!(Cond::new_text_empty_or_blank(true).test(""));
        assert!(!Cond::new_text_empty_or_blank(true).test("abc"));
        assert!(!Cond::new_text_empty_or_blank(true).test(" "));
        assert!(!Cond::new_text_empty_or_blank(true).test(" \n\t\r "));
        // blank
        assert!(Cond::new_text_empty_or_blank(false).test(""));
        assert!(!Cond::new_text_empty_or_blank(false).test("abc"));
        assert!(Cond::new_text_empty_or_blank(false).test(" "));
        assert!(Cond::new_text_empty_or_blank(false).test(" \n\t\r "));
    }

    #[test]
    fn test_reg_match() {
        assert!(Cond::new_reg_match(r"[").is_err());
        assert!(Cond::new_reg_match(r"\d+").unwrap().test("123"));
        assert!(!Cond::new_reg_match(r"\d+").unwrap().test("123abc"));
        assert!(!Cond::new_reg_match(r"\d+").unwrap().test("123\n123"));
        assert!(!Cond::new_reg_match(r"(?m)\d+").unwrap().test("123\n123"));
        assert!(Cond::new_reg_match(r"(?m)[\d\n]+").unwrap().test("123\n123"));
    }
}
