use std::mem;
use token::{Tokenizer, Token, Text, ShortOpt, LongOpt, LBracket, RBracket, LBrace, RBrace, Dots, Bar};

#[deriving(Eq, Show, Clone)]
pub enum Expr {
    Tok(Token),
    Seq(~[~Expr]),
    Opt(~Expr),
    Repeat(~Expr),
    Select(~[~Expr])
}

pub fn parse(mut tokenizer: Tokenizer) -> Expr {
    let (expr, next_token) = Expr::parse(&mut tokenizer);
    assert_eq!(None, next_token);
    expr
}

impl Expr {
    pub fn pretty(&self) -> ~str {
        match *self {
            Tok(ref t) => t.pretty(),
            Seq(ref s) => {
                s.iter()
                    .map(|expr| {
                        let p = expr.pretty();
                        match **expr {
                            Tok(_) | Opt(_) | Repeat(_) => p,
                            Select(_) => format!("\\{{}\\}", p),
                            _ => fail!("{}", expr)
                        }
                    }).collect::<~[~str]>()
                    .connect(" ")
            },
            Opt(ref e) => format!("[{}]", e.pretty()),
            Repeat(ref e) => {
                let p = e.pretty();
                match **e {
                    Tok(_) => format!("{}...", p),
                    Seq(_) | Select(_) => format!("\\{{}\\}...", p),
                    _ => fail!("{}", e)
                }
            },
            Select(ref s) => {
                s.iter()
                    .map(|expr| expr.pretty())
                    .collect::<~[~str]>()
                    .connect(" | ")
            }
        }
    }

    fn parse<T: Iterator<Token>>(tokenizer: &mut T) -> (Expr, Option<Token>) {
        let mut v = Vec::new();
        loop {
            let (term, n) = Expr::parse_term(&mut *tokenizer);
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
                Some(LBracket) => v.push(~Expr::parse_bracket(&mut *tokenizer)),
                Some(LBrace)   => v.push(~Expr::parse_brace(&mut *tokenizer)),
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
        let (expr, c) = Expr::parse(&mut *tokenizer);
        assert_eq!(Some(RBracket), c);
        Expr::new_opt(expr)
    }
    fn parse_brace<T: Iterator<Token>>(tokenizer: &mut T) -> Expr {
        let (expr, c) = Expr::parse(&mut *tokenizer);
        assert_eq!(Some(RBrace), c);
        expr
    }

    fn new_tok(tok: Token) -> Expr { Tok(tok) }

    fn new_seq(v: Vec<~Expr>) -> Expr {
        if v.is_empty() {
            fail!("Empty seq found. {}", v)
        }

        let mut output = v.move_iter().flat_map(|e| {
            match e {
                ~Seq(x) => x.move_iter(),
                x       => (~[x]).move_iter()
            }
        }).collect::<Vec<~Expr>>();

        if output.len() == 1 {
            *output.shift().unwrap()
        } else {
            Seq(output.as_slice().to_owned())
        }
    }

    fn new_opt(expr: Expr) -> Expr {
        match expr {
            Opt(x) => Opt(x),
            x      => Opt(~x)
        }
    }

    fn new_repeat(expr: Expr) -> Expr {
        match expr {
            Repeat(x) => Repeat(x),
            _         => Repeat(~expr)
        }
    }

    fn new_select(mut  v: Vec<~Expr>) -> Expr {
        if v.len() == 1 { return *v.pop().unwrap() }

        let mut has_opt = false;
        let mut sel = v.move_iter().map(|e| {
            match e {
                ~Opt(x) => { has_opt = true; x },
                x => x
            }
        }).collect::<Vec<_>>();
        if has_opt { return Expr::new_opt(Expr::new_select(sel)) }

        for e in mem::replace(&mut sel, Vec::new()).move_iter() {
            match e {
                ~Select(arg) => sel.extend(arg.move_iter()),
                x => sel.push(x)
            }
        }
        Select(sel.as_slice().to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::{Expr, Tok, Seq, Opt, Repeat, Select};
    use token::{Tokenizer, Text, ShortOpt, LongOpt};

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

    #[test]
    fn pretty_normalized() {
        fn check(s: &str) {
            let parsed = parse(s);
            let pretty = parsed.pretty();
            assert_eq!(s.to_owned(), pretty);
            assert_eq!(parsed, parse(pretty));
        }
        check("a");
        check("-b");
        check("-a");
        check("a b c");
        check("a [b] c");
        check("a b...");
        check("a | b...");
        check("{a | b}...");
        check("[a] {a | b}...");
    }
}
