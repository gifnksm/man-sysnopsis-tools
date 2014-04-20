use std::mem;
use token::Token;

#[deriving(Eq, Show, Clone)]
pub enum Expr {
    Tok(Token),
    Seq(~[~Expr]),
    Opt(~Expr),
    Repeat(~Expr),
    Select(~[~Expr])
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

    pub fn new_tok(tok: Token) -> Expr { Tok(tok) }

    pub fn new_seq(v: Vec<~Expr>) -> Expr {
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

    pub fn new_opt(expr: Expr) -> Expr {
        match expr {
            Opt(x) => Opt(x),
            x      => Opt(~x)
        }
    }

    pub fn new_repeat(expr: Expr) -> Expr {
        match expr {
            Repeat(x) => Repeat(x),
            _         => Repeat(~expr)
        }
    }

    pub fn new_select(mut  v: Vec<~Expr>) -> Expr {
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
    use parser;
    use token::Tokenizer;

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
}
