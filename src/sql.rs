use crate::types::{Comp, Expr, Ref};

#[derive(Debug, PartialEq, derive_more::Display, derive_more::Error)]
pub enum Error {
    ColumnIndexOutOfBounds,
    MultipleRowsReferenced,
}

#[derive(Debug)]
pub struct Context<'a> {
    columns: &'a Vec<String>,
    row: Option<usize>,
}

fn comp_sql(c: &Comp) -> &'static str {
    match c {
        Comp::Equal => "=",
        Comp::NotEqual => "<>",
        Comp::Greater => ">",
        Comp::Lower => "<",
        Comp::GreaterEqual => ">=",
        Comp::LowerEqual => "<=",
    }
}

pub fn transform(expr: &Expr) -> Result<String, Error> {
    transform_with_columns(expr, &vec![])
}

pub fn transform_with_columns(expr: &Expr, columns: &Vec<String>) -> Result<String, Error> {
    let mut ctx = Context { columns, row: None };
    transform_(expr, &mut ctx)
}

fn transform_(expr: &Expr, ctx: &mut Context) -> Result<String, Error> {
    match expr {
        Expr::Num(n) => Ok(format!("{:}", n)),
        Expr::Bool(b) => {
            if *b {
                Ok("1".into())
            } else {
                Ok("0".into())
            }
        }
        Expr::String(s) => Ok(format!("'{}'", s.replace('\'', "''"))),
        Expr::Perc(a) => Ok(format!("({}/100.0)", transform_(a, ctx)?)),
        Expr::Neg(a) => Ok(format!("-{}", transform_(a, ctx)?)),
        Expr::Add(a, b) => Ok(format!("{} + {}", transform_(a, ctx)?, transform_(b, ctx)?)),
        Expr::Sub(a, b) => Ok(format!("{} - {}", transform_(a, ctx)?, transform_(b, ctx)?)),
        Expr::Mul(a, b) => Ok(format!("{} * {}", transform_(a, ctx)?, transform_(b, ctx)?)),
        Expr::Div(a, b) => Ok(format!("{} / {}", transform_(a, ctx)?, transform_(b, ctx)?)),
        Expr::Pow(a, b) => Ok(format!(
            "POW({}, {})",
            transform_(a, ctx)?,
            transform_(b, ctx)?
        )),
        Expr::Concat(a, b) => Ok(format!(
            "CONCAT({}, {})",
            transform_(a, ctx)?,
            transform_(b, ctx)?
        )),
        Expr::Cond(c, a, b) => Ok(format!(
            "({} {} {})",
            transform_(a, ctx)?,
            comp_sql(c),
            transform_(b, ctx)?
        )),
        Expr::Ref(Ref::CellRef(col, row)) => {
            // check if index is out of bounds
            if *col > ctx.columns.len() {
                return Err(Error::ColumnIndexOutOfBounds);
            }
            // check if all references are on the same row
            if let Some(n) = ctx.row {
                if n != *row {
                    return Err(Error::MultipleRowsReferenced);
                }
            } else {
                ctx.row = Some(*row);
            }

            Ok(ctx.columns.get(*col - 1).unwrap().clone())
        }
        Expr::Func(f, args) => {
            // TODO: map common formula functions to sql logic

            // direct translation to sql functions as a fallback
            Ok(format!(
                "{}({})",
                f,
                args.iter()
                    .map(|x| transform_(x, ctx))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(",")
            ))
        }
        _ => todo!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        assert_eq!(transform(&Expr::Bool(false)).unwrap(), "0");
        assert_eq!(transform(&Expr::Bool(true)).unwrap(), "1");
        assert_eq!(transform(&Expr::Num(3.0)).unwrap(), "3");
        assert_eq!(
            transform(&Expr::Neg(Box::new(Expr::Num(3.0)))).unwrap(),
            "-3"
        );
    }

    #[test]
    fn cell_ref() {
        let cols = ["foo", "bar", "baz"].map(String::from).to_vec();
        let check_ref = |a, b| {
            assert_eq!(
                transform_with_columns(&Expr::Ref(Ref::CellRef(a, 1)), &cols).unwrap(),
                b
            );
        };
        check_ref(1, "foo");
        check_ref(2, "bar");
        check_ref(3, "baz");
    }

    #[test]
    fn cell_ref_invalid() {
        let cols = ["foo", "bar", "baz"].map(String::from).to_vec();
        let check_ref = |col: usize, row: usize, e: Error| {
            assert_eq!(
                transform_with_columns(&Expr::Ref(Ref::CellRef(col.into(), row)), &cols),
                Err(e)
            );
        };
        check_ref(24, 1, Error::ColumnIndexOutOfBounds);
        check_ref(50, 3, Error::ColumnIndexOutOfBounds);
        check_ref(625, 5, Error::ColumnIndexOutOfBounds);
    }

    #[test]
    fn cell_ref_multiple_rows() {
        let cols = ["foo", "bar", "baz"].map(String::from).to_vec();
        let check_refs = |rows: Vec<usize>, e: Result<String, Error>| {
            let expr = rows.iter().fold(Expr::Num(0.0), |sum, x| {
                Expr::Add(Box::new(sum), Box::new(Expr::Ref(Ref::CellRef(1, *x))))
            });
            assert_eq!(transform_with_columns(&expr, &cols), e);
        };
        check_refs(vec![1], Ok("0 + foo".into()));
        check_refs(vec![1, 1], Ok("0 + foo + foo".into()));
        check_refs(vec![3, 3], Ok("0 + foo + foo".into()));
        check_refs(vec![4, 4, 4, 4], Ok("0 + foo + foo + foo + foo".into()));
        check_refs(vec![1, 2], Err(Error::MultipleRowsReferenced));
        check_refs(vec![3, 1], Err(Error::MultipleRowsReferenced));
        check_refs(vec![1, 1, 1, 3], Err(Error::MultipleRowsReferenced));
    }
}
