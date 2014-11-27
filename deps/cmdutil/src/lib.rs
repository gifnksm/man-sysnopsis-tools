#![crate_name = "cmdutil"]
#![crate_type = "lib"]
#![deny(warnings, unused, bad_style, unused_qualifications, unused_typecasts)]

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
