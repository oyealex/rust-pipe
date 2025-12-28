use crate::input::{Item, Pipe};

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Output {
    Out,
    File { file: String },
    Clip,
}

impl Output {
    pub(crate) fn handle(self, pipe: Pipe) {
        match self {
            Output::Out => {
                for item in pipe {
                    match item {
                        Item::Integer(integer) => println!("{}", integer),
                        Item::String(string) => println!("{}", string),
                    }
                }
            }
            Output::File { .. } => {}
            Output::Clip => {}
        }
    }
}