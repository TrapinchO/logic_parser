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
    unary = w (term | not | ("(" expr ")")) w -> Expr;
    // solution from untwine doc examples
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

    pub expr = w (binary | unary) w -> Expr;
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

fn make_table(vars: HashSet<String>) -> Vec<HashMap<String, bool>> {
    // needs temporary entry so that the vector is not empty
    let mut tables: Vec<HashMap<String, bool>> = vec![HashMap::from([("<".to_string(), false)])];
    for v in vars {
        let mut ls2 = vec![];
        for table in tables {
            let mut new_table = table.clone();
            let mut new_table2 = table.clone();
            new_table.insert(v.clone(), false);
            new_table2.insert(v.clone(), true);
            ls2.push(new_table);
            ls2.push(new_table2);
        }
        tables = ls2;
    }
    let mut tables2 = vec![];
    for t in tables {
        let mut t2 = t.clone();
        t2.remove("<");
        tables2.push(t2);
    }
    tables2
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
        println!(
            "|-{}-|--------|",
            vars.iter()
                .map(|k| "-".repeat(k.len()))
                .collect::<Vec<String>>()
                .join("-|-")
        );

        let table = make_table(vars);
        for t in table {
            println!(
                "| {} |      {} |",
                t.keys()
                    //.map(|k| format!("{:>w$}", b2i(*t.get(k).unwrap()), w = k.len()))
                    .map(|k| format!("{:>w$}", *t.get(k).unwrap() as i32, w = k.len()))
                    .collect::<Vec<_>>()
                    .join(" | "),
                interpret(ast.clone(), t) as i32
            );
        }
    }
}
