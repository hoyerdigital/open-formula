use calc_sql::parser::parser;
use calc_sql::sql::transform;
use chumsky::Parser;
use inquire::autocompletion::{Autocomplete, Replacement};
use inquire::{CustomUserError, Text};

#[derive(Clone)]
struct Complete {}

impl Autocomplete for Complete {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        let (expr, err) = parser().parse_recovery_verbose(input);
        if let Some(expr) = expr {
            Ok(vec![format!("{}", transform(&expr)?)])
        } else {
            // TODO: use ariadne instead
            Ok(vec![format!("{:?}", err)])
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
    let name = Text::new("What is your name?")
        .with_autocomplete(auto)
        .prompt();
}
