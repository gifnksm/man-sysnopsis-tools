use std::io::BufReader;
use std::io::prelude::*;
use super::Tokenizer;
use ast::Expr;

pub type ReadResult = Result<Expr, String>;

pub fn read_ast<R: Read>(reader: R) -> ReadResult {
    let cs = BufReader::new(reader).chars().map(|c| c.unwrap());
    match super::parse(Tokenizer::new(cs)) {
        Ok(ast)  => Ok(ast),
        Err(msg) => Err(format!("Parse error: {}",  msg))
    }
}
