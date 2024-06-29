use chumsky::{error::Cheap, prelude::*};
use enum_as_inner::EnumAsInner;

#[derive(Debug, Clone, EnumAsInner)]
enum Expr {
    Num(f64),
    String(String),
    Add(Box<Self>, Box<Self>),
    Sub(Box<Self>, Box<Self>),
    Mul(Box<Self>, Box<Self>),
    Div(Box<Self>, Box<Self>),
    Func(String, Vec<Self>),
    CellRef(String, u64),
    CellRange((String, u64), (String, u64)),
}

fn parser() -> impl Parser<char, Expr, Error = Simple<char>> {
    let expr = recursive(|expr| {
        let uppercase = filter::<_, _, Simple<char>>(char::is_ascii_uppercase)
            .repeated()
            .at_least(1)
            .collect::<String>();
        let cellref = uppercase
            .then(text::digits(10))
            //.then_ignore(end())
            .map(|(cell, num)| Expr::CellRef(cell, num.parse::<u64>().unwrap()));
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
                    .separated_by(just(';'))
                    .allow_trailing()
                    .collect::<Vec<_>>()
                    .delimited_by(just('('), just(')')),
            )
            .map(|(f, args)| Expr::Func(f, args));
        let atom = num
            .or(expr.delimited_by(just('('), just(')')))
            .or(num)
            .or(str_)
            .or(cellrange)
            .or(cellref)
            .or(call)
            .padded();
        atom
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

    // FIXME: add asserts to check Expr values

    #[test]
    fn simple_num() {
        parse("3");
        parse("3.0");
        parse("3.000000000000000001");
    }

    #[test]
    fn simple_string() {
        parse("\"3\"");
        parse("\"ABCDEFG\"");
    }

    #[test]
    fn simple_cellref() {
        parse("A1");
        parse("XY23");
    }

    #[test]
    fn simple_cellrange() {
        parse("A1:Z99");
        parse("AA23:BB42");
    }

    #[test]
    fn simple_func() {
        parse("SUM(3;4)");
        parse("SUM(3.0;4.0)");
        parse("SUM(3.0 ;     4.0)");
        parse("(SUM(\"3\";\"4\"))");
    }

    #[test]
    fn ops() {
        parse("3+4");
        parse("3.0 + 4.0");
        parse("3*4");
        parse("3*4/5-2");
        parse("5.0-2");
        parse("1.0/1.0");
    }
}
