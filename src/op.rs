use crate::input::{Item, Pipe};

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Op {
    Upper,
    Lower,
    Replace { from: &'static str, to: &'static str, count: u32 },
}

impl Op {
    pub(crate) fn wrap(self, pipe: Pipe) -> Pipe {
        match self {
            Op::Upper => pipe.op_map(|mut item| {
                if let Item::String(ref mut string) = item {
                    string.make_ascii_uppercase();
                };
                item
            }),
            Op::Lower => pipe.op_map(|mut item| {
                if let Item::String(ref mut string) = item {
                    string.make_ascii_lowercase();
                };
                item
            }),
            Op::Replace { .. } => {todo!()}
        }
    }
}
