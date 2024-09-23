#[derive(Debug)]
pub struct Sheet {}

#[cfg(test)]
mod tests {
    use super::*;
    use dir_test::{dir_test, Fixture};

    fn load_ods(path: &'static str) -> Sheet {
        dbg!(path);
        let wb = spreadsheet_ods::read_ods(path).unwrap();
        let sheet = wb.sheet(0);
        sheet.iter().fold((), |_, ((row, col), cell)| {
            dbg!(row, col, cell.value(), cell.formula());
        });
        Sheet {}
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
