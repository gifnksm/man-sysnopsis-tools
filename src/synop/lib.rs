#![crate_id = "synop"]
#![crate_type = "lib"]

pub use token::{Token, Tokenizer};
pub use ast::Expr;
pub use parser::{ParseResult, parse};

mod token;
mod ast;
mod parser;
