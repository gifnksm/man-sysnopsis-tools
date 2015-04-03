#![crate_name = "synop"]
#![crate_type = "lib"]
#![warn(unused, bad_style)]

#![feature(io)]

pub use token::{Token, Tokenizer};
pub use ast::Expr;
pub use parser::{ParseResult, parse};
pub use util::{ReadResult, read_ast};

pub mod token;
pub mod ast;
mod parser;
mod util;
