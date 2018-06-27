#[macro_use]
extern crate nom;

use nom::*;

#[derive(Debug, Clone, PartialEq)]
enum Expr {
    BinOp(BinOp),
    Number(Number),
}

#[derive(Debug, Clone, PartialEq)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, PartialEq)]
struct BinOp {
    l: Box<Expr>,
    op: Op,
    r: Box<Expr>,
}

impl BinOp {
    fn new(l: BinOp, op: Op, r: BinOp) -> BinOp {
        BinOp {
            l: Box::new(Expr::BinOp(l)),
            op: op,
            r: Box::new(Expr::BinOp(r)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Number(i64);

named!(
    expr<&str, Expr>,
    alt!(expr1)
);

named!(expr1 <&str, Expr>, alt_complete!(
    map!(binop1, Expr::BinOp) |
    expr2
));

fn binop1(input: &str) -> IResult<&str, BinOp> {
    named!(parse_op <&str, Op>,
           alt!(map!(tag!("+"), |_|Op::Add) | map!(tag!("-"), |_|Op::Sub)));

    let (input, l) = expr2(input)?;
    let (input, _) = multispace0(input)?;
    let (input, op) = parse_op(input)?;
    let (input, _) = multispace0(input)?;
    let (input, r) = expr2(input)?;
    let mut ret = BinOp {
        l: Box::new(l),
        op: op,
        r: Box::new(r),
    };
    let mut input_mut = input;
    loop {
        let (input, _) = match multispace0(input_mut) {
            Ok(ok) => ok,
            Err(_) => return Ok((input_mut, ret)),
        };
        let (input, op) = match parse_op(input) {
            Ok(ok) => ok,
            Err(_) => return Ok((input_mut, ret)),
        };
        let (input, _) = multispace0(input)?;
        let (input, r) = expr2(input)?;
        ret = BinOp {
            l: Box::new(Expr::BinOp(ret)),
            op: op,
            r: Box::new(r),
        };
        input_mut = input;
    }
}

named!(expr2 <&str, Expr>, alt_complete!(
    map!(binop2, Expr::BinOp) |
    expr3
));

fn binop2(input: &str) -> IResult<&str, BinOp> {
    named!(parse_op <&str, Op>,
           alt!(map!(tag!("*"), |_|Op::Mul) | map!(tag!("/"), |_|Op::Div)));

    let (input, l) = expr3(input)?;
    let (input, _) = multispace0(input)?;
    let (input, op) = parse_op(input)?;
    let (input, _) = multispace0(input)?;
    let (input, r) = expr3(input)?;
    let mut ret = BinOp {
        l: Box::new(l),
        op: op,
        r: Box::new(r),
    };
    let mut input_mut = input;
    loop {
        let (input, _) = match multispace0(input_mut) {
            Ok(ok) => ok,
            Err(_) => return Ok((input_mut, ret)),
        };
        let (input, op) = match parse_op(input) {
            Ok(ok) => ok,
            Err(_) => return Ok((input_mut, ret)),
        };
        let (input, _) = multispace0(input)?;
        let (input, r) = expr3(input)?;
        ret = BinOp {
            l: Box::new(Expr::BinOp(ret)),
            op: op,
            r: Box::new(r),
        };
        input_mut = input;
    }
}

named!(expr3 <&str, Expr>, map!(number, Expr::Number));

named!(number<&str, Number>, map!(atom_number, Number));

named!(
    atom_number<&str, i64>,
    map!(
        // recognizeで文字列全体を受け取る
        recognize!(
            // do_parse!で文字列のみ認識。変換はしない
            do_parse!(
                opt!(tuple!(tag_s!("-"), multispace0)) >>
                    digit >> ()

            )),
            |n: &str| n.parse().unwrap()
    )
);

#[test]
fn test_expr_number_pos1() {
    assert_eq!(expr("1 "), Ok((" ", Expr::Number(Number(1)))))
}

#[test]
fn test_expr_number_neg1() {
    assert_eq!(expr("-1 "), Ok((" ", Expr::Number(Number(-1)))))
}

#[test]
fn test_expr_number_negn() {
    assert_eq!(expr("-10 "), Ok((" ", Expr::Number(Number(-10)))))
}

#[test]
fn test_expr_add() {
    assert_eq!(
        expr("1 + 2 "),
        Ok((
            " ",
            Expr::BinOp(BinOp {
                l: Box::new(Expr::Number(Number(1))),
                op: Op::Add,
                r: Box::new(Expr::Number(Number(2))),
            })
        ))
    )
}

#[test]
fn test_expr_add3() {
    assert_eq!(
        expr("1 + 2 + 3 "),
        Ok((
            " ",
            Expr::BinOp(BinOp {
                l: Box::new(Expr::BinOp(BinOp {
                    l: Box::new(Expr::Number(Number(1))),
                    op: Op::Add,
                    r: Box::new(Expr::Number(Number(2))),
                })),
                op: Op::Add,
                r: Box::new(Expr::Number(Number(3)))
            })
        ))
    )
}

#[test]
fn test_expr_add_mul() {
    assert_eq!(
        expr("1 + 2 * 3 "),
        Ok((
            " ",
            Expr::BinOp(BinOp {
                l: Box::new(Expr::Number(Number(1))),
                op: Op::Add,
                r: Box::new(Expr::BinOp(BinOp {
                    l: Box::new(Expr::Number(Number(2))),
                    op: Op::Mul,
                    r: Box::new(Expr::Number(Number(3))),
                }))
            })
        ))
    )
}

#[test]
fn test_expr_mul_add() {
    assert_eq!(
        expr("1 * 2 + 3 "),
        Ok((
            " ",
            Expr::BinOp(BinOp {
                l: Box::new(Expr::BinOp(BinOp {
                    l: Box::new(Expr::Number(Number(1))),
                    op: Op::Mul,
                    r: Box::new(Expr::Number(Number(2))),
                })),
                op: Op::Add,
                r: Box::new(Expr::Number(Number(3)))
            })
        ))
    )
}

#[test]
fn test_atom_number_pos1() {
    assert_eq!(atom_number("1 "), Ok((" ", 1)))
}

#[test]
fn test_atom_number_neg1() {
    assert_eq!(atom_number("-1 "), Ok((" ", -1)))
}

#[test]
fn test_atom_number_negn() {
    assert_eq!(atom_number("-10 "), Ok((" ", -10)))
}

fn calc(e: Expr) -> i64 {
    match e {
        Expr::Number(Number(n)) => n,
        Expr::BinOp(BinOp { l, op, r }) => {
            use self::Op::*;

            let l = calc(*l);
            let r = calc(*r);
            match op {
                Add => l + r,
                Sub => l - r,
                Mul => l * r,
                Div => l / r,
            }
        }
    }
}

fn main() {
    use std::io;
    use std::io::prelude::*;
    let stdin = io::stdin();
    let mut stdin = io::BufReader::new(stdin.lock());
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    loop {
        let mut input = String::new();
        stdout
            .write_all(b"> ")
            .and_then(|_| stdout.flush())
            .expect("faild writing to stdout");
        stdin
            .read_line(&mut input)
            .expect("failed reading from stdin");
        match expr(&input) {
            Ok((_rest, e)) => println!("{}", calc(e)),
            Err(e) => eprintln!("{}", e),
        }
    }
}
