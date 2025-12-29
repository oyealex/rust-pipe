use crate::input::{Item, Pipe};

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Output {
    Out,
    File { file: &'static str },
    Clip,
}

impl Output {
    pub(crate) fn handle(self, pipe: Pipe) {
        match self {
            Output::Out => {
                for item in pipe {
                    match item {
                        Item::String(string) => println!("{}", string),
                        Item::Str(string) => println!("{}", string),
                        Item::Integer(integer) => println!("{}", integer),
                    }
                }
            }
            Output::File { .. } => {}
            Output::Clip => {}
        }
    }
}
