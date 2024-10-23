use ahash::AHashMap;

use crate::types::{Error, Expr, Value};
pub use chumsky::Parser;

#[derive(Debug, Clone)]
pub struct Cell {
    value: Option<Value>,
    expr: Option<Expr>,
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

impl Sheet {
    pub fn has_value(&self, x: usize, y: usize) -> bool {
        self.map.contains_key(&(x, y))
    }

    pub fn get(&self, x: usize, y: usize) -> Option<Cell> {
        self.map.get(&(x, y)).cloned()
    }

    pub fn set(&mut self, x: usize, y: usize, cell: Cell) -> Option<Cell> {
        self.map.insert((x, y), cell)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parser;
    use dir_test::{dir_test, Fixture};

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

        dbg!(formula.clone());
        // FIXME: proper checking/parsing (remove unwraps)
        let (res, _errs) = parser().parse_recovery_verbose(formula);
        res.unwrap()
    }

    fn load_ods(path: &'static str) -> Sheet {
        let mut sheet = Sheet::default();
        dbg!(path);
        let wb = spreadsheet_ods::read_ods(path).unwrap();
        let ods_sheet = wb.sheet(0);
        ods_sheet.iter().fold((), |_, ((row, col), cell)| {
            dbg!(row, col, cell.value(), cell.formula());
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
        dbg!(fixture.content());
    }
}
