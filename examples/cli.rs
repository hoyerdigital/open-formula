use calc_sql::parser::parser;
use calc_sql::sql::transform_with_columns;
use chumsky::Parser;
use inquire::autocompletion::{Autocomplete, Replacement};
use inquire::{CustomUserError, Text};

#[derive(Clone)]
struct Complete {}

impl Autocomplete for Complete {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        let cols = ["A", "B", "C", "D", "E", "F"].map(String::from).to_vec();
        let (expr, err) = parser().parse_recovery_verbose(input);
        if let Some(expr) = expr {
            let sql = transform_with_columns(&expr, &cols);
            if let Ok(sql) = sql {
                Ok(vec![format!("{}", sql)])
            } else {
                Ok(vec![format!("error: {:?}", sql.unwrap_err())])
            }
        } else {
            // TODO: use ariadne instead
            Ok(vec![format!("error: {:?}", err)])
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
