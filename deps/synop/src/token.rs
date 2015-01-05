use std::iter::Peekable;
use token::Token::*;

#[derive(Eq, PartialEq, Show, Clone)]
pub enum Token {
    Text(String),
    ShortOpt(String),
    LongOpt(String),
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Dots,
    Bar
}

impl Token {
    pub fn pretty(&self) -> String {
        match *self {
            Text(ref s) => s.to_string(),
            ShortOpt(ref s) => format!("-{}", s),
            LongOpt(ref s) => format!("--{}", s),
            LBracket => "[".to_string(),
            RBracket => "]".to_string(),
            LBrace   => "{".to_string(),
            RBrace   => "}".to_string(),
            Dots     => "...".to_string(),
            Bar      => "|".to_string()
        }
    }
}

pub struct Tokenizer<T: Iterator<Item = char>> { input: Peekable<char, T> }

fn is_option_char(c: char) -> bool { c.is_alphanumeric() || c == '-' || c == '_' }

impl<T: Iterator<Item = char>> Tokenizer<T> {
    #[inline]
    pub fn new(input: T) -> Tokenizer<T> { Tokenizer { input: input.peekable() } }

    fn push_while(&mut self, buf: &mut String, pred: |char| -> bool) {
        loop {
            match self.input.peek() {
                Some(&c) => {
                    if !pred(c) { break; }
                    buf.push(c);
                    self.input.next();
                },
                None => break
            }
        }
    }
}

impl<T: Iterator<Item = char>> Iterator for Tokenizer<T> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        match self.input.by_ref().skip_while(|&c| c.is_whitespace()).next() {
            Some('-') => {
                let tok = if self.input.peek() == Some(&'-') {
                    self.input.next();
                    LongOpt as fn(String) -> Token
                } else {
                    ShortOpt as fn(String) -> Token
                };

                let mut s = String::new();
                self.push_while(&mut s, is_option_char);
                Some(tok(s.to_string()))
            },
            Some('[') => Some(LBracket),
            Some(']') => Some(RBracket),
            Some('{') => Some(LBrace),
            Some('}') => Some(RBrace),
            Some('.') => {
                assert_eq!(Some('.'), self.input.next());
                assert_eq!(Some('.'), self.input.next());
                Some(Dots)
            }
            Some('|') => Some(Bar),
            Some(c) => {
                let mut s = String::new();
                s.push(c);
                self.push_while(&mut s, is_option_char);
                Some(Text(s.to_string()))
            },
            None => None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Tokenizer, Token};
    use super::Token::*;

    fn check(output: &[Token], input: &str) {
        let v = Tokenizer::new(input.chars()).collect::<Vec<_>>();
        assert_eq!(output, v.as_slice());
    }
    fn short(s: &str) -> Token { ShortOpt(s.to_string()) }
    fn long(s: &str) -> Token { LongOpt(s.to_string()) }
    fn text(s: &str) -> Token { Text(s.to_string()) }

    #[test]
    fn short_opt() {
        check(&[short("")], "-");
        check(&[short("a")], "-a");
        check(&[short("a")], "  -a  ");
        check(&[short("a"), short("b"), short("c"), short("1")], "-a -b -c -1");
        check(&[short("a"), short("b"), short("c"), short("1")], "  -a -b   -c   -1  ");
    }

    #[test]
    fn long_opt() {
        check(&[long("")], "--");
        check(&[long("long")], "--long");
        check(&[long("aaa"), long("bbb"), long("ccc"), long("123")], "--aaa --bbb --ccc --123");
        check(&[long("aaa"), long("bbb"), long("ccc"), long("123")], "  --aaa --bbb   --ccc --123");
        check(&[long("aaa"), long("bbb"), long("ccc--1_23")], "  --aaa --bbb   --ccc--1_23");
    }

    #[test]
    fn mixed() {
        check(&[short("a"), LBrace, text("a"), Bar, text("b"), Bar, text("c"), RBrace,
               LBracket, text("p"), Dots, RBracket],
              "-a {a|b|c} [p ...]")
    }

    #[test]
    #[should_fail]
    fn invalid_dots() {
        for _tok in Tokenizer::new("....".chars()) {}
    }

    #[test]
    fn pretty() {
        fn check(s: &str) {
            let mut tok = Tokenizer::new(s.chars());
            assert_eq!(s.to_string(), tok.next().unwrap().pretty());
            assert_eq!(None, tok.next());
        }
        check("a");
        check("b");
        check("-a");
        check("--long");
        check("[");
        check("]");
        check("{");
        check("}");
        check("...");
        check("|");
    }
}
