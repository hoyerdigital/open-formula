use ahash::AHashMap;
use log::trace;

use crate::{
    conversion::ConvertToNumber,
    types::{Error, Expr, Ref, Result, Value},
};

#[derive(Debug, Clone)]
pub struct Cell {
    pub value: Option<Value>,
    pub expr: Option<Expr>,
}

type EvalFn = dyn Fn(&[Expr], &Context) -> Result<Value>;

// TODO: this should be expanded into multiple sheets one day (aka Workbook)
#[derive(Default)]
pub struct Context {
    pub sheet: Sheet,
    pub current_loc: Option<(usize, usize)>,
    pub functions: AHashMap<String, Box<EvalFn>>,
}

#[derive(Debug, Default, Clone)]
pub struct Sheet {
    map: AHashMap<(usize, usize), Cell>,
}

pub struct SheetCellsIter<'a> {
    iter: std::collections::hash_map::Iter<'a, (usize, usize), Cell>,
}

impl<'a> Iterator for SheetCellsIter<'a> {
    type Item = ((usize, usize), &'a Cell);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|((x, y), c)| ((*x, *y), c))
    }
}

impl Sheet {
    pub fn has_cell(&self, x: usize, y: usize) -> bool {
        self.map.contains_key(&(x, y))
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&Cell> {
        self.map.get(&(x, y))
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Cell> {
        self.map.get_mut(&(x, y))
    }

    pub fn iter(&self) -> SheetCellsIter<'_> {
        SheetCellsIter {
            iter: self.map.iter(),
        }
    }

    pub fn set(&mut self, x: usize, y: usize, cell: Cell) -> Option<Cell> {
        self.map.insert((x, y), cell)
    }
}

fn eval_to_num<F>(ctx: &Context, expr: &Expr, f: F) -> Result<Value>
where
    F: Fn(f64) -> Result<f64>,
{
    let v = eval(ctx, expr).convert_to_number(ctx)?;
    match v {
        Value::Num(n) => f(n).map(Value::Num),
        // converet_to_number always returns Value::Num
        _ => unreachable!(),
    }
}

fn eval_to_num_2<F>(ctx: &Context, lhs: &Expr, rhs: &Expr, f: F) -> Result<Value>
where
    F: Fn(f64, f64) -> Result<f64>,
{
    let vl = eval(ctx, lhs).convert_to_number(ctx)?;
    let vr = eval(ctx, rhs).convert_to_number(ctx)?;
    if let (Value::Num(vl), Value::Num(vr)) = (vl, vr) {
        f(vl, vr).map(Value::Num)
    } else {
        // converet_to_number always returns Value::Num
        unreachable!()
    }
}

/// Evaluate a reference to a single cell value.
///
/// Apply implied intersection if multiple cells are referenced.
pub fn eval_ref(ctx: &Context, r: &Ref) -> Result<Value> {
    match r {
        // evaluate single cell reference
        Ref::CellRef(x, y) => {
            let cell = ctx.sheet.get(*x, *y);
            if let Some(cell) = cell {
                if let Some(val) = cell.value.clone() {
                    Ok(val)
                } else {
                    Err(Error::Ref)
                }
            } else {
                Ok(Value::EmptyCell)
            }
        }
        // implied intersection
        Ref::ColumnRange(x1, x2) => {
            if let Some((x, y)) = ctx.current_loc {
                if *x1 != *x2 || x == *x1 {
                    Err(Error::Value)
                } else {
                    let r = Ref::CellRef(*x1, y);
                    eval_ref(ctx, &r)
                }
            } else {
                Err(Error::Ref)
            }
        }
        Ref::RowRange(y1, y2) => {
            if let Some((x, y)) = ctx.current_loc {
                if *y1 != *y2 || y == *y1 {
                    Err(Error::Value)
                } else {
                    let r = Ref::CellRef(x, *y1);
                    eval_ref(ctx, &r)
                }
            } else {
                Err(Error::Ref)
            }
        }
        Ref::CellRange((x1, y1), (x2, y2)) => {
            assert!(*x1 <= *x2);
            assert!(*y1 <= *y2);
            if let Some((x, y)) = ctx.current_loc {
                if x >= *x1 && x <= *x2 {
                    // columns overlap
                    if *y1 != *y2 || y == *y1 {
                        Err(Error::Value)
                    } else {
                        let r = Ref::CellRef(x, *y1);
                        eval_ref(ctx, &r)
                    }
                } else if y >= *y1 && y <= *y2 {
                    // rows overlap
                    if *x1 != *x2 || x == *x1 {
                        Err(Error::Value)
                    } else {
                        let r = Ref::CellRef(*x1, y);
                        eval_ref(ctx, &r)
                    }
                } else {
                    // no overlap, intersection is empty
                    Err(Error::Value)
                }
            } else {
                Err(Error::Ref)
            }
        }
    }
}

pub fn eval_fn(ctx: &Context, fname: &str, args: &[Expr]) -> Result<Value> {
    use crate::functions::*;
    match fname {
        // first check for functions defined at compile time
        "ABS" => abs(args, ctx),
        "ACOS" => acos(args, ctx),
        "ASIN" => asin(args, ctx),
        "ATAN" => atan(args, ctx),
        "COS" => cos(args, ctx),
        "DEGREES" => degrees(args, ctx),
        "EXP" => exp(args, ctx),
        "LN" => ln(args, ctx),
        "LOG10" => log10(args, ctx),
        "RADIANS" => radians(args, ctx),
        "SIN" => sin(args, ctx),
        "SQRT" => sqrt(args, ctx),
        "TAN" => tan(args, ctx),
        // check for functions defined at runtime
        _ => ctx
            .functions
            .get(fname)
            .ok_or(Error::Name)
            .and_then(|f| f(args, ctx)),
    }
}

pub fn eval(ctx: &Context, expr: &Expr) -> Result<Value> {
    trace!("{:?}", expr);
    let v = match expr {
        Expr::Num(n) => Ok(Value::Num(*n)),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::String(s) => Ok(Value::String(s.clone())),
        Expr::Perc(e) => eval_to_num(ctx, e, |n| Ok(n / 100.0)),
        Expr::Neg(e) => eval_to_num(ctx, e, |n| Ok(-n)),
        Expr::Add(l, r) => eval_to_num_2(ctx, l, r, |l, r| Ok(l + r)),
        Expr::Sub(l, r) => eval_to_num_2(ctx, l, r, |l, r| Ok(l - r)),
        Expr::Mul(l, r) => eval_to_num_2(ctx, l, r, |l, r| Ok(l * r)),
        Expr::Div(l, r) => eval_to_num_2(ctx, l, r, |l, r| {
            if r == 0.0 {
                Err(Error::Div0)
            } else {
                Ok(l / r)
            }
        }),
        Expr::Pow(l, r) => eval_to_num_2(ctx, l, r, |l, r| Ok(l.powf(r))),
        Expr::Ref(r) => Ok(Value::Ref(r.clone())),
        Expr::Func(fname, args) => eval_fn(ctx, fname, args),
        _ => Err(Error::Unimplemented),
    };
    trace!("{:?} → {:?}", expr, v);
    v
}

#[cfg(test)]
mod tests {
    use std::{fs::File, path::PathBuf};

    use super::*;
    use crate::{
        conversion::ConvertToScalar,
        parser::{parser, Parser},
    };
    use dir_test::{dir_test, Fixture};
    use log::trace;
    use serde::Deserialize;
    use test_log::test;

    #[derive(Debug, Deserialize)]
    struct FuncNameRow {
        function: String,
        small: bool,
        medium: bool,
        large: bool,
    }

    enum FuncEvalGroup {
        Small,
        Medium,
        Large,
    }

    fn eval_func_names(group: FuncEvalGroup) -> Vec<String> {
        let mut file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        file_path.push("fixtures");
        file_path.push("function_names.csv");
        let file = File::open(file_path).unwrap();
        let mut rdr = csv::Reader::from_reader(file);
        rdr.deserialize()
            .filter_map(|row| {
                let row: FuncNameRow = row.unwrap();
                use FuncEvalGroup::*;
                match group {
                    Small => row.small.then_some(row.function),
                    Medium => row.medium.then_some(row.function),
                    Large => row.large.then_some(row.function),
                }
            })
            .collect()
    }

    fn funcs_evaluator_impl(group: FuncEvalGroup) {
        let fns = eval_func_names(group);
        let ctx = Context::default();
        let mut missing: Vec<String> = vec![];
        for func in fns {
            let formula = format!("{func}()");
            let parser_result = parser().parse(&formula);
            let expr = parser_result.into_output().unwrap();
            let eval_result = eval(&ctx, &expr);
            if let Err(Error::Name) = eval_result {
                missing.push(func);
            } else if Err(Error::Args) != eval_result {
                dbg!(&eval_result);
                assert!(eval_result.is_ok());
            }
        }
        assert_eq!(Vec::<String>::new(), missing);
    }

    #[test]
    #[ignore]
    fn funcs_small_evaluator_impl() {
        funcs_evaluator_impl(FuncEvalGroup::Small);
    }

    #[test]
    #[ignore]
    fn funcs_medium_evaluator_impl() {
        funcs_evaluator_impl(FuncEvalGroup::Medium);
    }

    #[test]
    #[ignore]
    fn funcs_large_evaluator_impl() {
        funcs_evaluator_impl(FuncEvalGroup::Large);
    }

    fn ods_to_value(value: &spreadsheet_ods::Value) -> Option<Value> {
        use spreadsheet_ods::Value as ods;
        use Value::*;
        match value {
            ods::Boolean(b) => Some(Bool(*b)),
            ods::Number(f) => Some(Num(*f)),
            ods::Text(s) => Some(String(s.clone())),
            _ => None,
        }
    }

    fn ods_to_expr(formula: &str) -> Expr {
        // FIXME: proper checking/parsing (remove unwraps)
        let formula = formula.strip_prefix("of:=").unwrap();
        // this regex replaces Open Document Format references with
        // proper spreadsheet references, e.g. [.A1] →  A1, or [.A1:B2] →  A1:B2
        // FIXME: is this the best way to handle this? Maybe add an open document option to the parser?
        let re = regex::Regex::new(r"\[.([A-Z0-9]+)((:)\.([A-Z0-9]+))?\]").unwrap();
        let formula = re.replace_all(formula, "$1$3$4").into_owned();

        // FIXME: proper checking/parsing (remove unwraps)
        let res = parser().parse(&formula);
        trace!("{:?}", res);
        res.unwrap()
    }

    fn load_ods(path: &'static str) -> Sheet {
        let mut sheet = Sheet::default();
        let wb = spreadsheet_ods::read_ods(path).unwrap();
        let ods_sheet = wb.sheet(0);
        ods_sheet.iter().fold((), |_, ((y, x), cell)| {
            let cell = Cell {
                value: ods_to_value(cell.value()),
                expr: cell.formula().map(|f| ods_to_expr(f)),
            };
            trace!("{}, {}: {:?}", x, y, cell);
            sheet.set(x as usize, y as usize, cell);
        });
        sheet
    }

    #[dir_test(
        dir: "$CARGO_MANIFEST_DIR/fixtures",
        glob: "**/*.ods",
        loader: load_ods,
    )]
    fn test_ods(fixture: Fixture<Sheet>) {
        // FIXME: clone may not be necessary here, maybe construct context later
        let mut ctx = Context {
            sheet: fixture.content().clone(),
            ..Default::default()
        };
        for ((x, y), cell) in ctx.sheet.iter() {
            ctx.current_loc = Some((x, y));
            trace!("{},{}: {:?}", x, y, cell);
            if cell.value.is_some() && cell.expr.is_some() {
                let val = cell.value.clone().unwrap();
                let expr = cell.expr.clone().unwrap();
                // FIXME: is Scalar the right type for single cell evaluation? may depend on cell format
                let eval_val = eval(&ctx, &expr).convert_to_scalar(&ctx);
                trace!("{:?}", eval_val);
                assert_eq!(Ok(val), eval_val);
            }
        }
    }
}
