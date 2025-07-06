use std::collections::{HashMap, HashSet};
use std::env;
use std::io::{self, BufRead};

use untwine::{parse, parser};

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
    ident = <('a'-'z' | 'A'-'Z')+> -> &str;
    operator: val=<("&" | "|" | "=>" | "<=>")> -> String { val.to_string() }


    term: term=ident -> Expr {
        match term {
            "true" => Expr::True,
            "false" => Expr::False,
            t => Expr::Term(t.to_string())
        }
    }

    not: "!" w e=expr -> Expr { Expr::Not(e.into()) }
    unary = w (term | not | ("(" expr ")")) w -> Expr;
    // solution from untwine doc examples
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
    pub start: start=(w ident w "=" w expr ";" w)* -> Vec<(String, Expr)> {
        start.into_iter().map(|(s, e)| (s.to_string(), e)).collect::<Vec<_>>()
    }
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
    // assumes there is at least one var
    // which is reasonable I would say
    let mut it = vars.iter();

    let first = it.next().unwrap().clone();
    let mut tables: Vec<HashMap<String, bool>> = vec![
        HashMap::from([(first.clone(), false)]),
        HashMap::from([(first, true)]),
    ];

    for v in it {
        let mut ls2 = vec![];
        for table in tables {
            let mut new_table = table.clone();
            new_table.insert(v.clone(), false);
            ls2.push(new_table);

            let mut new_table2 = table.clone();
            new_table2.insert(v.clone(), true);

            ls2.push(new_table2);
        }
        tables = ls2;
    }
    tables
}

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    println!("Hello, world!");

    //parser_repl(expr);
    loop {
        println!("##########");
        let stdin = io::stdin();
        let line1 = stdin.lock().lines().next().unwrap().unwrap();
        let ast = parse(start, &line1).unwrap();
        //println!("{:?}", ast.clone());

        let (names, asts): (Vec<String>, Vec<Expr>) = ast.into_iter().unzip();
        let vars = asts.iter().flat_map(|e| get_vars(e.clone())).collect::<HashSet<String>>();
        let mut vars_sorted = vars.clone().into_iter().collect::<Vec<_>>();
        vars_sorted.sort();

        println!(
            "| {} ||| {} |",
            vars_sorted.join(" | "),
            names.join(" | ")
        );
        for i in make_table(vars) {
            println!("{:?}", i);
        }
        /*
        println!(
            "| {} | result |",
            ast.iter().map(|v| v.iter()
                .map(|k| k.clone())
                .collect::<Vec<String>>()
                .join(" | ")
        )
                .collect::<Vec<String>>()
                .join(" | ")
            );
        println!(
            "|-{}-|--------|",
            ast.iter()
                .map(|k| "-".repeat(k.len()))
                .collect::<Vec<String>>()
                .join("-|-")
        );
        */
        /*
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
        */
    }
}
