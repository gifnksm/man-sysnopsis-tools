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
        v.push(~term);
        if n != Some(Bar) {
            if v.len() == 1 {
                return (*v.pop().unwrap(), n)
            }
            return (Select(v.move_iter().collect()), n)
        }
    }
}

fn parse_term<T: Iterator<Token>>(tokenizer: &mut T) -> (Expr, Option<Token>) {
    let mut v = Vec::new();
    loop {
        match tokenizer.next() {
            Some(LBracket) => v.push(~parse_bracket(&mut *tokenizer)),
            Some(LBrace)   => v.push(~parse_brace(&mut *tokenizer)),
            Some(Dots) => {
                // Only last one element is repeated in this implementation.
                let last = v.pop().unwrap();
                v.push(~Repeat(last))
            },
            Some(tok @ Text(_))
                | Some(tok @ ShortOpt(_))
                | Some(tok @ LongOpt(_))
                => v.push(~Tok(tok)),
            n => {
                if v.len() == 1 {
                    return (*v.pop().unwrap(), n)
                }
                return (Seq(v.move_iter().collect()), n)
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
        let p = super::parse(Tokenizer::new(s));
        let pp = super::parse(Tokenizer::new(p.pretty()));
        if p != pp {
            println!("{} => {}", s, p);
            println!("{} => {}", p.pretty(), pp);
            assert_eq!(p, pp);
        }
        p
    }

    fn parse_normalized(s: &str) -> Expr {
        let p = super::parse(Tokenizer::new(s));
        let pp = super::parse(Tokenizer::new(p.pretty()));
        if p != pp {
            println!("{} => {}", s, p);
            println!("{} => {}", p.pretty(), pp);
            assert_eq!(p, pp);
        }

        let pn = super::parse(Tokenizer::new(s)).normalize().unwrap();
        let ppn = super::parse(Tokenizer::new(pn.pretty())).normalize().unwrap();
        if pn != ppn {
            println!("{} => {}", s, pn);
            println!("{} => {}", pn.pretty(), ppn);
            assert_eq!(pn, ppn);
        }
        pn
    }
    fn text(s: ~str) -> Expr { Tok(Text(s)) }
    fn short(s: ~str) -> Expr { Tok(ShortOpt(s)) }
    fn long(s: ~str) -> Expr { Tok(LongOpt(s)) }

    #[test]
    fn seq_one() {
        assert_eq!(short(~"a"), parse("-a"));
    }
    #[test]
    fn seq_multi() {
        assert_eq!(Seq(~[~short(~"a"), ~short(~"b"), ~text(~"c"), ~long(~"foo")]),
                   parse("-a -b c --foo"));
    }
    #[test]
    fn seq_empty() {
        assert_eq!(Seq(~[]), parse(""));
    }
    #[test]
    fn set_nested() {
        assert_eq!(Seq(~[~Seq(~[~text(~"a"), ~text(~"b")]), ~text(~"c")]),
                   parse("{a b} c"));
    }

    #[test]
    fn opt() { assert_eq!(Opt(~text(~"aaa")), parse_normalized("[aaa]")); }

    #[test]
    fn opt_nested() {
        assert_eq!(Opt(~Seq(~[~text(~"a"), ~Opt(~text(~"b")), ~text(~"c")])), parse_normalized("[a[b]c]"));
        assert_eq!(Opt(~text(~"a")), parse_normalized("[[a]]"));
    }
    #[test]
    fn opt_empty() { assert_eq!(Opt(~Seq(~[])), parse("[]")); }
    #[test]
    fn opt_nested_empty() { assert_eq!(Opt(~Opt(~Seq(~[]))), parse("[[]]")); }
    #[test]
    #[should_fail]
    fn empty_opt() { parse_normalized("[]"); }
    #[test]
    #[should_fail]
    fn empty_nested_opt() { parse_normalized("[[]]"); }

    #[test]
    fn repeat() {
        assert_eq!(Seq(~[~text(~"aaa"), ~Repeat(~text(~"bbb"))]), parse_normalized("aaa bbb ..."));
        assert_eq!(Repeat(~text(~"aaa")), parse_normalized("aaa ... ..."));
    }
    #[test]
    fn repeat_with_group() {
        assert_eq!(Seq(~[~text(~"aaa"), ~Repeat(~text(~"bbb"))]), parse_normalized("aaa {bbb}..."));
        assert_eq!(Repeat(~Seq(~[~text(~"aaa"), ~text(~"bbb")])), parse_normalized("{aaa bbb}..."));
        assert_eq!(Repeat(~Opt(~text(~"aaa"))), parse_normalized("[aaa]..."));
    }
    #[test]
    #[should_fail]
    fn empty_repeat() { parse("..."); }

    #[test]
    fn bar() {
        assert_eq!(Select(~[~text(~"a"), ~text(~"b")]),
                   parse_normalized("a|b"));
        assert_eq!(Select(~[~text(~"a"), ~text(~"b"), ~text(~"c")]),
                   parse_normalized("a|b|c"));
        assert_eq!(Seq(~[~text(~"a"), ~Select(~[~text(~"b"), ~text(~"c")]), ~text(~"d")]),
                   parse_normalized("a { b | c } d"));
        assert_eq!(Select(~[~text(~"a"), ~Seq(~[~text(~"b"), ~text(~"c")]), ~text(~"d")]),
                   parse_normalized("a|b c|d"));
        assert_eq!(Select(~[~Seq(~[~text(~"a"), ~text(~"b")]),
                            ~Seq(~[~text(~"c"), ~text(~"d")]),
                            ~Seq(~[~text(~"e"), ~text(~"f")])]),
                   parse_normalized("a b|c d|e f"));
        assert_eq!(Opt(~Select(~[~text(~"b"), ~text(~"ccc")])),
                   parse_normalized("[b]|ccc"));
        assert_eq!(Opt(~Select(~[~text(~"b"), ~text(~"ccc")])),
                   parse_normalized("[b]|[ccc]"));
        assert_eq!(Opt(~Select(~[~text(~"a"), ~text(~"b"), ~text(~"ccc")])),
                   parse_normalized("a|[b]|ccc"));
        assert_eq!(Opt(~Select(~[~text(~"a"), ~text(~"b"), ~text(~"ccc")])),
                   parse_normalized("a|[b]|[ccc]"));
        assert_eq!(Opt(~Select(~[~text(~"a"), ~text(~"b"), ~text(~"ccc")])),
                   parse_normalized("[a]|[b]|ccc"));
        assert_eq!(Opt(~Select(~[~text(~"a"), ~text(~"b"), ~text(~"ccc")])),
                   parse_normalized("[a]|[b]|[ccc]"));
        assert_eq!(Opt(~Select(~[~text(~"a"), ~text(~"b"), ~text(~"c"), ~text(~"ccc")])),
                   parse_normalized("a|[b|c]|ccc"));
        assert_eq!(Opt(~Select(~[~text(~"a"), ~Seq(~[~text(~"b"), ~text(~"c")]), ~text(~"ccc")])),
                   parse_normalized("a|[b c]|ccc"));
        assert_eq!(Seq(~[~text(~"a"), ~Opt(~Select(~[~text(~"b"), ~text(~"c")])), ~text(~"d")]),
                   parse_normalized("a [b|c] d"));
   }

    #[test]
    fn bar_empty() {
        assert_eq!(Select(~[~text(~"a"), ~Seq(~[]), ~text(~"c")]), parse("a||c"));
        assert_eq!(Seq(~[~text(~"a"), ~Select(~[~Seq(~[]), ~text(~"a")]), ~text(~"c")]),
                   parse("a{|a}c"));
        assert_eq!(Seq(~[~text(~"a"), ~Select(~[~Seq(~[]), ~Seq(~[])]), ~text(~"c")]),
                   parse("a{|}c"));
    }
    #[test]
    fn bar_nested() {
        assert_eq!(Select(~[~text(~"a"), ~Select(~[~text(~"b"), ~text(~"c")])]),
                   parse("a|{b|c}"));
    }
    #[test]
    #[should_fail]
    fn unclosed_brace() { parse_normalized("{a b"); }
    #[test]
    #[should_fail]
    fn unclosed_bracket() { parse_normalized("[a |b"); }
    #[test]
    #[should_fail]
    fn unbaranced_parens() { parse_normalized("[a b}"); }
}
