#![crate_id = "cmdutil"]
#![crate_type = "lib"]

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
