use std::collections::HashMap;
use std::io::{self, BufRead};

use untwine::{parse, parser, parser_repl};

#[derive(Debug)]
enum Expr {
    True,
    False,
    Term(String),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Imply(Box<Expr>, Box<Expr>),
    Equiv(Box<Expr>, Box<Expr>),
}

parser! {
    w = #["\n\r\t "]*;
    term: term=<('a'-'z' | 'A'-'Z')+> -> Expr {
        match term {
            "true" => Expr::True,
            "false" => Expr::False,
            t => Expr::Term(t.to_string())
        }
    }

    not: "!" w e=expr -> Expr { Expr::Not(e.into()) }

         /*
    operator: val=<("&" | "|" | "=>" | "<=>")> -> String { val.to_string() }
    binary: left=expr w rest=(w operator w expr)* -> Expr {
        rest.into_iter().fold(left, |l, (op, r)| {
            let typ = match op.as_str() {
                "&" => Expr::And,
                "|" => Expr::Or,
                "=>" => Expr::Imply,
                "<=>" => Expr::Equiv,
                x => unreachable!("found operator {}", x),
            };
            typ(l.into(), r.into())
        })
    }
         */

    unary = w ((term | not) | ("(" expr ")")) w -> Expr;

    or: left=unary w rest=(w ("||" | "|") w unary)* -> Expr {
        box_expr(left, rest, Expr::Or)
    }

    and: left=or w rest=(w ("&&" | "&") w or)* -> Expr {
        box_expr(left, rest, Expr::And)
    }

    imply: left=and w rest=(w "=>" w and)* -> Expr {
        box_expr(left, rest, Expr::Imply)
    }

    equiv: left=imply w rest=(w "<=>" w imply)* -> Expr {
        box_expr(left, rest, Expr::Equiv)
    }




    //pub expr = w (binary | term | not) w -> Expr;
    pub expr = w equiv w -> Expr;
}

fn box_expr(left: Expr, rest: Vec<Expr>, f: fn(Box<Expr>, Box<Expr>)->Expr) -> Expr {
    rest.into_iter().fold(left, |l, r| {
        f(l.into(), r.into())
    })
}


fn interpret(expr: Expr, vars: HashMap<String, bool>) -> bool {
    match expr {
        Expr::True => true,
        Expr::False => false,
        Expr::Term(t) => {
            let Some(val) = vars.get(&t) else {
                panic!("Value for \"{t}\" not found!");
            };
            *val
        },
        Expr::Not(e) => !interpret(*e, vars),
        Expr::Or(l, r) => interpret(*l, vars.clone()) || interpret(*r, vars),
        Expr::And(l, r) => interpret(*l, vars.clone()) && interpret(*r, vars),
        Expr::Imply(l, r) => !interpret(*l, vars.clone()) || interpret(*r, vars),
        Expr::Equiv(l, r) => !(interpret(*l, vars.clone()) ^ interpret(*r, vars)),
    }
}


fn main() {
    println!("Hello, world!");
    //parser_repl(expr);
    loop {
        let stdin = io::stdin();
        let line1 = stdin.lock().lines().next().unwrap().unwrap();
        let ast = parse(expr, &line1).unwrap();
        println!("{ast:?}");
        println!("{}", interpret(ast, HashMap::from([
                    ("a".to_string(), true),
                    ("b".to_string(), false),
        ])));
    }
}
