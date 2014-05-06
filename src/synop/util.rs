use std::io::Buffer;
use super::Tokenizer;
use ast::Expr;

pub type ReadResult = Result<Expr, ~str>;

pub fn read_ast<R: Buffer>(reader: &mut R) -> ReadResult {
    let cs = reader.chars().map(|c| c.unwrap());
    match super::parse(Tokenizer::new(cs)) {
        Ok(ast)  => Ok(ast),
        Err(msg) => Err(format!("Parse error: {}",  msg))
    }
}
