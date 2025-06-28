use std::collections::{HashMap, HashSet};
use std::env;
use std::io::{self, BufRead};

use untwine::{parse, parser, parser_repl};

#[derive(Debug, Clone)]
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

    operator: val=<("&" | "|" | "=>" | "<=>")> -> String { val.to_string() }
    binary: left=unary w rest=(w operator w binary)* -> Expr {
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

    unary = w (term | not | ("(" expr ")")) w -> Expr;

         /*
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
         */




    pub expr = w (binary | unary) w -> Expr;
    //pub expr = w equiv w -> Expr;
}

fn box_expr(left: Expr, rest: Vec<Expr>, f: fn(Box<Expr>, Box<Expr>) -> Expr) -> Expr {
    rest.into_iter().fold(left, |l, r| f(l.into(), r.into()))
}

fn interpret(expr: Expr, vars: HashMap<String, bool>) -> bool {
    let symbols = get_vars(expr.clone());
    for s in symbols {
        if !vars.contains_key(&s) {
            panic!("symbol \"{s}\" not found in vars!");
        }
    }
    interpret_(expr, vars)
}
fn interpret_(expr: Expr, vars: HashMap<String, bool>) -> bool {
    // REMINDER: || and && SHORT-CIRCUIT
    // which means that if there is an unknown symbol, it gets ignored
    match expr {
        Expr::True => true,
        Expr::False => false,
        Expr::Term(t) => {
            let Some(val) = vars.get(&t) else {
                panic!("Value for \"{t}\" not found!");
            };
            *val
        }
        Expr::Not(e) => !interpret(*e, vars),
        Expr::Or(l, r) => interpret(*l, vars.clone()) | interpret(*r, vars),
        Expr::And(l, r) => interpret(*l, vars.clone()) & interpret(*r, vars),
        Expr::Imply(l, r) => !interpret(*l, vars.clone()) | interpret(*r, vars),
        Expr::Equiv(l, r) => !(interpret(*l, vars.clone()) ^ interpret(*r, vars)),
    }
}

// TODO: use hashset
fn get_vars(expr: Expr) -> HashSet<String> {
    let mut set = HashSet::new();
    set.extend(get_vars_(expr));
    set
}
fn get_vars_(expr: Expr) -> Vec<String> {
    match expr {
        Expr::True => vec![],
        Expr::False => vec![],
        Expr::Term(t) => vec![t],
        Expr::Not(e) => get_vars_(*e),
        Expr::Or(l, r) => {
            let mut l1 = get_vars_(*l);
            let mut r1 = get_vars_(*r);
            l1.append(&mut r1);
            l1
        }
        Expr::And(l, r) => {
            let mut l1 = get_vars_(*l);
            let mut r1 = get_vars_(*r);
            l1.append(&mut r1);
            l1
        }
        Expr::Imply(l, r) => {
            let mut l1 = get_vars_(*l);
            let mut r1 = get_vars_(*r);
            l1.append(&mut r1);
            l1
        }
        Expr::Equiv(l, r) => {
            let mut l1 = get_vars_(*l);
            let mut r1 = get_vars_(*r);
            l1.append(&mut r1);
            l1
        }
    }
}

fn make_table(expr: Expr) -> Vec<HashMap<String, bool>> {
    let vars = get_vars(expr);
    let mut ls: Vec<HashMap<String, bool>> = vec![HashMap::from([("<".to_string(), false)])];
    for v in vars {
        let mut ls2 = vec![];
        for table in ls {
            let mut new_table = table.clone();
            let mut new_table2 = table.clone();
            new_table.insert(v.clone(), false);
            new_table2.insert(v.clone(), true);
            ls2.push(new_table);
            ls2.push(new_table2);
        }
        ls = ls2;
    }
    ls
}

fn b2i(b: bool) -> i32 {
    if b {
        1
    } else {
        0
    }
}

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    println!("Hello, world!");

    //parser_repl(expr);
    loop {
        println!("##########");
        let stdin = io::stdin();
        let line1 = stdin.lock().lines().next().unwrap().unwrap();
        let ast = parse(expr, &line1).unwrap();
        //println!("{:?}", ast.clone());

        let vars = get_vars(ast.clone());
        println!(
            "| {} | result |",
            vars.iter()
                .map(|k| k.clone())
                .collect::<Vec<String>>()
                .join(" | ")
        );

        let table = make_table(ast.clone());
        for t in table {
            println!(
                "| {} |      {} |",
                t.keys()
                    .filter(|k| **k != "<".to_string())
                    .map(|k| format!("{:>w$}", b2i(*t.get(k).unwrap()), w = k.len()))
                    .collect::<Vec<String>>()
                    .join(" | "),
                b2i(interpret(ast.clone(), t))
            );
        }
    }
}
