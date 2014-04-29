#![crate_id = "synopfmt"]

extern crate synop;

#[cfg(not(test))]
use synop::Tokenizer;

#[cfg(not(test))]
fn main() {
    for line in std::io::stdin().lines() {
        let p = synop::parse(Tokenizer::new(line.unwrap()));
        match p.normalize() {
            Some(x) => println!("{}", x.pretty()),
            None    => println!("")
        }
    }
}
