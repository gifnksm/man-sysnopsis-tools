#![crate_id = "synop"]
#![crate_type = "lib"]

pub use token::Tokenizer;
pub use parser::parse;

pub mod token;
pub mod parser;
