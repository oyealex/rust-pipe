use rpipe::run;

fn main() {
    if let Err(e) = run(std::env::args().skip(1).peekable()) {
        e.termination();
    }
}
