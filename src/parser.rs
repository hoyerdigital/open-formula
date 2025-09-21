use chumsky::pratt::*;
use chumsky::prelude::*;
pub use chumsky::Parser;

use crate::{
    helpers::column_to_id,
    types::{Comp, Expr, Ref},
    xmlchar::XmlChar,
};

pub fn parser<'a>() -> impl Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> {
    recursive(|expr| {
        let uppercase = any()
            .filter(char::is_ascii_uppercase)
            .repeated()
            .at_least(1)
            .to_slice();
        let cellref = uppercase.then(text::digits(10).to_slice()).try_map(
            |(col_chars, row_num): (&str, &str), span: SimpleSpan| {
                let row = row_num
                    .parse::<usize>()
                    .map_err(|e| Rich::custom(span, format!("{}", e)))?
                    .checked_sub(1)
                    .ok_or(Rich::custom(
                        span,
                        "row reference must be greater than zero",
                    ))?;
                let col =
                    column_to_id(col_chars).map_err(|e| Rich::custom(span, format!("{}", e)))?;
                Ok(Expr::Ref(Ref::CellRef(col, row)))
            },
        );
        let columnrange = uppercase.then_ignore(just(":")).then(uppercase).try_map(
            |(a_chars, b_chars), span: SimpleSpan<usize>| {
                let a = column_to_id(a_chars).map_err(|e| Rich::custom(span, format!("{}", e)))?;
                let b = column_to_id(b_chars).map_err(|e| Rich::custom(span, format!("{}", e)))?;
                Ok(Expr::Ref(Ref::ColumnRange(a, b)))
            },
        );
        let rowrange = text::digits(10)
            .to_slice()
            .then_ignore(just(":"))
            .then(text::digits(10).to_slice())
            .try_map(|(a, b): (&str, &str), span: SimpleSpan<usize>| {
                // TODO: refactor integer parsing into a function (DRY)
                let int_a = a
                    .parse::<usize>()
                    .map_err(|e| Rich::custom(span, format!("{}", e)))?
                    .checked_sub(1)
                    .ok_or(Rich::custom(
                        span,
                        "row reference must be greater than zero",
                    ))?;
                let int_b = b
                    .parse::<usize>()
                    .map_err(|e| Rich::custom(span, format!("{}", e)))?
                    .checked_sub(1)
                    .ok_or(Rich::custom(
                        span,
                        "row reference must be greater than zero",
                    ))?;
                Ok(Expr::Ref(Ref::RowRange(int_a, int_b)))
            });
        let cellrange = cellref.then_ignore(just(":")).then(cellref).map(|(a, b)| {
            // FIXME: .as_cell_ref returns (&a0, &a1), which is not (a0, a1), can this be converted better?
            let (a0, a1) = a.as_ref().unwrap().as_cell_ref().unwrap();
            let (b0, b1) = b.as_ref().unwrap().as_cell_ref().unwrap();
            Expr::Ref(Ref::CellRange((*a0, *a1), (*b0, *b1)))
        });
        // custom ident that differs from chumsky::text::ident, because a lot more
        // characters are allowed
        let ident = any()
            .filter(|c: &char| c.is_xml_letter())
            .then(
                any()
                    .filter(|c: &char| {
                        c.is_xml_letter()
                            || c.is_xml_digit()
                            || *c == '_'
                            || *c == '.'
                            || c.is_xml_combining_char()
                    })
                    .repeated(),
            )
            .padded()
            .to_slice();
        let num = text::int(10)
            // TODO: make decimal point character configurable
            .then(just('.').then(text::digits(10)).or_not())
            .to_slice()
            .from_str()
            .unwrapped()
            .map(Expr::Num);
        let str_ = none_of('"')
            .repeated()
            .to_slice()
            .map(|s: &str| Expr::String(s.to_string()))
            .delimited_by(just('"'), just('"'));
        let call = ident
            .then(
                expr.clone()
                    .separated_by(just(';'))
                    .allow_trailing()
                    .collect::<Vec<_>>()
                    .delimited_by(just('('), just(')')),
            )
            .map(|(f, args): (&str, _)| Expr::Func(f.to_string(), args));
        let bool = choice((
            text::keyword("TRUE").to(Expr::Bool(true)),
            text::keyword("FALSE").to(Expr::Bool(false)),
        ))
        .padded();

        // FIXME: check for proper order of choices
        let atom = choice((
            rowrange,
            expr.delimited_by(just('('), just(')')),
            num,
            bool,
            str_,
            columnrange,
            cellrange,
            cellref,
            call,
        ))
        .padded();

        let op = |c| just(c).padded();

        // all pieces are defined, the root of the parser starts here 🐉
        let comp = choice((
            just("<>").to(Comp::NotEqual),
            just(">=").to(Comp::GreaterEqual),
            just("<=").to(Comp::LowerEqual),
            just("=").to(Comp::Equal),
            just(">").to(Comp::Greater),
            just("<").to(Comp::Lower),
        ));

        #[rustfmt::skip]
        let expr = atom.pratt((
            prefix(7, op('-'), |_, rhs, _| Expr::Neg(Box::new(rhs))),
            prefix(7, op('+'), |_, rhs, _| rhs),
            postfix(6, op('%'), |lhs, _, _| Expr::Perc(Box::new(lhs))),
            infix(left(10), op(':'), |l, _, r, _| Expr::Range(Box::new(l), Box::new(r))),
            infix(left(9), op('!'), |l, _, r, _| Expr::RefIntersection(Box::new(l), Box::new(r))),
            #[cfg(feature = "medium")]
            infix(left(8), op('~'), |l, _, r, _| Expr::RefUnion(Box::new(l), Box::new(r))),
            infix(left(5), op('^'), |l, _, r, _| Expr::Pow(Box::new(l), Box::new(r))),
            infix(left(4), op('*'), |l, _, r, _| Expr::Mul(Box::new(l), Box::new(r))),
            infix(left(4), op('/'), |l, _, r, _| Expr::Div(Box::new(l), Box::new(r))),
            infix(left(3), op('+'), |l, _, r, _| Expr::Add(Box::new(l), Box::new(r))),
            infix(left(3), op('-'), |l, _, r, _| Expr::Sub(Box::new(l), Box::new(r))),
            infix(left(2), op('&'), |l, _, r, _| Expr::Concat(Box::new(l), Box::new(r))),
            infix(left(1), comp, |l, c, r, _| Expr::Cond(c, Box::new(l), Box::new(r)))
        ));

        expr
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::error;
    use test_log::test;

    fn parse(input: &str) -> Expr {
        let res = parser().parse(input);
        if res.has_errors() {
            error!("{:?}", res.clone().into_errors());
        }
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
        assert_eq!(parse("A1"), Expr::Ref(Ref::CellRef(0, 0)));
        assert_eq!(parse("XY23"), Expr::Ref(Ref::CellRef(648, 22)));
    }

    #[test]
    fn simple_columnrange() {
        assert_eq!(parse("A:A"), Expr::Ref(Ref::ColumnRange(0, 0)));
        assert_eq!(parse("H:H"), Expr::Ref(Ref::ColumnRange(7, 7)));
        assert_eq!(parse("B:AB"), Expr::Ref(Ref::ColumnRange(1, 27)));
    }

    #[test]
    fn simple_rowrange() {
        assert_eq!(parse("3:3"), Expr::Ref(Ref::RowRange(2, 2)));
        assert_eq!(parse("1:5"), Expr::Ref(Ref::RowRange(0, 4)));
        assert_eq!(parse("21:9"), Expr::Ref(Ref::RowRange(20, 8)));
    }

    #[test]
    fn simple_cellrange() {
        assert_eq!(parse("A1:Z99"), Expr::Ref(Ref::CellRange((0, 0), (25, 98))));
        assert_eq!(
            parse("AA23:BB42"),
            Expr::Ref(Ref::CellRange((26, 22), (53, 41)))
        );
    }

    #[test]
    fn simple_func() {
        assert_eq!(
            parse("SUM(3;4)"),
            Expr::Func("SUM".into(), vec![Expr::Num(3.0), Expr::Num(4.0)])
        );
        assert_eq!(
            parse("SUM(3.0;4.0)"),
            Expr::Func("SUM".into(), vec![Expr::Num(3.0), Expr::Num(4.0)])
        );
        assert_eq!(
            parse("SUM(3.0 ;     4.0)"),
            Expr::Func("SUM".into(), vec![Expr::Num(3.0), Expr::Num(4.0)])
        );
        assert_eq!(
            parse("TRUES(\"FOOBAR\")"),
            Expr::Func("TRUES".into(), vec![Expr::String("FOOBAR".into())])
        );
        assert_eq!(
            parse("(SUM(\"3\";\"4\"))"),
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
        parse("+5");
        parse("+\"A\"");
        assert_eq!(
            parse("B4:B5:C5"),
            Expr::Range(
                Box::new(Expr::Ref(Ref::CellRange((1, 3), (1, 4)))),
                Box::new(Expr::Ref(Ref::CellRef(2, 4)))
            )
        );
        assert_eq!(
            parse("A1:C4!B1:B5"),
            Expr::RefIntersection(
                Box::new(Expr::Ref(Ref::CellRange((0, 0), (2, 3)))),
                Box::new(Expr::Ref(Ref::CellRange((1, 0), (1, 4))))
            )
        );
        assert_eq!(
            parse("A1:B2~B2:C3"),
            Expr::RefUnion(
                Box::new(Expr::Ref(Ref::CellRange((0, 0), (1, 1)))),
                Box::new(Expr::Ref(Ref::CellRange((1, 1), (2, 2))))
            )
        );
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
        parse("++4.0");
        parse("+-+\"123\"");
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
        parse("SUM(--(FREQUENCY(IF(C5:C11=G5;MATCH(B5:B11;B5:B11;0));ROW(B5:B11)-ROW(B5)+1)>0))");
        parse("SUM(--(MMULT(TRANSPOSE(ROW(A1:A99)^0);--(A1:A99=I4))>0))");
    }
}
