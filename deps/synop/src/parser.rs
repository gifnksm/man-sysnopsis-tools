use token::{Tokenizer, Token};
use token::Token::*;
use ast::Expr;
use ast::Expr::*;

pub type ParseResult<T> = Result<T, String>;

pub fn parse<T: Iterator<Item = char>>(mut tokenizer: Tokenizer<T>) -> ParseResult<Expr> {
    let (expr, next_token) = try!(parse_expr(&mut tokenizer));
    if next_token != None {
        return Err(unexpected_msg(&next_token.unwrap()));
    }
    Ok(expr)
}

fn parse_expr<T: Iterator<Item = Token>>(tokenizer: &mut T) -> ParseResult<(Expr, Option<Token>)> {
    let mut v = Vec::new();
    loop {
        let (term, n) = try!(parse_term(&mut *tokenizer));
        v.push(term);
        if n != Some(Bar) {
            if v.len() == 1 {
                return Ok((v.pop().unwrap(), n))
            }
            return Ok((Select(v), n))
        }
    }
}

fn parse_term<T: Iterator<Item = Token>>(tokenizer: &mut T) -> ParseResult<(Expr, Option<Token>)> {
    let mut v = Vec::new();
    loop {
        match tokenizer.next() {
            Some(LBracket) => v.push(try!(parse_bracket(&mut *tokenizer))),
            Some(LBrace)   => v.push(try!(parse_brace(&mut *tokenizer))),
            Some(Dots) => {
                // Only last one element is repeated in this implementation.
                match v.pop() {
                    Some(last) => v.push(Repeat(Box::new(last))),
                    None => return Err(unexpected_msg(&Dots))
                }
            },
            Some(tok @ Text(_))
                | Some(tok @ ShortOpt(_))
                | Some(tok @ LongOpt(_))
                => v.push(Tok(tok)),
            n => {
                if v.len() == 1 {
                    return Ok((v.pop().unwrap(), n))
                }
                return Ok((Seq(v), n))
            }
        }
    }
}

fn parse_bracket<T: Iterator<Item = Token>>(tokenizer: &mut T) -> ParseResult<Expr> {
    let (expr, c) = try!(parse_expr(&mut *tokenizer));
    try!(expect_token(&RBracket, &c));
    Ok(Opt(Box::new(expr)))
}

fn parse_brace<T: Iterator<Item = Token>>(tokenizer: &mut T) -> ParseResult<Expr> {
    let (expr, c) = try!(parse_expr(&mut *tokenizer));
    try!(expect_token(&RBrace, &c));
    Ok(expr)
}

fn expect_token(expect: &Token, actual: &Option<Token>) -> ParseResult<()> {
    match *actual {
        Some(ref ac) => {
            if ac != expect {
                return Err(format!("expected `{}`, found `{}`", expect.pretty(), ac.pretty()))
            }
        }
        None => return Err(format!("expected `{}`, found EOF", expect.pretty()))
    }
    Ok(())
}

fn unexpected_msg(unexpect: &Token) -> String {
    format!("unexpected token `{}` found", unexpect.pretty())
}

#[cfg(test)]
mod tests {
    use token::Tokenizer;
    use token::Token::*;
    use ast::Expr;
    use ast::Expr::*;

    fn parse(s: &str) -> Expr {
        let p  = super::parse(Tokenizer::new(s.chars())).unwrap();
        let pp = super::parse(Tokenizer::new(p.pretty().chars())).unwrap();
        assert_eq!(p, pp);
        p
    }
    fn parse_err(s: &str) -> String {
        super::parse(Tokenizer::new(s.chars())).unwrap_err()
    }

    fn text(s: &str) -> Expr { Tok(Text(s.to_string())) }
    fn short(s: &str) -> Expr { Tok(ShortOpt(s.to_string())) }
    fn long(s: &str) -> Expr { Tok(LongOpt(s.to_string())) }

    #[test]
    fn seq_one() {
        assert_eq!(short("a"), parse("-a"));
    }
    #[test]
    fn seq_multi() {
        assert_eq!(Seq(vec![short("a"), short("b"), text("c"), long("foo")]),
                   parse("-a -b c --foo"));
    }
    #[test]
    fn seq_empty() {
        assert_eq!(Seq(vec![]), parse(""));
    }
    #[test]
    fn set_nested() {
        assert_eq!(Seq(vec![Seq(vec![text("a"), text("b")]), text("c")]),
                   parse("{a b} c"));
    }

    #[test]
    fn opt() { assert_eq!(Opt(Box::new(text("aaa"))), parse("[aaa]")); }
    #[test]
    fn opt_nested() {
        assert_eq!(Opt(Box::new(Seq(vec![text("a"), Opt(Box::new(text("b"))), text("c")]))),
                   parse("[a[b]c]"));
        assert_eq!(Opt(Box::new(Opt(Box::new(text("a"))))),
                   parse("[[a]]"));
    }
    #[test]
    fn opt_empty() { assert_eq!(Opt(Box::new(Seq(vec![]))),
                                parse("[]")); }
    #[test]
    fn opt_nested_empty() { assert_eq!(Opt(Box::new(Opt(Box::new(Seq(vec![]))))),
                                           parse("[[]]")); }

    #[test]
    fn repeat() {
        assert_eq!(Seq(vec![text("aaa"), Repeat(Box::new(text("bbb")))]),
                   parse("aaa bbb ..."));
        assert_eq!(Repeat(Box::new(Repeat(Box::new(text("aaa"))))),
                   parse("aaa ... ..."));
    }
    #[test]
    fn repeat_with_group() {
        assert_eq!(Seq(vec![text("aaa"), Repeat(Box::new(text("bbb")))]), parse("aaa {bbb}..."));
        assert_eq!(Repeat(Box::new(Seq(vec![text("aaa"), text("bbb")]))), parse("{aaa bbb}..."));
        assert_eq!(Repeat(Box::new(Opt(Box::new(text("aaa"))))), parse("[aaa]..."));
    }
    #[test]
    fn empty_repeat() { assert_eq!("unexpected token `...` found".to_string(), parse_err("...")); }

    #[test]
    fn bar() {
        assert_eq!(Select(vec!(text("a"), text("b"))), parse("a|b"));
        assert_eq!(Select(vec!(text("a"), text("b"), text("c"))), parse("a|b|c"));
        assert_eq!(Seq(vec!(text("a"), Select(vec!(text("b"), text("c"))), text("d"))),
                   parse("a { b | c } d"));
        assert_eq!(Select(vec!(text("a"), Seq(vec!(text("b"), text("c"))), text("d"))),
                   parse("a|b c|d"));
        assert_eq!(Select(vec!(Seq(vec!(text("a"), text("b"))),
                               Seq(vec!(text("c"), text("d"))),
                               Seq(vec!(text("e"), text("f"))))),
                   parse("a b|c d|e f"));
        assert_eq!(Select(vec!(Opt(Box::new(text("b"))), text("ccc"))), parse("[b]|ccc"));
        assert_eq!(Select(vec!(Opt(Box::new(text("b"))), Opt(Box::new(text("ccc"))))), parse("[b]|[ccc]"));
        assert_eq!(Select(vec!(text("a"), Opt(Box::new(text("b"))), text("ccc"))), parse("a|[b]|ccc"));
        assert_eq!(Select(vec!(text("a"), Opt(Box::new(Select(vec![text("b"), text("c")]))), text("ccc"))),
                   parse("a|[b|c]|ccc"));
        assert_eq!(Seq(vec![text("a"), Opt(Box::new(Select(vec![text("b"), text("c")]))), text("d")]),
                   parse("a [b|c] d"));
    }

    #[test]
    fn bar_empty() {
        assert_eq!(Select(vec!(text("a"), Seq(vec![]), text("c"))), parse("a||c"));
        assert_eq!(Seq(vec!(text("a"), Select(vec!(Seq(vec![]), text("a"))), text("c"))),
                   parse("a{|a}c"));
        assert_eq!(Seq(vec!(text("a"), Select(vec!(Seq(vec![]), Seq(vec![]))), text("c"))),
                   parse("a{|}c"));
    }
    #[test]
    fn bar_nested() {
        assert_eq!(Select(vec!(text("a"), Select(vec!(text("b"), text("c"))))),
                   parse("a|{b|c}"));
    }
    #[test]
    fn unclosed_brace() { assert_eq!("expected `}`, found EOF".to_string(), parse_err("{a b")) }
    #[test]
    fn unclosed_bracket() { assert_eq!("expected `]`, found EOF".to_string(), parse_err("[a |b")); }
    #[test]
    fn unbaranced_parens() { assert_eq!("expected `]`, found `}`".to_string(), parse_err("[a b}")); }
    #[test]
    fn close_only() { assert_eq!("unexpected token `}` found".to_string(), parse_err("a }")) }
}
