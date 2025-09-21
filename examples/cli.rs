use chumsky::Parser;
use inquire::autocompletion::{Autocomplete, Replacement};
use inquire::{CustomUserError, Text};
use open_formula::parser::parser;
use open_formula::sql::transform_with_columns;

#[derive(Clone)]
struct Complete {}

impl Autocomplete for Complete {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        let cols = ["A", "B", "C", "D", "E", "F"].map(String::from).to_vec();
        let res = parser().parse(input);
        if res.has_output() {
            let expr = res.into_output().unwrap();
            let sql = transform_with_columns(&expr, &cols);
            if let Ok(sql) = sql {
                Ok(vec![format!("{}", sql)])
            } else {
                Ok(vec![format!("error: {:?}", sql.unwrap_err())])
            }
        } else if res.has_errors() {
            // TODO: use ariadne instead
            Ok(vec![format!("error: {:?}", res.into_errors())])
        } else {
            Ok(vec![])
        }
    }

    fn get_completion(
        &mut self,
        _input: &str,
        _highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, CustomUserError> {
        Ok(Replacement::None)
    }
}

fn main() {
    let auto = Complete {};
    let _ = Text::new("formula >").with_autocomplete(auto).prompt();
}
