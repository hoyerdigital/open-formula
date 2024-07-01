use chumsky::prelude::*;
use enum_as_inner::EnumAsInner;

#[derive(Debug, Clone, PartialEq)]
pub enum Comp {
    Equal,
    NotEqual,
    Lower,
    Greater,
    LowerEqual,
    GreaterEqual,
}

#[derive(Debug, Clone, EnumAsInner, PartialEq)]
pub enum Expr {
    Num(f64),
    Bool(bool),
    String(String),
    Perc(Box<Self>),
    Neg(Box<Self>),
    Add(Box<Self>, Box<Self>),
    Sub(Box<Self>, Box<Self>),
    Mul(Box<Self>, Box<Self>),
    Div(Box<Self>, Box<Self>),
    Pow(Box<Self>, Box<Self>),
    Concat(Box<Self>, Box<Self>),
    Cond(Comp, Box<Self>, Box<Self>),
    Func(String, Vec<Self>),
    CellRef(String, usize),
    CellRange((String, usize), (String, usize)),
}

pub fn parser() -> impl Parser<char, Expr, Error = Simple<char>> {
    let expr = recursive(|expr| {
        let uppercase = filter::<_, _, Simple<char>>(char::is_ascii_uppercase)
            .repeated()
            .at_least(1)
            .collect::<String>();
        let cellref = uppercase
            .then(text::digits(10))
            .map(|(cell, num)| Expr::CellRef(cell, num.parse::<usize>().unwrap()));
        let cellrange = cellref
            .clone()
            .then_ignore(just(":"))
            .then(cellref.clone())
            .map(|(a, b)| {
                // FIXME: .as_cell_ref returns (&a0, &a1), which is not (a0, a1), can this be converted better?
                let (a0, a1) = a.as_cell_ref().unwrap();
                let (b0, b1) = b.as_cell_ref().unwrap();
                Expr::CellRange((a0.clone(), *a1), (b0.clone(), *b1))
            });
        let ident = text::ident().padded();
        let num = text::int(10)
            // TODO: make decimal point character configurable
            .then(just('.').ignore_then(text::digits(10)).or_not())
            .map(|(a, o)| if let Some(b) = o { a + "." + &b } else { a })
            .from_str()
            .unwrapped()
            .map(Expr::Num);
        let str_ = just('"')
            .ignore_then(none_of('"').repeated())
            .then_ignore(just('"'))
            .collect::<String>()
            .map(Expr::String);
        let call = ident
            .then(
                expr.clone()
                    // TODO: make function argument configurable (, vs ;)
                    .separated_by(just(','))
                    .allow_trailing()
                    .collect::<Vec<_>>()
                    .delimited_by(just('('), just(')')),
            )
            .map(|(f, args)| Expr::Func(f, args));
        let bool = choice::<_, Simple<char>>((
            text::keyword("TRUE").to(Expr::Bool(true)),
            text::keyword("FALSE").to(Expr::Bool(false)),
        ))
        .padded();

        let atom = num
            .or(expr.delimited_by(just('('), just(')')))
            .or(num)
            .or(bool)
            .or(str_)
            .or(cellrange)
            .or(cellref)
            .or(call)
            .padded();

        let op = |c| just(c).padded();

        let unary = op('-')
            .repeated()
            .then(atom.clone())
            .foldr(|_op, rhs| Expr::Neg(Box::new(rhs)));

        let unary_suffix = unary
            .clone()
            .then(op('%').repeated())
            .foldl(|lhs, _op| Expr::Perc(Box::new(lhs)));

        let pow = unary_suffix
            .clone()
            .then(
                op('^')
                    .to(Expr::Pow as fn(_, _) -> _)
                    .then(unary)
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

        let product = pow
            .clone()
            .then(
                op('*')
                    .to(Expr::Mul as fn(_, _) -> _)
                    .or(op('/').to(Expr::Div as fn(_, _) -> _))
                    .then(pow)
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

        let sum = product
            .clone()
            .then(
                op('+')
                    .to(Expr::Add as fn(_, _) -> _)
                    .or(op('-').to(Expr::Sub as fn(_, _) -> _))
                    .then(product)
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

        let conc = sum
            .clone()
            .then(
                op('&')
                    .to(Expr::Concat as fn(_, _) -> _)
                    .then(sum)
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

        let comp = choice::<_, Simple<char>>((
            just("<>").to(Comp::NotEqual),
            just(">=").to(Comp::GreaterEqual),
            just("<=").to(Comp::LowerEqual),
            just("=").to(Comp::Equal),
            just(">").to(Comp::Greater),
            just("<").to(Comp::Lower),
        ));

        let cond = conc
            .clone()
            .then(
                comp.map(|c| move |lhs, rhs| Expr::Cond(c, lhs, rhs))
                    .then(conc)
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

        cond
    });

    expr.then_ignore(end())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Expr {
        let (res, errs) = parser().parse_recovery_verbose(input);
        dbg!(res.clone());
        dbg!(errs);
        res.unwrap()
    }

    #[test]
    fn simple_num() {
        assert_eq!(parse("3"), Expr::Num(3.0));
        assert_eq!(parse("3.0"), Expr::Num(3.0));
        assert_eq!(
            parse("3.000000000000000001"),
            Expr::Num(3.000000000000000001)
        );
    }

    #[test]
    fn simple_bool() {
        assert_eq!(parse("TRUE"), Expr::Bool(true));
        assert_eq!(parse("FALSE"), Expr::Bool(false));
    }

    #[test]
    fn simple_string() {
        assert_eq!(parse("\"3\""), Expr::String("3".into()));
        assert_eq!(parse("\"ABCDEFG\""), Expr::String("ABCDEFG".into()));
    }

    #[test]
    fn simple_cellref() {
        assert_eq!(parse("A1"), Expr::CellRef("A".into(), 1));
        assert_eq!(parse("XY23"), Expr::CellRef("XY".into(), 23));
    }

    #[test]
    fn simple_cellrange() {
        assert_eq!(
            parse("A1:Z99"),
            Expr::CellRange(("A".into(), 1), ("Z".into(), 99))
        );
        assert_eq!(
            parse("AA23:BB42"),
            Expr::CellRange(("AA".into(), 23), ("BB".into(), 42))
        );
    }

    #[test]
    fn simple_func() {
        assert_eq!(
            parse("SUM(3,4)"),
            Expr::Func("SUM".into(), vec![Expr::Num(3.0), Expr::Num(4.0)])
        );
        assert_eq!(
            parse("SUM(3.0,4.0)"),
            Expr::Func("SUM".into(), vec![Expr::Num(3.0), Expr::Num(4.0)])
        );
        assert_eq!(
            parse("SUM(3.0 ,     4.0)"),
            Expr::Func("SUM".into(), vec![Expr::Num(3.0), Expr::Num(4.0)])
        );
        assert_eq!(
            parse("TRUES(\"FOOBAR\")"),
            Expr::Func("TRUES".into(), vec![Expr::String("FOOBAR".into())])
        );
        assert_eq!(
            parse("(SUM(\"3\",\"4\"))"),
            Expr::Func(
                "SUM".into(),
                vec![Expr::String("3".into()), Expr::String("4".into())]
            )
        );
    }

    #[test]
    fn simple_ops() {
        // TODO: add asserts
        parse("3+4");
        parse("3.0 + 4.0");
        parse("3*  4");
        parse("3*4/5-2");
        parse("5.0-2");
        parse("1.0/1.0");
        parse("-1.0 / -1.0");
        parse("2^0");
        parse("-3 * -1");
        parse("2*20%");
        parse("\"A\"&TRUE");
    }

    #[test]
    fn complex_ops() {
        // TODO: add asserts
        parse("3+4/2");
        parse("3*  4+10");
        parse("2^2+5/2");
        parse("2^3*3");
        parse("--(3)");
        parse("2*-20%%%");
        parse("\"A\" &\"B\"");
    }

    #[test]
    fn simple_comp() {
        // TODO: add asserts
        parse("3=4");
        parse("3>4");
        parse("3<4");
        parse("3<=4");
        parse("3>=4");
        parse("3<>4");
    }

    #[test]
    fn complex() {
        // TODO: add asserts
        parse("SUM(--(FREQUENCY(IF(C5:C11=G5,MATCH(B5:B11,B5:B11,0)),ROW(B5:B11)-ROW(B5)+1)>0))");
        parse("SUM(--(MMULT(TRANSPOSE(ROW(A1:A99)^0),--(A1:A99=I4))>0))");
    }
}
