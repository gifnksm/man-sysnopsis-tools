use token::{Tokenizer, Token, Text, ShortOpt, LongOpt, LBracket, RBracket, LBrace, RBrace, Dots, Bar};
use ast::Expr;

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
            return (Expr::new_select(v), n)
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
                v.push(~Expr::new_repeat(*last))
            },
            Some(tok @ Text(_))
                | Some(tok @ ShortOpt(_))
                | Some(tok @ LongOpt(_))
                => v.push(~Expr::new_tok(tok)),
            n => return (Expr::new_seq(v), n)
        }
    }
}

fn parse_bracket<T: Iterator<Token>>(tokenizer: &mut T) -> Expr {
    let (expr, c) = parse_expr(&mut *tokenizer);
    assert_eq!(Some(RBracket), c);
    Expr::new_opt(expr)
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
    fn text(s: ~str) -> Expr { Tok(Text(s)) }
    fn short(s: ~str) -> Expr { Tok(ShortOpt(s)) }
    fn long(s: ~str) -> Expr { Tok(LongOpt(s)) }

    #[test]
    fn seq_one() {
        assert_eq!(short(~"a"), super::parse(Tokenizer::new("-a")));
    }
    #[test]
    fn seq_multi() {
        assert_eq!(Seq(~[~short(~"a"), ~short(~"b"), ~text(~"c"), ~long(~"foo")]),
                   parse("-a -b c --foo"));
    }
    #[test]
    #[should_fail]
    fn empty_seq() { parse(""); }

    #[test]
    fn opt() { assert_eq!(Opt(~text(~"aaa")), parse("[aaa]")); }
    #[test]
    fn nested_opt() {
        assert_eq!(Opt(~Seq(~[~text(~"a"), ~Opt(~text(~"b")), ~text(~"c")])), parse("[a[b]c]"));
        assert_eq!(Opt(~text(~"a")), parse("[[a]]"));
    }
    #[test]
    #[should_fail]
    fn empty_opt() { parse("[]"); }
    #[test]
    #[should_fail]
    fn empty_nested_opt() { parse("[[]]"); }

    #[test]
    fn repeat() {
        assert_eq!(Seq(~[~text(~"aaa"), ~Repeat(~text(~"bbb"))]), parse("aaa bbb ..."));
        assert_eq!(Repeat(~text(~"aaa")), parse("aaa ... ..."));
    }
    #[test]
    fn repeat_with_group() {
        assert_eq!(Seq(~[~text(~"aaa"), ~Repeat(~text(~"bbb"))]), parse("aaa {bbb}..."));
        assert_eq!(Repeat(~Seq(~[~text(~"aaa"), ~text(~"bbb")])), parse("{aaa bbb}..."));
    }
    #[test]
    #[should_fail]
    fn empty_repeat() { parse("..."); }

    #[test]
    fn bar() {
        assert_eq!(Select(~[~text(~"a"), ~text(~"b")]),
                   parse("a|b"));
        assert_eq!(Select(~[~text(~"a"), ~text(~"b"), ~text(~"c")]),
                   parse("a|b|c"));
        assert_eq!(Seq(~[~text(~"a"), ~Select(~[~text(~"b"), ~text(~"c")]), ~text(~"d")]),
                   parse("a { b | c } d"));
        assert_eq!(Select(~[~text(~"a"), ~Seq(~[~text(~"b"), ~text(~"c")]), ~text(~"d")]),
                   parse("a|b c|d"));
        assert_eq!(Select(~[~Seq(~[~text(~"a"), ~text(~"b")]),
                            ~Seq(~[~text(~"c"), ~text(~"d")]),
                            ~Seq(~[~text(~"e"), ~text(~"f")])]),
                   parse("a b|c d|e f"));
        assert_eq!(Opt(~Select(~[~text(~"b"), ~text(~"ccc")])),
                   parse("[b]|ccc"));
        assert_eq!(Opt(~Select(~[~text(~"b"), ~text(~"ccc")])),
                   parse("[b]|[ccc]"));
        assert_eq!(Opt(~Select(~[~text(~"a"), ~text(~"b"), ~text(~"ccc")])),
                   parse("a|[b]|ccc"));
        assert_eq!(Opt(~Select(~[~text(~"a"), ~text(~"b"), ~text(~"ccc")])),
                   parse("a|[b]|[ccc]"));
        assert_eq!(Opt(~Select(~[~text(~"a"), ~text(~"b"), ~text(~"ccc")])),
                   parse("[a]|[b]|ccc"));
        assert_eq!(Opt(~Select(~[~text(~"a"), ~text(~"b"), ~text(~"ccc")])),
                   parse("[a]|[b]|[ccc]"));
        assert_eq!(Opt(~Select(~[~text(~"a"), ~text(~"b"), ~text(~"c"), ~text(~"ccc")])),
                   parse("a|[b|c]|ccc"));
        assert_eq!(Opt(~Select(~[~text(~"a"), ~Seq(~[~text(~"b"), ~text(~"c")]), ~text(~"ccc")])),
                   parse("a|[b c]|ccc"));
        assert_eq!(Seq(~[~text(~"a"), ~Opt(~Select(~[~text(~"b"), ~text(~"c")])), ~text(~"d")]),
                   parse("a [b|c] d"));
   }

    #[test]
    #[should_fail]
    fn empty_bar() { parse("a||c"); }
    #[test]
    #[should_fail]
    fn empty_bar_in_bracket() { parse("a{|a}c"); }
    #[test]
    #[should_fail]
    fn empty_bar_in_bracket2() { parse("a{|}c"); }
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
