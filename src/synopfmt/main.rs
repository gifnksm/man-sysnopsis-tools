#![crate_name = "synopfmt"]
#![crate_type = "bin"]
#![warn(unused, bad_style, unused_qualifications, unused_typecasts)]

#![cfg_attr(not(test), feature(io))]

#[cfg(not(test))]
extern crate cmdutil;
#[cfg(not(test))]
extern crate synop;

#[cfg(not(test))]
use std::old_io as io;

#[cfg(not(test))]
fn main() {
    cmdutil::main(|| {
        let ast = match synop::read_ast(io::stdin()) {
            Ok(ast) => ast,
            Err(s)  => return Err(s)
        };
        match ast.normalize() {
            Some(x) => println!("{}", x.pretty()),
            None    => println!("")
        }
        Ok(())
    });
}
