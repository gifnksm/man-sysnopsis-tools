#![crate_name = "cmdutil"]
#![crate_type = "lib"]
#![warn(unused, bad_style, unused_qualifications, unused_typecasts)]

#![feature(io)]

use std::old_io as io;
use std::fmt;

pub fn main<E: fmt::Display, F: FnOnce() -> Result<(), E>>(f: F) {
    match f() {
        Ok(()) => {}
        Err(msg) => {
            let _ = writeln!(&mut io::stderr(), "{}", msg);
        }
    }
}
