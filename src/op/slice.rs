use std::fmt::Debug;
use std::iter::{Enumerate, Peekable};

#[derive(Debug)]
pub(crate) struct SliceIter<I: Iterator<Item: Debug>, R: Iterator<Item = (Option<usize>, Option<usize>)>> {
    source: Enumerate<I>,
    ranges: Peekable<R>,
}

impl<I: Iterator<Item: Debug>, R: Iterator<Item = (Option<usize>, Option<usize>)>> Iterator for SliceIter<I, R> {
    type Item = <I as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ranges.peek().is_none() {
            // 如果没有任何范围，就不再继续迭代原始数据了 OPT 增加UT？
            return None;
        }
        while let Some((idx, item)) = self.source.next() {
            match self.ranges.peek() {
                Some((Some(start), Some(end))) if *start <= idx && idx <= *end => {
                    if idx == *end {
                        self.ranges.next();
                    }
                    return Some(item);
                }
                Some((Some(start), None)) if *start <= idx => return Some(item),
                Some((None, Some(end))) if idx <= *end => {
                    if idx == *end {
                        self.ranges.next();
                    }
                    return Some(item);
                }
                None => return None, // 如果没有任何范围了，立即终止迭代，否则会无限循环 OPT 增加UT？
                _ => continue,
            }
        }
        None
    }
}

impl<I: Iterator<Item: Debug>, R: Iterator<Item = (Option<usize>, Option<usize>)>> SliceIter<I, R> {
    pub(crate) fn new(
        source: impl IntoIterator<IntoIter = I>, ranges: impl IntoIterator<IntoIter = R>,
    ) -> SliceIter<I, R> {
        SliceIter { source: source.into_iter().enumerate(), ranges: ranges.into_iter().peekable() }
    }
}

#[cfg(test)]
mod tests {
    use crate::op::slice::SliceIter;

    #[test]
    fn test_slice() {
        assert_eq!(Vec::<i32>::new(), SliceIter::new(0..=10, vec![]).collect::<Vec<_>>());
        assert_eq!(Vec::<i32>::new(), SliceIter::new(0..=10, vec![(Some(5), Some(2))]).collect::<Vec<_>>());
        assert_eq!((2..=5).collect::<Vec<_>>(), SliceIter::new(0..=10, vec![(Some(2), Some(5))]).collect::<Vec<_>>());
        assert_eq!((2..=10).collect::<Vec<_>>(), SliceIter::new(0..=10, vec![(Some(2), None)]).collect::<Vec<_>>());
        assert_eq!(
            (2..=10).collect::<Vec<_>>(),
            SliceIter::new(0..=10, vec![(Some(2), None), (None, Some(7))]).collect::<Vec<_>>()
        );
        assert_eq!(
            (2..=10).collect::<Vec<_>>(),
            SliceIter::new(0..=10, vec![(Some(2), None), (Some(5), Some(7))]).collect::<Vec<_>>()
        );
        assert_eq!((0..=2).collect::<Vec<_>>(), SliceIter::new(0..=10, vec![(None, Some(2))]).collect::<Vec<_>>());
        assert_eq!(
            (0..=7).collect::<Vec<_>>(),
            SliceIter::new(0..=10, vec![(None, Some(2)), (None, Some(7))]).collect::<Vec<_>>()
        );
        assert_eq!(
            (0..=2).chain(5..=7).collect::<Vec<_>>(),
            SliceIter::new(0..=10, vec![(None, Some(2)), (Some(5), Some(7))]).collect::<Vec<_>>()
        );
        assert_eq!(
            (0..=2).chain(5..=10).collect::<Vec<_>>(),
            SliceIter::new(0..=10, vec![(None, Some(2)), (Some(5), None)]).collect::<Vec<_>>()
        );
        assert_eq!(
            (2..=5).chain(7..=9).collect::<Vec<_>>(),
            SliceIter::new(0..=10, vec![(Some(2), Some(5)), (Some(7), Some(9))]).collect::<Vec<_>>()
        );
    }
}
