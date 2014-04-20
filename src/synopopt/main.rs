#![crate_id = "synopopt"]

extern crate synop;

#[cfg(not(test))]
use synop::Tokenizer;

#[cfg(not(test))]
fn main() {
    for line in std::io::stdin().lines() {
        let p = synop::parse(Tokenizer::new(line.unwrap()));
        println!("{}", p.pretty());
    }
}
