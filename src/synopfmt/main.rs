#![crate_id = "synopfmt"]

extern crate synop;

#[cfg(not(test))]
use synop::Tokenizer;

#[cfg(not(test))]
fn main() {
    let mut stdin = std::io::stdin();
    let cs = stdin.chars().map(|c| c.unwrap());
    match synop::parse(Tokenizer::new(cs)).normalize() {
        Some(x) => println!("{}", x.pretty()),
        None    => println!("")
    }
}
