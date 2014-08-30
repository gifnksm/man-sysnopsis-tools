#![crate_name = "cmdutil"]
#![crate_type = "lib"]
#![deny(warnings, unused, bad_style, unnecessary_qualification, unnecessary_typecast)]

use std::io;
use std::fmt::Show;

pub fn main<E: Show>(f: proc() -> Result<(), E>) {
    match f() {
        Ok(()) => {}
        Err(msg) => {
            let _ = writeln!(&mut io::stderr(), "{}", msg);
        }
    }
}
