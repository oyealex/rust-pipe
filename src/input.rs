use crate::Integer;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::iter::repeat;

#[derive(Debug, Eq, PartialEq, Clone)]
pub(crate) enum Item {
    String(String),
    Integer(Integer),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Input {
    /// 标准输入：`rp in`
    StdIn,
    /// 外部文件
    File { files: Vec<String> },
    /// 剪切板
    Clip,
    /// 直接字面值
    Of { values: Vec<String> },
    /// 整数生成器
    Gen { start: Integer, end: Integer, included: bool, step: Integer },
    /// 重复
    Repeat { value: String, count: Option<usize> },
}

pub(crate) enum Pipe {
    Unbounded(Box<dyn Iterator<Item = Item>>),
    Bounded(Box<dyn DoubleEndedIterator<Item = Item>>),
}

impl Pipe {
    pub(crate) fn op_map(self, f: impl FnMut(Item) -> Item + 'static) -> Pipe {
        match self {
            Pipe::Unbounded(iter) => Pipe::Unbounded(Box::new(iter.map(f))),
            Pipe::Bounded(iter) => Pipe::Unbounded(Box::new(iter.map(f))),
        }
    }

    pub(crate) fn op_filter(self, f: impl FnMut(&Item) -> bool + 'static) -> Pipe {
        match self {
            Pipe::Unbounded(iter) => Pipe::Unbounded(Box::new(iter.filter(f))),
            Pipe::Bounded(iter) => Pipe::Bounded(Box::new(iter.filter(f))),
        }
    }
}

impl Iterator for Pipe {
    type Item = Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Pipe::Unbounded(iter) => iter.next(),
            Pipe::Bounded(iter) => iter.next(),
        }
    }
}

impl Input {
    pub(crate) fn pipe(self) -> Pipe {
        match self {
            Input::StdIn => Pipe::Unbounded(Box::new(
                io::stdin()
                    .lock()
                    .lines()
                    .into_iter()
                    .take_while(Result::is_ok)
                    .map(|line| Item::String(line.unwrap())),
            )),
            Input::File { files } => Pipe::Unbounded(Box::new(
                files
                    .into_iter()
                    .map(File::open)
                    .take_while(Result::is_ok)
                    .map(Result::unwrap)
                    .map(BufReader::new)
                    .flat_map(|reader| BufRead::lines(reader).into_iter())
                    .take_while(Result::is_ok)
                    .map(|line| Item::String(line.unwrap())),
            )),
            Input::Clip => {
                todo!("Clip not implemented yet")
            }
            Input::Of { values } => Pipe::Bounded(Box::new(values.into_iter().map(Item::String))),
            Input::Gen { start, end, included, step } => {
                // TODO 2025-12-28 21:59 如果没有指定end，设定为Unbounded。
                Pipe::Bounded(Box::new(range_to_iter(start, end, included, step).map(|x| Item::Integer(x))))
            }
            Input::Repeat { value, count } => {
                if count.is_none() {
                    Pipe::Unbounded(Box::new(repeat(Item::String(value))))
                } else {
                    Pipe::Bounded(Box::new(repeat(Item::String(value)).take(count.unwrap())))
                }
            }
        }
    }
}

fn range_to_iter(
    start: Integer, end: Integer, included: bool, step: Integer,
) -> Box<dyn DoubleEndedIterator<Item = Integer>> {
    let iter = RangeIter {
        start,
        end,
        included,
        step: Integer::abs(step),
        next: start,
        next_back: if included { end } else { end - 1 },
    };
    if step < 0 { Box::new(iter.rev()) } else { Box::new(iter) }
}

#[derive(Debug, Eq, PartialEq)]
struct RangeIter {
    start: Integer,
    end: Integer,
    included: bool,
    step: Integer,
    next: Integer,
    next_back: Integer,
}

impl Iterator for RangeIter {
    type Item = Integer;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next >= self.start
            && (self.included && self.next <= self.end || !self.included && self.next < self.end)
            && self.next <= self.next_back
        {
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
        if self.next_back >= self.start
            && (self.included && self.next_back <= self.end || !self.included && self.next_back < self.end)
            && self.next_back >= self.next
        {
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
    fn test_positive() {
        assert_eq!(range_to_iter(0, 10, false, 1).collect::<Vec<_>>(), (0..10).collect::<Vec<_>>());
        assert_eq!(range_to_iter(0, 10, true, 1).collect::<Vec<_>>(), (0..=10).collect::<Vec<_>>());
        assert_eq!(range_to_iter(0, 10, false, 2).collect::<Vec<_>>(), (0..10).step_by(2).collect::<Vec<_>>());
        assert_eq!(range_to_iter(0, 10, true, 2).collect::<Vec<_>>(), (0..=10).step_by(2).collect::<Vec<_>>());
    }

    #[test]
    fn test_negative() {
        assert_eq!(range_to_iter(0, 10, false, -1).collect::<Vec<_>>(), (0..10).rev().collect::<Vec<_>>());
        assert_eq!(range_to_iter(0, 10, true, -1).collect::<Vec<_>>(), (0..=10).rev().collect::<Vec<_>>());
        assert_eq!(range_to_iter(0, 10, false, -2).collect::<Vec<_>>(), (0..10).rev().step_by(2).collect::<Vec<_>>());
        assert_eq!(range_to_iter(0, 10, true, -2).collect::<Vec<_>>(), (0..=10).rev().step_by(2).collect::<Vec<_>>());
    }

    #[test]
    fn test_empty() {
        assert_eq!(range_to_iter(0, 0, false, 1).collect::<Vec<_>>(), (0..0).collect::<Vec<_>>());
        assert_eq!(range_to_iter(0, 0, true, 1).collect::<Vec<_>>(), (0..=0).collect::<Vec<_>>());
        assert_eq!(range_to_iter(0, 0, false, 2).collect::<Vec<_>>(), (0..0).step_by(2).collect::<Vec<_>>());
        assert_eq!(range_to_iter(0, 0, true, 2).collect::<Vec<_>>(), (0..=0).step_by(2).collect::<Vec<_>>());
    }

    #[test]
    fn test_reverted_range() {
        assert_eq!(range_to_iter(10, 0, false, 1).collect::<Vec<_>>(), (10..0).collect::<Vec<_>>());
        assert_eq!(range_to_iter(10, 0, true, 1).collect::<Vec<_>>(), (10..=0).collect::<Vec<_>>());
        assert_eq!(range_to_iter(10, 0, false, 2).collect::<Vec<_>>(), (10..0).step_by(2).collect::<Vec<_>>());
        assert_eq!(range_to_iter(10, 0, true, 2).collect::<Vec<_>>(), (10..=0).step_by(2).collect::<Vec<_>>());
    }

    #[test]
    fn test_zero_step() {
        assert_eq!(range_to_iter(0, 0, false, 0).next().is_none(), true);
        assert_eq!(range_to_iter(0, 1, false, 0).take(10).collect::<Vec<_>>(), vec![0; 10]);
        assert_eq!(range_to_iter(0, 1, false, 0).take(100).collect::<Vec<_>>(), vec![0; 100]);
    }
}
