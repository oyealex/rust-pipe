use crate::input::Item;

pub(crate) struct Pipe {
    pub(crate) iter: Box<dyn Iterator<Item = Item>>,
    // TODO 2026-01-10 01:27 增加特征描述
}

impl Iterator for Pipe {
    type Item = Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl Pipe {
    pub(crate) fn op_map(self, f: impl FnMut(Item) -> Item + 'static) -> Pipe {
        Pipe { iter: Box::new(self.map(f)) }
    }

    pub(crate) fn op_filter(self, f: impl FnMut(&Item) -> bool + 'static) -> Pipe {
        Pipe { iter: Box::new(self.filter(f)) }
    }

    pub(crate) fn op_inspect(self, f: impl FnMut(&Item) + 'static) -> Pipe {
        Pipe { iter: Box::new(self.inspect(f)) }
    }
}
