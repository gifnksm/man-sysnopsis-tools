#![crate_id = "synopfmt"]

extern crate synop;

#[cfg(not(test))]
use std::io;

#[cfg(not(test))]
fn main() {
    let ast = match synop::read_ast(&mut io::stdin()) {
        Ok(ast) => ast,
        Err(msg) => {
            let _ = writeln!(&mut io::stderr(), "{}", msg);
            return
        }
    };
    match ast.normalize() {
        Some(x) => println!("{}", x.pretty()),
        None    => println!("")
    }
}
