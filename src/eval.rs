use ahash::AHashMap;
use log::trace;

use crate::{
    conversion::ConvertToNumber,
    types::{Error, Expr, Ref, Value},
};
pub use chumsky::Parser;

#[derive(Debug, Clone)]
pub struct Cell {
    pub value: Option<Value>,
    pub expr: Option<Expr>,
}

// TODO: this should be expanded into multiple sheets one day (aka Workbook)
#[derive(Debug)]
pub struct Context {
    sheet: Sheet,
}

#[derive(Debug)]
pub struct Sheet {
    map: AHashMap<(usize, usize), Cell>,
}

impl Default for Sheet {
    fn default() -> Self {
        Self {
            map: AHashMap::new(),
        }
    }
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
    let v = eval(ctx, expr).convert_to_number();
    match v {
        Value::Num(n) => f(n),
        _ => Value::Err(Error::Value),
    }
}

fn eval_to_num_2<F>(ctx: &Context, lhs: &Expr, rhs: &Expr, f: F) -> Value
where
    F: Fn(f64, f64) -> Value,
{
    let vl = eval(ctx, lhs).convert_to_number();
    let vr = eval(ctx, rhs).convert_to_number();
    if let (Value::Num(vl), Value::Num(vr)) = (vl, vr) {
        f(vl, vr)
    } else {
        Value::Err(Error::Value)
    }
}

fn eval_ref(ctx: &Context, r: &Ref) -> Value {
    match r {
        Ref::CellRef(x, y) => {
            // single cell reference is called a "criterion"
            // see https://docs.oasis-open.org/office/OpenDocument/v1.4/OpenDocument-v1.4-part4-formula.html#Criterion
            let cell = ctx.sheet.get(x - 1, y - 1);
            if let Some(cell) = cell {
                if let Some(val) = cell.value.clone() {
                    val
                } else {
                    Value::Err(Error::Ref)
                }
            } else {
                Value::Num(0f64)
            }
        }
        _ => Value::Err(Error::Ref),
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

        Expr::Ref(r) => eval_ref(ctx, r),
        _ => Value::Err(Error::Unimplemented),
    };
    trace!("{:?} →  {:?}", expr, v);
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parser;
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

    fn load_ods(path: &'static str) -> Context {
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
        Context { sheet }
    }

    #[dir_test(
        dir: "$CARGO_MANIFEST_DIR/fixtures",
        glob: "**/*.ods",
        loader: load_ods,
    )]
    fn test_ods(fixture: Fixture<Context>) {
        let ctx = fixture.content();
        for ((x, y), cell) in ctx.sheet.iter() {
            trace!("{},{}: {:?}", x, y, cell);
            if cell.value.is_some() && cell.expr.is_some() {
                let val = cell.value.clone().unwrap();
                let expr = cell.expr.clone().unwrap();
                let eval_val = eval(ctx, &expr);
                assert_eq!(val, eval_val);
            }
        }
    }
}
