use ahash::AHashMap;
use log::trace;

use crate::{
    conversion::ConvertToNumber,
    types::{Error, Expr, Ref, Value},
};

#[derive(Debug, Clone)]
pub struct Cell {
    pub value: Option<Value>,
    pub expr: Option<Expr>,
}

type EvalFn = dyn Fn(&[Expr], &Context) -> Result<Value, Error>;

// TODO: this should be expanded into multiple sheets one day (aka Workbook)
pub struct Context {
    pub sheet: Sheet,
    pub current_loc: Option<(usize, usize)>,
    pub functions: AHashMap<String, Box<EvalFn>>,
}

impl Default for Context {
    fn default() -> Self {
        let mut ctx = Self {
            sheet: Sheet::default(),
            current_loc: None,
            functions: AHashMap::default(),
        };
        ctx.add_small_functions();
        ctx
    }
}

impl Context {
    fn add_small_functions(&mut self) {
        use crate::functions::*;
        self.functions.insert("ABS".into(), Box::new(abs));
        self.functions.insert("ACOS".into(), Box::new(acos));
        self.functions.insert("ASIN".into(), Box::new(asin));
        self.functions.insert("ATAN".into(), Box::new(atan));
        self.functions.insert("COS".into(), Box::new(cos));
        self.functions.insert("DEGREES".into(), Box::new(degrees));
        self.functions.insert("EXP".into(), Box::new(exp));
        self.functions.insert("LN".into(), Box::new(ln));
        self.functions.insert("LOG10".into(), Box::new(log10));
        self.functions.insert("RADIANS".into(), Box::new(radians));
        self.functions.insert("SIN".into(), Box::new(sin));
        self.functions.insert("SQRT".into(), Box::new(sqrt));
        self.functions.insert("TAN".into(), Box::new(tan));
    }
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

    pub fn iter(&self) -> SheetCellsIter {
        SheetCellsIter {
            iter: self.map.iter(),
        }
    }

    pub fn set(&mut self, x: usize, y: usize, cell: Cell) -> Option<Cell> {
        self.map.insert((x, y), cell)
    }
}

fn eval_to_num<F>(ctx: &Context, expr: &Expr, f: F) -> Value
where
    F: Fn(f64) -> Value,
{
    let v = eval(ctx, expr).convert_to_number(ctx);
    match v {
        Value::Num(n) => f(n),
        _ => Value::Err(Error::Value),
    }
}

fn eval_to_num_2<F>(ctx: &Context, lhs: &Expr, rhs: &Expr, f: F) -> Value
where
    F: Fn(f64, f64) -> Value,
{
    let vl = eval(ctx, lhs).convert_to_number(ctx);
    let vr = eval(ctx, rhs).convert_to_number(ctx);
    if let (Value::Num(vl), Value::Num(vr)) = (vl, vr) {
        f(vl, vr)
    } else {
        Value::Err(Error::Value)
    }
}

/// Evaluate a reference to a single cell value.
///
/// Apply implied intersection if multiple cells are referenced.
pub fn eval_ref(ctx: &Context, r: &Ref) -> Value {
    match r {
        // evaluate single cell reference
        Ref::CellRef(x, y) => {
            let cell = ctx.sheet.get(*x, *y);
            if let Some(cell) = cell {
                if let Some(val) = cell.value.clone() {
                    val
                } else {
                    Value::Err(Error::Ref)
                }
            } else {
                Value::EmptyCell
            }
        }
        // implied intersection
        Ref::ColumnRange(x1, x2) => {
            if let Some((x, y)) = ctx.current_loc {
                if *x1 != *x2 || x == *x1 {
                    Value::Err(Error::Value)
                } else {
                    let r = Ref::CellRef(*x1, y);
                    eval_ref(ctx, &r)
                }
            } else {
                Value::Err(Error::Ref)
            }
        }
        Ref::RowRange(y1, y2) => {
            if let Some((x, y)) = ctx.current_loc {
                if *y1 != *y2 || y == *y1 {
                    Value::Err(Error::Value)
                } else {
                    let r = Ref::CellRef(x, *y1);
                    eval_ref(ctx, &r)
                }
            } else {
                Value::Err(Error::Ref)
            }
        }
        Ref::CellRange((x1, y1), (x2, y2)) => {
            assert!(*x1 <= *x2);
            assert!(*y1 <= *y2);
            if let Some((x, y)) = ctx.current_loc {
                if x >= *x1 && x <= *x2 {
                    // columns overlap
                    if *y1 != *y2 || y == *y1 {
                        Value::Err(Error::Value)
                    } else {
                        let r = Ref::CellRef(x, *y1);
                        eval_ref(ctx, &r)
                    }
                } else if y >= *y1 && y <= *y2 {
                    // rows overlap
                    if *x1 != *x2 || x == *x1 {
                        Value::Err(Error::Value)
                    } else {
                        let r = Ref::CellRef(*x1, y);
                        eval_ref(ctx, &r)
                    }
                } else {
                    // no overlap, intersection is empty
                    Value::Err(Error::Value)
                }
            } else {
                Value::Err(Error::Ref)
            }
        }
    }
}

pub fn eval_fn(ctx: &Context, fname: &str, args: &[Expr]) -> Value {
    if let Some(f) = ctx.functions.get(fname) {
        // TODO: this will always evaluate each argument
        // we may need to pass Vec<Expr> to the functions, to let them decide
        // e.g. IF does not need to evaluate every argument
        match f(&args, ctx) {
            Ok(v) => v,
            Err(e) => Value::Err(e),
        }
    } else {
        Value::Err(Error::Name)
    }
}

pub fn eval(ctx: &Context, expr: &Expr) -> Value {
    trace!("{:?}", expr);
    let v = match expr {
        Expr::Num(n) => Value::Num(*n),
        Expr::Bool(b) => Value::Bool(*b),
        Expr::String(s) => Value::String(s.clone()),
        Expr::Perc(e) => eval_to_num(ctx, e, |n| Value::Num(n / 100.0)),
        Expr::Neg(e) => eval_to_num(ctx, e, |n| Value::Num(-n)),
        Expr::Add(l, r) => eval_to_num_2(ctx, l, r, |l, r| Value::Num(l + r)),
        Expr::Sub(l, r) => eval_to_num_2(ctx, l, r, |l, r| Value::Num(l - r)),
        Expr::Mul(l, r) => eval_to_num_2(ctx, l, r, |l, r| Value::Num(l * r)),
        Expr::Div(l, r) => eval_to_num_2(ctx, l, r, |l, r| {
            if r == 0.0 {
                Value::Err(Error::Div0)
            } else {
                Value::Num(l / r)
            }
        }),
        Expr::Pow(l, r) => eval_to_num_2(ctx, l, r, |l, r| Value::Num(l.powf(r))),
        Expr::Ref(r) => Value::Ref(r.clone()),
        Expr::Func(fname, args) => eval_fn(ctx, fname, args),
        _ => Value::Err(Error::Unimplemented),
    };
    trace!("{:?} → {:?}", expr, v);
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        conversion::ConvertToScalar,
        parser::{parser, Parser},
    };
    use dir_test::{dir_test, Fixture};
    use log::trace;
    use test_log::test;

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
        let (res, errs) = parser().parse_recovery_verbose(formula);
        trace!("{:?}", errs);
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
                assert_eq!(val, eval_val);
            }
        }
    }
}
