use ahash::AHashMap;
use log::trace;

use crate::types::{Error, Expr, Ref, Value};
pub use chumsky::Parser;

#[derive(Debug, Clone)]
pub struct Cell {
    pub value: Option<Value>,
    pub expr: Option<Expr>,
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

// TODO: handle implicit string (or other type) to number conversion
// see https://docs.oasis-open.org/office/OpenDocument/v1.3/os/part4-formula/OpenDocument-v1.3-os-part4-formula.html#ConversionToNumber
fn eval_to_num<F>(sheet: &Sheet, expr: &Expr, f: F) -> Value
where
    F: Fn(f64) -> Value,
{
    let v = eval(sheet, expr);
    match v {
        Value::Num(n) => f(n),
        _ => Value::Err(Error::Value),
    }
}

// TODO: generalize eval_to_num to n parameters? maybe with generics and/or tuples
// TODO: handle implicit string (or other type) to number conversion
// see https://docs.oasis-open.org/office/OpenDocument/v1.3/os/part4-formula/OpenDocument-v1.3-os-part4-formula.html#ConversionToNumber
fn eval_to_num_2<F>(sheet: &Sheet, lhs: &Expr, rhs: &Expr, f: F) -> Value
where
    F: Fn(f64, f64) -> Value,
{
    let vl = eval(sheet, lhs);
    let vr = eval(sheet, rhs);
    if let (Value::Num(vl), Value::Num(vr)) = (vl, vr) {
        f(vl, vr)
    } else {
        Value::Err(Error::Value)
    }
}

fn column_letter_to_number<F>(s: &str, f: F) -> Value
where
    F: Fn(usize) -> Value,
{
    let mut num: usize = 0;
    for (cnt, c) in s.chars().enumerate() {
        if !c.is_ascii_uppercase() {
            return Value::Err(Error::Ref);
        }
        num += ((c as usize) - 64) + (cnt * 26);
    }
    f(num)
}

fn eval_ref(sheet: &Sheet, r: &Ref) -> Value {
    match r {
        Ref::CellRef(col, y) => column_letter_to_number(col, |x| {
            let cell = sheet.get(x - 1, *y - 1);
            if let Some(cell) = cell {
                if let Some(val) = cell.value.clone() {
                    val
                } else {
                    Value::Err(Error::Ref)
                }
            } else {
                Value::Err(Error::Ref)
            }
        }),
        _ => Value::Err(Error::Ref),
    }
}

pub fn eval(sheet: &Sheet, expr: &Expr) -> Value {
    trace!("{:?}", expr);
    let v = match expr {
        Expr::Num(n) => Value::Num(*n),
        Expr::Bool(b) => Value::Bool(*b),
        Expr::String(s) => Value::String(s.clone()),
        Expr::Perc(e) => eval_to_num(sheet, e, |n| Value::Num(n / 100.0)),
        Expr::Neg(e) => eval_to_num(sheet, e, |n| Value::Num(-n)),
        Expr::Add(l, r) => eval_to_num_2(sheet, l, r, |l, r| Value::Num(l + r)),
        Expr::Sub(l, r) => eval_to_num_2(sheet, l, r, |l, r| Value::Num(l - r)),
        Expr::Mul(l, r) => eval_to_num_2(sheet, l, r, |l, r| Value::Num(l * r)),
        // FIXME: handle division by zero
        Expr::Div(l, r) => eval_to_num_2(sheet, l, r, |l, r| Value::Num(l / r)),
        Expr::Pow(l, r) => eval_to_num_2(sheet, l, r, |l, r| Value::Num(l.powf(r))),

        Expr::Ref(r) => eval_ref(sheet, r),
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
        let (res, _errs) = parser().parse_recovery_verbose(formula);
        res.unwrap()
    }

    fn load_ods(path: &'static str) -> Sheet {
        let mut sheet = Sheet::default();
        let wb = spreadsheet_ods::read_ods(path).unwrap();
        let ods_sheet = wb.sheet(0);
        ods_sheet.iter().fold((), |_, ((row, col), cell)| {
            let cell = Cell {
                value: ods_to_value(cell.value()),
                expr: cell.formula().map(|f| ods_to_expr(f)),
            };
            sheet.set(row as usize, col as usize, cell);
        });
        sheet
    }

    #[dir_test(
        dir: "$CARGO_MANIFEST_DIR/fixtures",
        glob: "**/*.ods",
        loader: load_ods,
    )]
    fn test_ods(fixture: Fixture<Sheet>) {
        let sheet = fixture.content();
        for ((x, y), cell) in sheet.iter() {
            trace!("{},{}: {:?}", x, y, cell);
            if cell.value.is_some() && cell.expr.is_some() {
                let val = cell.value.clone().unwrap();
                let expr = cell.expr.clone().unwrap();
                let eval_val = eval(sheet, &expr);
                assert_eq!(val, eval_val);
            }
        }
    }
}
