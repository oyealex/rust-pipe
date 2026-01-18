use rpipe::run;

fn main() {
    let args = vec![":of", "abc123ABC", ":trimc", "cab1", "nocase"].into_iter().map(|s| s.to_owned()).peekable();
    if let Err(err) = run(args) {
        eprintln!("{err:?}");
    }
}
