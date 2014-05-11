#![crate_id = "synopexpand"]
#![crate_type = "bin"]

extern crate cmdutil;
extern crate synop;

#[cfg(not(test))]
use std::io;
#[cfg(not(test))]
use synop::{Expr, Tok, Seq, Opt, Repeat, Select};

#[cfg(not(test))]
fn print_expand(expr: &Expr) {
    for cmd in inner(expr).iter() {
        println!("{}", cmd);
    }

    fn inner(expr: &Expr) -> Vec<~str> {
        match *expr {
            Tok(ref tok) => vec!(tok.pretty()),
            Seq(ref seq) => {
                let mut v = vec!("".to_owned());
                for ss in seq.iter().map(inner) {
                    let mut v2 = vec!();
                    for s in ss.iter() {
                        v2.extend(v.iter().map(|x| format!("{} {}", x, s)));
                    }
                    v = v2;
                }
                v
            }
            Opt(ref opt) => {
                let mut v = inner(*opt);
                v.unshift("".to_owned());
                v
            }
            Repeat(box Opt(ref rep)) => {
                let mut v = vec!();
                v.push_all_move(inner(&Seq(vec![])));
                v.push_all_move(inner(&Seq(vec![(**rep).clone()])));
                v.push_all_move(inner(&Seq(vec![(**rep).clone(), (**rep).clone()])));
                v
            }
            Repeat(ref rep) => {
                let mut v = vec!();
                v.push_all_move(inner(&Seq(vec![(**rep).clone()])));
                v.push_all_move(inner(&Seq(vec![(**rep).clone(), (**rep).clone()])));
                v.push_all_move(inner(&Seq(vec![(**rep).clone(), (**rep).clone(), (**rep).clone()])));
                v
            }
            Select(ref sel) => {
                sel.iter()
                    .map(inner)
                    .flat_map(|e| e.move_iter())
                    .collect()
            }
        }
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
