use std::io::BufferedReader;
use super::Tokenizer;
use ast::Expr;

pub type ReadResult = Result<Expr, String>;

pub fn read_ast<R: Reader>(reader: R) -> ReadResult {
    let mut br = BufferedReader::new(reader);
    let cs = br.chars().map(|c| c.unwrap());
    match super::parse(Tokenizer::new(cs)) {
        Ok(ast)  => Ok(ast),
        Err(msg) => Err(format!("Parse error: {}",  msg))
    }
}
