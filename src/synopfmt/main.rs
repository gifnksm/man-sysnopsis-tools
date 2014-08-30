#![crate_name = "synopfmt"]
#![crate_type = "bin"]
#![deny(warnings, unused, bad_style, unnecessary_qualification, unnecessary_typecast)]

extern crate cmdutil;
extern crate synop;

#[cfg(not(test))]
use std::io;

#[cfg(not(test))]
fn main() {
    cmdutil::main(proc() {
        let ast = try!(synop::read_ast(&mut io::stdin()));
        match ast.normalize() {
            Some(x) => println!("{}", x.pretty()),
            None    => println!("")
        }
        Ok(())
    });
}
