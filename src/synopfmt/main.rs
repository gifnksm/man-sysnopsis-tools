#![crate_id = "synopfmt"]

extern crate synop;

#[cfg(not(test))]
use synop::Tokenizer;

#[cfg(not(test))]
fn main() {
    let mut stdin = std::io::stdin();
    let cs = stdin.chars().map(|c| c.unwrap());
    let ast = match synop::parse(Tokenizer::new(cs)) {
        Ok(ast) => ast,
        Err(msg) => {
            let _ =writeln!(&mut std::io::stderr(), "Parse error: {}", msg);
            return
        }
    };
    match ast.normalize() {
        Some(x) => println!("{}", x.pretty()),
        None    => println!("")
    }
}
