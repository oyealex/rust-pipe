pub(crate) enum Output {
    Out,
    File { file: &'static str },
    Clip,
}
