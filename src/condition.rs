use crate::err::RpErr;
use crate::{Float, Integer, Num};
use cmd_help::CmdHelp;
use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CondRangeArg<T> {
    min: Option<T>,
    max: Option<T>,
    not: bool,
}

impl<T> CondRangeArg<T> {
    pub(crate) fn new(min: Option<T>, max: Option<T>, not: bool) -> CondRangeArg<T> {
        CondRangeArg { min, max, not }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CondSpecArg<T> {
    spec: T,
    not: bool,
}

impl<T> CondSpecArg<T> {
    pub(crate) fn new(spec: T, not: bool) -> CondSpecArg<T> {
        CondSpecArg { spec, not }
    }
}

/// 条件
#[derive(Debug, Clone, CmdHelp)]
pub(crate) enum Cond {
    /// len [!][<min_len>],[<max_len>]
    ///     按照字符串长度范围选择，范围表达式最小值和最大值至少指定其一，支持可选的否定。
    ///     例如：
    ///         len 2,
    ///         len 2,5
    ///         len ,5
    ///         len !,5
    ///         len !2,5
    TextLenRange(CondRangeArg<usize>),
    /// len [!]=<len>
    ///     按照字符串特定长度选择，支持可选的否定。
    ///     例如：
    ///         len =3
    ///         len !=3
    TextLenSpec(CondSpecArg<usize>),
    /// num [!][<min>],[<max>]
    ///     按照数值范围选择，范围表达式最小值和最大值至少指定其一，支持可选的否定。
    ///     如果无法解析为数则不选择。
    ///     例如：
    ///         num 2,5
    ///         num -2.1,5
    ///         num 2,5.3
    ///         num ,5.3
    ///         num !1,5.3
    NumRange(CondRangeArg<Num>),
    /// num [!]=<spec>
    ///     按照数值特定值选择，支持可选的否定。
    ///     如果无法解析为数则不选择。
    ///     例如：
    ///         num =3
    ///         num =3.3
    ///         num !=3.3
    NumSpec(CondSpecArg<Num>),
    /// num[ [!][integer|float]]
    ///     按照整数或浮点数选择，如果不指定则选择数值数据，支持可选的否定。
    ///     例如：
    ///         num
    ///         num integer
    ///         num float
    ///         num !
    ///         num !integer
    ///         num !float
    Number { is_integer: Option<bool>, not: bool },
    /// upper|lower
    ///     选择全部为大写或小写字符的数据，不支持大小写的字符总是满足。
    TextAllCase(bool /*is_upper*/),
    /// empty|blank
    ///     选择没有任何字符或全部为空白字符的数据。
    TextEmptyOrBlank(bool /*is_empty*/),
    /// reg <exp>
    ///     选择匹配给定正则表达式的数据。
    ///     例如：
    ///         reg '\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}'
    RegMatch(Regex),
}

impl PartialEq for Cond {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Cond::TextLenRange(l), Cond::TextLenRange(r)) => l == r,
            (Cond::TextLenSpec(l), Cond::TextLenSpec(r)) => l == r,
            (Cond::NumRange(l), Cond::NumRange(r)) => l == r,
            (Cond::NumSpec(l), Cond::NumSpec(r)) => l == r,
            (
                Cond::Number { is_integer: l_is_integer, not: l_not },
                Cond::Number { is_integer: r_is_integer, not: r_not },
            ) => l_is_integer == r_is_integer && l_not == r_not,
            (Cond::TextAllCase(l), Cond::TextAllCase(r)) => l == r,
            (Cond::TextEmptyOrBlank(l), Cond::TextEmptyOrBlank(r)) => l == r,
            // Regex 比较模式字符串
            (Cond::RegMatch(re1), Cond::RegMatch(re2)) => re1.as_str() == re2.as_str(),
            // 其他情况都不相等
            _ => false,
        }
    }
}

#[inline]
fn with_not(res: bool, not: bool) -> bool {
    if not { !res } else { res }
}

impl Cond {
    pub(crate) fn new_number(is_integer: Option<bool>, not: bool) -> Cond {
        Cond::Number { is_integer, not }
    }
    pub(crate) fn new_reg_match(regex: &str) -> Result<Cond, RpErr> {
        let reg = format!(r"\A(?:{})\z", regex);
        Regex::new(&reg)
            .map(|reg| Cond::RegMatch(reg))
            .map_err(|err| RpErr::ParseRegexErr { reg, err: err.to_string() })
    }

    pub(crate) fn test(&self, input: &str) -> bool {
        match self {
            Cond::TextLenRange(CondRangeArg { min, max, not }) => {
                let len = *&input.chars().count();
                with_not(min.map_or(true, |min_len| len >= min_len) && max.map_or(true, |max_len| len <= max_len), *not)
            }
            Cond::TextLenSpec(CondSpecArg { spec: len, not }) => with_not(input.chars().count() == *len, *not),
            Cond::NumRange(CondRangeArg { min, max, not }) => input
                .parse::<Num>()
                .map(|i| {
                    with_not(min.map_or(true, |min_len| i >= min_len) && max.map_or(true, |max_len| i <= max_len), *not)
                })
                .unwrap_or(false),
            Cond::NumSpec(CondSpecArg { spec: val, not }) => {
                input.parse::<Num>().ok().map(|i| with_not(&i == val, *not)).unwrap_or(false)
            }
            Cond::Number { is_integer, not } => match is_integer {
                Some(integer) => {
                    if *integer {
                        with_not(input.parse::<Integer>().is_ok(), *not)
                    } else {
                        with_not(
                            input.parse::<Integer>().is_err()
                                && input.parse::<Float>().map_or(false, |v| v.is_finite()),
                            *not,
                        )
                    }
                }
                None => with_not(input.parse::<Float>().map_or(false, |v| v.is_finite()), *not),
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
        assert!(!Cond::TextLenRange(CondRangeArg::new(Some(3), Some(5), false)).test("12"));
        assert!(Cond::TextLenRange(CondRangeArg::new(Some(3), Some(5), false)).test("123"));
        assert!(Cond::TextLenRange(CondRangeArg::new(Some(3), Some(5), false)).test("1234"));
        assert!(Cond::TextLenRange(CondRangeArg::new(Some(3), Some(5), false)).test("12345"));
        assert!(!Cond::TextLenRange(CondRangeArg::new(Some(3), Some(5), false)).test("123456"));
        assert!(!Cond::TextLenRange(CondRangeArg::new(Some(3), None, false)).test("12"));
        assert!(Cond::TextLenRange(CondRangeArg::new(Some(3), None, false)).test("123"));
        assert!(Cond::TextLenRange(CondRangeArg::new(Some(3), None, false)).test("1234"));
        assert!(Cond::TextLenRange(CondRangeArg::new(None, Some(3), false)).test("12"));
        assert!(Cond::TextLenRange(CondRangeArg::new(None, Some(3), false)).test("123"));
        assert!(!Cond::TextLenRange(CondRangeArg::new(None, Some(3), false)).test("1234"));
        assert!(Cond::TextLenRange(CondRangeArg::new(None, None, false)).test("123"));
        // not
        assert!(Cond::TextLenRange(CondRangeArg::new(Some(3), Some(5), true)).test("12"));
        assert!(!Cond::TextLenRange(CondRangeArg::new(Some(3), Some(5), true)).test("123"));
        assert!(!Cond::TextLenRange(CondRangeArg::new(Some(3), Some(5), true)).test("1234"));
        assert!(!Cond::TextLenRange(CondRangeArg::new(Some(3), Some(5), true)).test("12345"));
        assert!(Cond::TextLenRange(CondRangeArg::new(Some(3), Some(5), true)).test("123456"));
        assert!(Cond::TextLenRange(CondRangeArg::new(Some(3), None, true)).test("12"));
        assert!(!Cond::TextLenRange(CondRangeArg::new(Some(3), None, true)).test("123"));
        assert!(!Cond::TextLenRange(CondRangeArg::new(Some(3), None, true)).test("1234"));
        assert!(!Cond::TextLenRange(CondRangeArg::new(None, Some(3), true)).test("12"));
        assert!(!Cond::TextLenRange(CondRangeArg::new(None, Some(3), true)).test("123"));
        assert!(Cond::TextLenRange(CondRangeArg::new(None, Some(3), true)).test("1234"));
        assert!(!Cond::TextLenRange(CondRangeArg::new(None, None, true)).test("123"));
    }

    #[test]
    fn test_text_len_spec() {
        assert!(Cond::TextLenSpec(CondSpecArg::new(0, false)).test(""));
        assert!(!Cond::TextLenSpec(CondSpecArg::new(0, false)).test("1"));
        assert!(!Cond::TextLenSpec(CondSpecArg::new(3, false)).test(""));
        assert!(!Cond::TextLenSpec(CondSpecArg::new(3, false)).test("12"));
        assert!(Cond::TextLenSpec(CondSpecArg::new(3, false)).test("123"));
        assert!(!Cond::TextLenSpec(CondSpecArg::new(3, false)).test("1234"));
        // not
        assert!(!Cond::TextLenSpec(CondSpecArg::new(0, true)).test(""));
        assert!(Cond::TextLenSpec(CondSpecArg::new(0, true)).test("1"));
        assert!(Cond::TextLenSpec(CondSpecArg::new(3, true)).test(""));
        assert!(Cond::TextLenSpec(CondSpecArg::new(3, true)).test("12"));
        assert!(!Cond::TextLenSpec(CondSpecArg::new(3, true)).test("123"));
        assert!(Cond::TextLenSpec(CondSpecArg::new(3, true)).test("1234"));
    }

    #[test]
    fn test_integer_range() {
        assert!(!Cond::NumRange(CondRangeArg::new(Some(Num::from(3)), Some(Num::from(5)), false)).test("2"));
        assert!(Cond::NumRange(CondRangeArg::new(Some(Num::from(3)), Some(Num::from(5)), false)).test("3"));
        assert!(Cond::NumRange(CondRangeArg::new(Some(Num::from(3)), Some(Num::from(5)), false)).test("4"));
        assert!(Cond::NumRange(CondRangeArg::new(Some(Num::from(3)), Some(Num::from(5)), false)).test("5"));
        assert!(!Cond::NumRange(CondRangeArg::new(Some(Num::from(3)), Some(Num::from(5)), false)).test("6"));
        assert!(!Cond::NumRange(CondRangeArg::new(Some(Num::from(3)), None, false)).test("2"));
        assert!(Cond::NumRange(CondRangeArg::new(Some(Num::from(3)), None, false)).test("3"));
        assert!(Cond::NumRange(CondRangeArg::new(Some(Num::from(3)), None, false)).test("4"));
        assert!(Cond::NumRange(CondRangeArg::new(None, Some(Num::from(3)), false)).test("2"));
        assert!(Cond::NumRange(CondRangeArg::new(None, Some(Num::from(3)), false)).test("3"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, Some(Num::from(3)), false)).test("4"));
        assert!(Cond::NumRange(CondRangeArg::new(None, None, false)).test("3"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, None, false)).test("abc"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, None, false)).test(""));
        // not
        assert!(Cond::NumRange(CondRangeArg::new(Some(Num::from(3)), Some(Num::from(5)), true)).test("2"));
        assert!(!Cond::NumRange(CondRangeArg::new(Some(Num::from(3)), Some(Num::from(5)), true)).test("3"));
        assert!(!Cond::NumRange(CondRangeArg::new(Some(Num::from(3)), Some(Num::from(5)), true)).test("4"));
        assert!(!Cond::NumRange(CondRangeArg::new(Some(Num::from(3)), Some(Num::from(5)), true)).test("5"));
        assert!(Cond::NumRange(CondRangeArg::new(Some(Num::from(3)), Some(Num::from(5)), true)).test("6"));
        assert!(Cond::NumRange(CondRangeArg::new(Some(Num::from(3)), None, true)).test("2"));
        assert!(!Cond::NumRange(CondRangeArg::new(Some(Num::from(3)), None, true)).test("3"));
        assert!(!Cond::NumRange(CondRangeArg::new(Some(Num::from(3)), None, true)).test("4"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, Some(Num::from(3)), true)).test("2"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, Some(Num::from(3)), true)).test("3"));
        assert!(Cond::NumRange(CondRangeArg::new(None, Some(Num::from(3)), true)).test("4"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, None, true)).test("3"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, None, true)).test("abc"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, None, true)).test(""));
    }

    #[test]
    fn test_integer_spec() {
        assert!(Cond::NumSpec(CondSpecArg::new(Num::from(0), false)).test("0"));
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(0), false)).test("1"));
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(3), false)).test("1"));
        assert!(Cond::NumSpec(CondSpecArg::new(Num::from(3), false)).test("3"));
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(3), false)).test("abc"));
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(3), false)).test(""));
        // not
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(0), true)).test("0"));
        assert!(Cond::NumSpec(CondSpecArg::new(Num::from(0), true)).test("1"));
        assert!(Cond::NumSpec(CondSpecArg::new(Num::from(3), true)).test("1"));
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(3), true)).test("3"));
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(3), true)).test("abc"));
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(3), true)).test(""));
    }

    #[test]
    fn test_float_range() {
        assert!(!Cond::NumRange(CondRangeArg::new(Some(Num::from(3.0)), Some(Num::from(5.0)), false)).test("2"));
        assert!(Cond::NumRange(CondRangeArg::new(Some(Num::from(3.0)), Some(Num::from(5.0)), false)).test("3"));
        assert!(Cond::NumRange(CondRangeArg::new(Some(Num::from(3.0)), Some(Num::from(5.0)), false)).test("4"));
        assert!(Cond::NumRange(CondRangeArg::new(Some(Num::from(3.0)), Some(Num::from(5.0)), false)).test("5"));
        assert!(!Cond::NumRange(CondRangeArg::new(Some(Num::from(3.0)), Some(Num::from(5.0)), false)).test("6"));
        assert!(!Cond::NumRange(CondRangeArg::new(Some(Num::from(3.0)), None, false)).test("2"));
        assert!(Cond::NumRange(CondRangeArg::new(Some(Num::from(3.0)), None, false)).test("3"));
        assert!(Cond::NumRange(CondRangeArg::new(Some(Num::from(3.0)), None, false)).test("4"));
        assert!(Cond::NumRange(CondRangeArg::new(None, Some(Num::from(3.0)), false)).test("2"));
        assert!(Cond::NumRange(CondRangeArg::new(None, Some(Num::from(3.0)), false)).test("3"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, Some(Num::from(3.0)), false)).test("4"));
        assert!(Cond::NumRange(CondRangeArg::new(None, None, false)).test("3"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, None, false)).test("abc"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, None, false)).test("NaN"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, None, false)).test("nan"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, None, false)).test("inf"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, None, false)).test("Inf"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, None, false)).test("-inf"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, None, false)).test("-Inf"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, None, false)).test(""));
        // not
        assert!(Cond::NumRange(CondRangeArg::new(Some(Num::from(3.0)), Some(Num::from(5.0)), true)).test("2"));
        assert!(!Cond::NumRange(CondRangeArg::new(Some(Num::from(3.0)), Some(Num::from(5.0)), true)).test("3"));
        assert!(!Cond::NumRange(CondRangeArg::new(Some(Num::from(3.0)), Some(Num::from(5.0)), true)).test("4"));
        assert!(!Cond::NumRange(CondRangeArg::new(Some(Num::from(3.0)), Some(Num::from(5.0)), true)).test("5"));
        assert!(Cond::NumRange(CondRangeArg::new(Some(Num::from(3.0)), Some(Num::from(5.0)), true)).test("6"));
        assert!(Cond::NumRange(CondRangeArg::new(Some(Num::from(3.0)), None, true)).test("2"));
        assert!(!Cond::NumRange(CondRangeArg::new(Some(Num::from(3.0)), None, true)).test("3"));
        assert!(!Cond::NumRange(CondRangeArg::new(Some(Num::from(3.0)), None, true)).test("4"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, Some(Num::from(3.0)), true)).test("2"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, Some(Num::from(3.0)), true)).test("3"));
        assert!(Cond::NumRange(CondRangeArg::new(None, Some(Num::from(3.0)), true)).test("4"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, None, true)).test("3"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, None, true)).test("abc"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, None, true)).test("NaN"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, None, true)).test("nan"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, None, true)).test("inf"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, None, true)).test("Inf"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, None, true)).test("-inf"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, None, true)).test("-Inf"));
        assert!(!Cond::NumRange(CondRangeArg::new(None, None, true)).test(""));
    }

    #[test]
    fn test_float_spec() {
        assert!(Cond::NumSpec(CondSpecArg::new(Num::from(0.0), false)).test("0"));
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(0.0), false)).test("1"));
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(3.0), false)).test("1"));
        assert!(Cond::NumSpec(CondSpecArg::new(Num::from(3.0), false)).test("3"));
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(3.0), false)).test("abc"));
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(3.0), false)).test("NaN"));
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(3.0), false)).test("nan"));
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(3.0), false)).test("inf"));
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(3.0), false)).test("Inf"));
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(3.0), false)).test("-inf"));
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(3.0), false)).test("-Inf"));
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(3.0), false)).test(""));
        // not
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(0.0), true)).test("0"));
        assert!(Cond::NumSpec(CondSpecArg::new(Num::from(0.0), true)).test("1"));
        assert!(Cond::NumSpec(CondSpecArg::new(Num::from(3.0), true)).test("1"));
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(3.0), true)).test("3"));
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(3.0), true)).test("abc"));
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(3.0), true)).test("NaN"));
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(3.0), true)).test("nan"));
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(3.0), true)).test("inf"));
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(3.0), true)).test("Inf"));
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(3.0), true)).test("-inf"));
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(3.0), true)).test("-Inf"));
        assert!(!Cond::NumSpec(CondSpecArg::new(Num::from(3.0), true)).test(""));
    }

    #[test]
    fn test_number_not() {
        // integer
        assert!(Cond::new_number(Some(true), true).test("abc"));
        assert!(!Cond::new_number(Some(true), true).test("123"));
        assert!(Cond::new_number(Some(true), true).test("123.1"));
        assert!(Cond::new_number(Some(true), true).test("123.0"));
        assert!(Cond::new_number(Some(true), true).test("NaN"));
        assert!(Cond::new_number(Some(true), true).test("nan"));
        assert!(Cond::new_number(Some(true), true).test("inf"));
        assert!(Cond::new_number(Some(true), true).test("Inf"));
        assert!(Cond::new_number(Some(true), true).test("-inf"));
        assert!(Cond::new_number(Some(true), true).test("-Inf"));
        assert!(Cond::new_number(Some(true), true).test(""));
        assert!(!Cond::new_number(Some(true), false).test("abc"));
        assert!(Cond::new_number(Some(true), false).test("123"));
        assert!(!Cond::new_number(Some(true), false).test("123.1"));
        assert!(!Cond::new_number(Some(true), false).test("123.0"));
        assert!(!Cond::new_number(Some(true), false).test("NaN"));
        assert!(!Cond::new_number(Some(true), false).test("nan"));
        assert!(!Cond::new_number(Some(true), false).test("inf"));
        assert!(!Cond::new_number(Some(true), false).test("Inf"));
        assert!(!Cond::new_number(Some(true), false).test("-inf"));
        assert!(!Cond::new_number(Some(true), false).test("-Inf"));
        assert!(!Cond::new_number(Some(true), false).test(""));
        // float
        assert!(Cond::new_number(Some(false), true).test("abc"));
        assert!(Cond::new_number(Some(false), true).test("123"));
        assert!(!Cond::new_number(Some(false), true).test("123.1"));
        assert!(!Cond::new_number(Some(false), true).test("123.0"));
        assert!(Cond::new_number(Some(false), true).test("NaN"));
        assert!(Cond::new_number(Some(false), true).test("nan"));
        assert!(Cond::new_number(Some(false), true).test("inf"));
        assert!(Cond::new_number(Some(false), true).test("Inf"));
        assert!(Cond::new_number(Some(false), true).test("-inf"));
        assert!(Cond::new_number(Some(false), true).test("-Inf"));
        assert!(Cond::new_number(Some(false), true).test(""));
        assert!(!Cond::new_number(Some(false), false).test("abc"));
        assert!(!Cond::new_number(Some(false), false).test("123"));
        assert!(Cond::new_number(Some(false), false).test("123.1"));
        assert!(Cond::new_number(Some(false), false).test("123.0"));
        assert!(!Cond::new_number(Some(false), false).test("NaN"));
        assert!(!Cond::new_number(Some(false), false).test("nan"));
        assert!(!Cond::new_number(Some(false), false).test("inf"));
        assert!(!Cond::new_number(Some(false), false).test("Inf"));
        assert!(!Cond::new_number(Some(false), false).test("-inf"));
        assert!(!Cond::new_number(Some(false), false).test("-Inf"));
        assert!(!Cond::new_number(Some(false), false).test(""));
        // number
        assert!(Cond::new_number(None, true).test("abc"));
        assert!(!Cond::new_number(None, true).test("123"));
        assert!(!Cond::new_number(None, true).test("123.1"));
        assert!(!Cond::new_number(None, true).test("123.0"));
        assert!(Cond::new_number(None, true).test("NaN"));
        assert!(Cond::new_number(None, true).test("nan"));
        assert!(Cond::new_number(None, true).test("inf"));
        assert!(Cond::new_number(None, true).test("Inf"));
        assert!(Cond::new_number(None, true).test("-inf"));
        assert!(Cond::new_number(None, true).test("-Inf"));
        assert!(Cond::new_number(None, true).test(""));
        assert!(!Cond::new_number(None, false).test("abc"));
        assert!(Cond::new_number(None, false).test("123"));
        assert!(Cond::new_number(None, false).test("123.1"));
        assert!(Cond::new_number(None, false).test("123.0"));
        assert!(!Cond::new_number(None, false).test("NaN"));
        assert!(!Cond::new_number(None, false).test("nan"));
        assert!(!Cond::new_number(None, false).test("inf"));
        assert!(!Cond::new_number(None, false).test("Inf"));
        assert!(!Cond::new_number(None, false).test("-inf"));
        assert!(!Cond::new_number(None, false).test("-Inf"));
        assert!(!Cond::new_number(None, false).test(""));
    }

    #[test]
    fn test_text_all_case() {
        // upper
        assert!(!Cond::TextAllCase(true).test("abc"));
        assert!(Cond::TextAllCase(true).test("ABC"));
        assert!(!Cond::TextAllCase(true).test("abcABC"));
        assert!(Cond::TextAllCase(true).test("你好123.#!@"));
        // lower
        assert!(Cond::TextAllCase(false).test("abc"));
        assert!(!Cond::TextAllCase(false).test("ABC"));
        assert!(!Cond::TextAllCase(false).test("abcABC"));
        assert!(Cond::TextAllCase(false).test("你好123.#!@"));
    }

    #[test]
    fn test_text_empty_or_blank() {
        // empty
        assert!(Cond::TextEmptyOrBlank(true).test(""));
        assert!(!Cond::TextEmptyOrBlank(true).test("abc"));
        assert!(!Cond::TextEmptyOrBlank(true).test(" "));
        assert!(!Cond::TextEmptyOrBlank(true).test(" \n\t\r "));
        // blank
        assert!(Cond::TextEmptyOrBlank(false).test(""));
        assert!(!Cond::TextEmptyOrBlank(false).test("abc"));
        assert!(Cond::TextEmptyOrBlank(false).test(" "));
        assert!(Cond::TextEmptyOrBlank(false).test(" \n\t\r "));
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
