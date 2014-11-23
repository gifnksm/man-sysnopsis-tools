#![crate_name = "synop"]
#![crate_type = "lib"]
#![deny(warnings, unused, bad_style, unnecessary_typecast)]

#![feature(globs)]

pub use token::{Token, Tokenizer};
pub use ast::Expr;
pub use parser::{ParseResult, parse};
pub use util::{ReadResult, read_ast};

pub mod token;
pub mod ast;
mod parser;
mod util;
