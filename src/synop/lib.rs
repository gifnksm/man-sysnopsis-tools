#![crate_id = "synop"]
#![crate_type = "lib"]

pub use token::{Token, Tokenizer};
pub use ast::{Expr, Tok, Seq, Opt, Repeat, Select};
pub use parser::{ParseResult, parse};
pub use util::{ReadResult, read_ast};

mod token;
mod ast;
mod parser;
mod util;
