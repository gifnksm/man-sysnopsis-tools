use token::{Tokenizer, Token, Text, ShortOpt, LongOpt, LBracket, RBracket, LBrace, RBrace, Dots, Bar};
use ast::{Expr, Tok, Seq, Opt, Repeat, Select};

pub fn parse(mut tokenizer: Tokenizer) -> Expr {
    let (expr, next_token) = parse_expr(&mut tokenizer);
    assert_eq!(None, next_token);
    expr
}

fn parse_expr<T: Iterator<Token>>(tokenizer: &mut T) -> (Expr, Option<Token>) {
    let mut v = Vec::new();
    loop {
        let (term, n) = parse_term(&mut *tokenizer);
        v.push(term);
        if n != Some(Bar) {
            if v.len() == 1 {
                return (v.pop().unwrap(), n)
            }
            return (Select(v), n)
        }
    }
}

fn parse_term<T: Iterator<Token>>(tokenizer: &mut T) -> (Expr, Option<Token>) {
    let mut v = Vec::new();
    loop {
        match tokenizer.next() {
            Some(LBracket) => v.push(parse_bracket(&mut *tokenizer)),
            Some(LBrace)   => v.push(parse_brace(&mut *tokenizer)),
            Some(Dots) => {
                // Only last one element is repeated in this implementation.
                let last = v.pop().unwrap();
                v.push(Repeat(~last))
            },
            Some(tok @ Text(_))
                | Some(tok @ ShortOpt(_))
                | Some(tok @ LongOpt(_))
                => v.push(Tok(tok)),
            n => {
                if v.len() == 1 {
                    return (v.pop().unwrap(), n)
                }
                return (Seq(v), n)
            }
        }
    }
}

fn parse_bracket<T: Iterator<Token>>(tokenizer: &mut T) -> Expr {
    let (expr, c) = parse_expr(&mut *tokenizer);
    assert_eq!(Some(RBracket), c);
    Opt(~expr)
}

fn parse_brace<T: Iterator<Token>>(tokenizer: &mut T) -> Expr {
    let (expr, c) = parse_expr(&mut *tokenizer);
    assert_eq!(Some(RBrace), c);
    expr
}

#[cfg(test)]
mod tests {
    use token::{Tokenizer, Text, ShortOpt, LongOpt};
    use ast::{Expr, Tok, Seq, Opt, Repeat, Select};

    fn parse(s: &str) -> Expr {
        let p  = super::parse(Tokenizer::new(s));
        let pp = super::parse(Tokenizer::new(p.pretty()));
        if p != pp {
            println!("{} => {}", s, p);
            println!("{} => {}", p.pretty(), pp);
            assert_eq!(p, pp);
        }
        p
    }

    fn text(s: &str) -> Expr { Tok(Text(s.to_owned())) }
    fn short(s: &str) -> Expr { Tok(ShortOpt(s.to_owned())) }
    fn long(s: &str) -> Expr { Tok(LongOpt(s.to_owned())) }

    #[test]
    fn seq_one() {
        assert_eq!(short("a"), parse("-a"));
    }
    #[test]
    fn seq_multi() {
        assert_eq!(Seq(vec!(short("a"), short("b"), text("c"), long("foo"))),
                   parse("-a -b c --foo"));
    }
    #[test]
    fn seq_empty() {
        assert_eq!(Seq(vec!()), parse(""));
    }
    #[test]
    fn set_nested() {
        assert_eq!(Seq(vec!(Seq(vec!(text("a"), text("b"))), text("c"))),
                   parse("{a b} c"));
    }

    #[test]
    fn opt() { assert_eq!(Opt(~text("aaa")), parse("[aaa]")); }
    #[test]
    fn opt_nested() {
        assert_eq!(Opt(~Seq(vec!(text("a"), Opt(~text("b")), text("c")))), parse("[a[b]c]"));
        assert_eq!(Opt(~Opt(~text("a"))), parse("[[a]]"));
    }
    #[test]
    fn opt_empty() { assert_eq!(Opt(~Seq(vec!())), parse("[]")); }
    #[test]
    fn opt_nested_empty() { assert_eq!(Opt(~Opt(~Seq(vec!()))), parse("[[]]")); }

    #[test]
    fn repeat() {
        assert_eq!(Seq(vec!(text("aaa"), Repeat(~text("bbb")))), parse("aaa bbb ..."));
        assert_eq!(Repeat(~Repeat(~text("aaa"))), parse("aaa ... ..."));
    }
    #[test]
    fn repeat_with_group() {
        assert_eq!(Seq(vec!(text("aaa"), Repeat(~text("bbb")))), parse("aaa {bbb}..."));
        assert_eq!(Repeat(~Seq(vec!(text("aaa"), text("bbb")))), parse("{aaa bbb}..."));
        assert_eq!(Repeat(~Opt(~text("aaa"))), parse("[aaa]..."));
    }
    #[test]
    #[should_fail]
    fn empty_repeat() { parse("..."); }

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
        assert_eq!(Select(vec!(Opt(~text("b")), text("ccc"))), parse("[b]|ccc"));
        assert_eq!(Select(vec!(Opt(~text("b")), Opt(~text("ccc")))), parse("[b]|[ccc]"));
        assert_eq!(Select(vec!(text("a"), Opt(~text("b")), text("ccc"))), parse("a|[b]|ccc"));
        assert_eq!(Select(vec!(text("a"), Opt(~Select(vec!(text("b"), text("c")))), text("ccc"))),
                   parse("a|[b|c]|ccc"));
        assert_eq!(Seq(vec!(text("a"), Opt(~Select(vec!(text("b"), text("c")))), text("d"))),
                   parse("a [b|c] d"));
    }

    #[test]
    fn bar_empty() {
        assert_eq!(Select(vec!(text("a"), Seq(vec!()), text("c"))), parse("a||c"));
        assert_eq!(Seq(vec!(text("a"), Select(vec!(Seq(vec!()), text("a"))), text("c"))),
                   parse("a{|a}c"));
        assert_eq!(Seq(vec!(text("a"), Select(vec!(Seq(vec!()), Seq(vec!()))), text("c"))),
                   parse("a{|}c"));
    }
    #[test]
    fn bar_nested() {
        assert_eq!(Select(vec!(text("a"), Select(vec!(text("b"), text("c"))))),
                   parse("a|{b|c}"));
    }
    #[test]
    #[should_fail]
    fn unclosed_brace() { parse("{a b"); }
    #[test]
    #[should_fail]
    fn unclosed_bracket() { parse("[a |b"); }
    #[test]
    #[should_fail]
    fn unbaranced_parens() { parse("[a b}"); }
}
