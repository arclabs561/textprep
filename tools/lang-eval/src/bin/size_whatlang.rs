//! Binary size with whatlang: textprep's `detect` (whatlang).
fn main() {
    let text = std::env::args().nth(1).unwrap_or_default();
    println!("{:?}", textprep::language::detect(&text).map(|l| l.code()));
}
