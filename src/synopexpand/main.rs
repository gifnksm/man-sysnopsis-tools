#![crate_name = "synopexpand"]
#![crate_type = "bin"]
#![deny(warnings, unused, bad_style, unnecessary_qualification, unnecessary_typecast)]

#[cfg(not(test))]
extern crate cmdutil;
extern crate synop;

#[cfg(not(test))]
use std::io;
use synop::{Token, Expr};
use synop::Expr::{Tok, Seq, Opt, Repeat, Select};

fn expand(expr: &Expr) -> Vec<Vec<Token>> {
    match *expr {
        Tok(ref tok) => vec![vec![tok.clone()]],
        Seq(ref seq) => {
            let mut v = vec![vec![]];
            for ss in seq.iter().map(expand) {
                let mut v2 = vec![];
                for s in ss.iter() {
                    v2.extend(v.iter().map(|x| *x + *s))
                }
                v = v2;
            }
            v
        }
        Opt(ref opt) => {
            let mut v = expand(&**opt);
            v.insert(0, vec![]);
            v
        }
        Repeat(box Opt(ref rep)) => {
            let mut v = vec![];
            v.extend(expand(&Seq(vec![])).into_iter());
            v.extend(expand(&Seq(vec![(**rep).clone()])).into_iter());
            v.extend(expand(&Seq(vec![(**rep).clone(), (**rep).clone()])).into_iter());
            v
        }
        Repeat(ref rep) => {
            let mut v = vec![];
            v.extend(expand(&Seq(vec![(**rep).clone()])).into_iter());
            v.extend(expand(&Seq(vec![(**rep).clone(), (**rep).clone()])).into_iter());
            v.extend(expand(&Seq(vec![(**rep).clone(), (**rep).clone(), (**rep).clone()])).into_iter());
            v
        }
        Select(ref sel) => {
            sel.iter()
                .map(expand)
                .flat_map(|e| e.into_iter())
                .collect()
        }
    }
}

#[cfg(not(test))]
fn print_expand(expr: &Expr) {
    for cmd in expand(expr).iter() {
        println!("{}", cmd.iter().map(|c| c.pretty()).collect::<Vec<String>>().connect(" "));
    }
}

#[cfg(not(test))]
fn main() {
    cmdutil::main(proc() {
        let ast = try!(synop::read_ast(&mut io::stdin()));
        match ast.normalize() {
            Some(e) => print_expand(&e),
            None => {}
        }
        Ok(())
    });
}

#[cfg(test)]
mod tests {
    use synop::{Token, Expr};
    use synop::Token::Text;
    use synop::Expr::{Tok, Seq, Opt, Repeat, Select};

    fn text_tok(v: Vec<Vec<&str>>) -> Vec<Vec<Token>> {
        v.into_iter()
            .map(|x| x.into_iter().map(|s| Text(s.to_string())).collect())
            .collect()
    }
    fn text(s: &str) -> Expr { Tok(Text(s.to_string())) }

    #[test]
    fn expand() {
        assert_eq!(text_tok(vec![vec!["a"]]), super::expand(&text("a")))
        assert_eq!(text_tok(vec![vec!["a", "b"]]),
                   super::expand(&Seq(vec![text("a"), text("b")])));
        assert_eq!(text_tok(vec![vec![], vec!["a"]]),
                   super::expand(&Opt(box text("a"))));
        assert_eq!(text_tok(vec![vec!["a"], vec!["a", "a"], vec!["a", "a", "a"]]),
                   super::expand(&Repeat(box text("a"))));
        assert_eq!(text_tok(vec![vec![], vec!["a"], vec!["a", "a"]]),
                   super::expand(&Repeat(box Opt(box text("a")))));
        assert_eq!(text_tok(vec![vec!["a"], vec!["b"], vec!["c"]]),
                   super::expand(&Select(vec![text("a"), text("b"), text("c")])));

        assert_eq!(text_tok(vec![vec!["a", "c"], vec!["b", "c"]]),
                   super::expand(&Seq(vec![Select(vec![text("a"), text("b")]), text("c")])));
        assert_eq!(text_tok(vec![vec!["a", "c"], vec!["b", "c"], vec!["a", "d"], vec!["b", "d"]]),
                   super::expand(&Seq(vec![Select(vec![text("a"), text("b")]),
                                           Select(vec![text("c"), text("d")])])));
    }
}
