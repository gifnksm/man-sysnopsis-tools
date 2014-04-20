use std::iter::Peekable;
use std::str::Chars;

#[deriving(Eq, Show, Clone)]
pub enum Token {
    Text(~str),
    ShortOpt(~str),
    LongOpt(~str),
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Dots,
    Bar
}

impl Token {
    pub fn pretty(&self) -> ~str {
        match *self {
            Text(ref s) => s.to_owned(),
            ShortOpt(ref s) => format!("-{}", s),
            LongOpt(ref s) => format!("--{}", s),
            LBracket => ~"[",
            RBracket => ~"]",
            LBrace   => ~"{",
            RBrace   => ~"}",
            Dots     => ~"...",
            Bar      => ~"|"
        }
    }
}

pub struct Tokenizer<'a> { input: Peekable<char, Chars<'a>> }

fn is_option_char(c: char) -> bool { c.is_alphanumeric() || c == '-' || c == '_' }

impl<'a> Tokenizer<'a> {
    #[inline]
    pub fn new(input: &'a str) -> Tokenizer<'a> { Tokenizer { input: input.chars().peekable() } }

    fn push_while(&mut self, buf: &mut StrBuf, pred: |char| -> bool) {
        loop {
            match self.input.peek() {
                Some(&c) => {
                    if !pred(c) { break; }
                    buf.push_char(c);
                    self.input.next();
                },
                None => break
            }
        }
    }
}

impl<'a> Iterator<Token> for Tokenizer<'a> {
    fn next(&mut self) -> Option<Token> {
        match self.input.by_ref().skip_while(|&c| c.is_whitespace()).next() {
            Some('-') => {
                let tok = if self.input.peek() == Some(&'-') {
                    self.input.next();
                    LongOpt
                } else {
                    ShortOpt
                };

                let mut s = StrBuf::new();
                self.push_while(&mut s, is_option_char);
                Some(tok(s.to_str()))
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
                let mut s = StrBuf::new();
                s.push_char(c);
                self.push_while(&mut s, is_option_char);
                Some(Text(s.to_str()))
            },
            None => None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Tokenizer, Token, Text, ShortOpt, LongOpt,
                LBracket, RBracket, LBrace, RBrace, Dots, Bar};

    fn check(output: &[Token], input: &str) {
        assert_eq!(output.to_owned(), FromIterator::from_iter(Tokenizer::new(input)))
    }

    #[test]
    fn short_opt() {
        check([ShortOpt(~"")], "-");
        check([ShortOpt(~"a")], "-a");
        check([ShortOpt(~"a")], "  -a  ");
        check([ShortOpt(~"a"), ShortOpt(~"b"), ShortOpt(~"c"), ShortOpt(~"1")],
              "-a -b -c -1");
        check([ShortOpt(~"a"), ShortOpt(~"b"), ShortOpt(~"c"), ShortOpt(~"1")],
              "  -a -b   -c   -1  ");
    }

    #[test]
    fn long_opt() {
        check([LongOpt(~"")], "--");
        check([LongOpt(~"long")], "--long");
        check([LongOpt(~"aaa"), LongOpt(~"bbb"), LongOpt(~"ccc"), LongOpt(~"123")],
              "--aaa --bbb --ccc --123");
        check([LongOpt(~"aaa"), LongOpt(~"bbb"), LongOpt(~"ccc"), LongOpt(~"123")],
              "  --aaa --bbb   --ccc --123");
        check([LongOpt(~"aaa"), LongOpt(~"bbb"), LongOpt(~"ccc--1_23")],
              "  --aaa --bbb   --ccc--1_23");
    }

    #[test]
    fn mixed() {
        check([ShortOpt(~"a"), LBrace, Text(~"a"), Bar, Text(~"b"), Bar, Text(~"c"), RBrace,
               LBracket, Text(~"p"), Dots, RBracket],
              "-a {a|b|c} [p ...]")
    }

    #[test]
    #[should_fail]
    fn invalid_dots() {
        for _tok in Tokenizer::new("....") {}
    }

    #[test]
    fn pretty() {
        fn check(s: &str) {
            let mut tok = Tokenizer::new(s);
            assert_eq!(s.to_owned(), tok.next().unwrap().pretty());
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