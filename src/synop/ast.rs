use token::Token;

#[deriving(Eq, Show, Clone)]
pub enum Expr {
    Tok(Token),
    Seq(Vec<Expr>),
    Opt(~Expr),
    Repeat(~Expr),
    Select(Vec<Expr>)
}

impl Expr {
    pub fn pretty(&self) -> ~str {
        match *self {
            Tok(ref t) => t.pretty(),
            Seq(ref s) => {
                s.iter()
                    .map(|expr| {
                        let p = expr.pretty();
                        match *expr {
                            Tok(_) | Opt(_) | Repeat(_) => p,
                            Seq(_) | Select(_) => format!("\\{{}\\}", p)
                        }
                    }).collect::<~[~str]>()
                    .connect(" ")
            },
            Opt(ref e) => format!("[{}]", e.pretty()),
            Repeat(ref e) => {
                let p = e.pretty();
                match **e {
                    Tok(_) | Opt(_) | Repeat(_) => format!("{}...", p),
                    Seq(_) | Select(_) => format!("\\{{}\\}...", p),
                }
            },
            Select(ref s) => {
                s.iter()
                    .map(|expr| {
                        let p = expr.pretty();
                        match *expr {
                            Select(_) => format!("\\{{}\\}", p),
                            _ => p
                        }
                    })
                    .collect::<~[~str]>()
                    .connect(" | ")
            }
        }
    }

    pub fn normalize(&self) -> Option<Expr> {
        match *self {
            Seq(ref xs) => {
                let mut v = Vec::new();
                for x in xs.iter().filter_map(|x| x.normalize()) {
                    match x {
                        Seq(x) => v.push_all_move(x),
                        _      => v.push(x)
                    }
                }
                if v.is_empty() {
                    None
                } else if v.len() == 1 {
                    Some(v.pop().unwrap())
                } else {
                    Some(Seq(v))
                }
            }
            Opt(ref x) => {
                x.normalize().map(|x| {
                    match x {
                        Opt(x) => Opt(x),
                        _      => Opt(~x)
                    }
                })
            }
            Repeat(~Repeat(ref x)) => {
                x.normalize().map(|x| {
                    match x {
                        Repeat(x) => Repeat(x),
                        _ => Repeat(~x)
                    }
                })
            },
            Select(ref xs) => {
                let mut has_opt = false;
                let mut v = Vec::new();
                for x in xs.iter().filter_map(|x| x.normalize()) {
                    match x {
                        Select(x) => v.push_all_move(x),
                        Opt(~Select(x)) => {
                            has_opt = true;
                            v.push_all_move(x)
                        },
                        Opt(x) => {
                            has_opt = true;
                            v.push(*x)
                        }
                        _ => v.push(x)
                    }
                }
                if v.is_empty() {
                    None
                } else if v.len() == 1 {
                    Some(v.pop().unwrap())
                } else {
                    Some(Select(v))
                }.map(|sel| {
                    if has_opt {
                        Opt(~sel)
                    } else {
                        sel
                    }
                })
            }
            _ => Some(self.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Expr, Tok, Seq, Opt, Repeat, Select};
    use parser;
    use token::{Tokenizer, Text};

    #[test]
    fn pretty_normalized() {
        fn check(s: &str) {
            let parsed = parser::parse(Tokenizer::new(s));
            let pretty = parsed.pretty();
            assert_eq!(s.to_owned(), pretty);
            assert_eq!(parsed, parser::parse(Tokenizer::new(pretty)));
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

    #[test]
    fn normalize() {
        fn check(result: Option<Expr>, input: Expr) {
            assert_eq!(result, input.normalize());
        }

        check(Some(Tok(Text(~"aa"))), Seq(vec!(Tok(Text(~"aa")))));
        check(None, Seq(vec!()));
        check(None, Opt(~Seq(vec!())));
        check(None, Opt(~Opt(~Seq(vec!()))));
        check(None, Opt(~Opt(~Opt(~Seq(vec!())))));
        check(Some(Repeat(~Tok(Text(~"aa")))), Repeat(~Repeat(~Tok(Text(~"aa")))));
        check(Some(Tok(Text(~"aa"))), Select(vec!(Tok(Text(~"aa")))));
        check(Some(Seq(vec!(Tok(Text(~"a")), Tok(Text(~"b")), Tok(Text(~"c"))))),
              Seq(vec!(Seq(vec!(Tok(Text(~"a")), Tok(Text(~"b")))), Tok(Text(~"c")))));
    }
}