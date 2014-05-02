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
                            Tok(_) | Opt(_) | Repeat(_) | Seq(_) => p
                        }
                    }).collect::<~[~str]>()
                    .connect(" | ")
            }
        }
    }

    pub fn normalize(self) -> Option<Expr> {
        match self {
            Tok(_) => Some(self),
            Seq(xs) => {
                let mut v = xs.move_iter()
                    .filter_map(|x| x.normalize())
                    .map(|x| match x { Seq(y) => y, _ => vec!(x) })
                    .flat_map(|xs| xs.move_iter())
                    .collect::<Vec<_>>();
                match v.len() {
                    0 => None,
                    1 => Some(v.pop().unwrap()),
                    _ => Some(Seq(v))
                }
            }
            Opt(x)    => x.normalize().map(|y| match y { Opt(z)    => z, _ => ~y }).map(Opt),
            Repeat(x) => x.normalize().map(|y| match y { Repeat(z) => z, _ => ~y }).map(Repeat),
            Select(xs) => {
                let mut has_opt = false;
                let mut v = xs.move_iter()
                    .filter_map(|x| x.normalize())
                    .map(|x| match x { Opt(y) => { has_opt = true; *y }, _ => x })
                    .map(|x| match x { Select(y) => y, _ => vec!(x) })
                    .flat_map(|xs| xs.move_iter())
                    .collect::<Vec<_>>();
                let sel = match v.len() {
                    0 => None,
                    1 => Some(v.pop().unwrap()),
                    _ => Some(Select(v))
                };
                if has_opt { sel.map(|x| Opt(~x)) } else { sel }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Expr, Tok, Seq, Opt, Repeat, Select};
    use parser;
    use token::{Tokenizer, Text};

    fn text(s: &str) -> Expr { Tok(Text(s.to_owned())) }

    #[test]
    fn pretty_normalized() {
        fn check(s: &str) {
            let parsed = parser::parse(Tokenizer::new(s.chars()));
            let pretty = parsed.pretty();
            assert_eq!(s.to_owned(), pretty);
            assert_eq!(parsed, parser::parse(Tokenizer::new(pretty.chars())));
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

        check(Some(text("aa")), Seq(vec!(text("aa"))));
        check(None, Seq(vec!()));
        check(None, Opt(~Seq(vec!())));
        check(None, Opt(~Opt(~Seq(vec!()))));
        check(None, Opt(~Opt(~Opt(~Seq(vec!())))));
        check(Some(Repeat(~text("aa"))), Repeat(~Repeat(~text("aa"))));
        check(Some(text("aa")), Select(vec!(text("aa"))));
        check(Some(Seq(vec!(text("a"), text("b"), text("c")))),
              Seq(vec!(Seq(vec!(text("a"), text("b"))), text("c"))));
        check(Some(Repeat(~text("a"))), Repeat(~Seq(vec!(Repeat(~text("a"))))));
    }
}
