## OpenFormula Parser and Evaluator

A spreadsheet formula parser and evaluator that conforms to the [Open Document Format for Office Applications Version 1.4 Format](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html).

## Example Usage

```rust
use open_formula::prelude::*;

fn main() {
    let mut context = Context::default();
    // Add two cells to our worksheet:
    // A   | B
    // ---------
    // 5.0 | 2.0
    context.sheet.set(
        0,
        0,
        Cell {
            value: Some(Value::Num(5.0)),
            expr: None,
        },
    );
    context.sheet.set(
        1,
        0,
        Cell {
            value: Some(Value::Num(2.0)),
            expr: None,
        },
    );
    // Parse formula string
    let expr = parser().parse("A1+B1").unwrap();
    // Evaluate formula expression
    println!("{:?}", eval(&context, &expr));
    // Ok(Num(7.0))
}
```

## Implementation Status

Currently parsing is mostly complete and main focus is to get a proper [OpenDocument Formula Small Group Evaluator](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#__RefHeading__711846_826425813) working.

### Small Group Evaluator

| Specification   | Status         |
| --------------- | -------------- |
| [Basic Limits](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Basic_Limits) | 🟢 |
| **Syntax / Types** |             |
| [Criteria](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Criteria) | 🟢 |
| [Basic Expressions](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Basic_Expressions) | 🟢 |
| [Constant Numbers](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Constant_Numbers) | 🟢 |
| [Constant Strings](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Constant_Strings) | 🟢 |
| [Operators](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Operators) | 🟢 |
| [Functions and Parameters](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Functions_and_Function_Parameters) | 🟢 |
| [Nonstandard Function Names](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#NonstandardFunctionNames) | 🟢 |
| [References](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#References) | 🟡 |
| [Errors](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Constant_Errors) | 🟢 |
| [Whitespace](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Whitespace) | 🟢 |
| **Implicit Conversion** |         |
| [Text](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Conversion_to_Text) | 🟢 |
| [Number](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Conversion_to_Number) | 🟢 |
| [Logical](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Conversion_to_Logical) | 🟢 |
| **Operators**    |                |
| [Infix Operator Ordered Comparison](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#__RefHeading__1018028_715980110) | 🟢 |
| [Infix Operator "&"](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#a_6_4_10_Infix_Operator_)| 🟢 |
| [Infix Operator "+"](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Infix_Operator_PLUS) | 🟢  |
| [Infix Operator "-"](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Infix_Operator_MINUS) | 🟢 |
| [Infix Operator "*"](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Infix_Operator_MUL) | 🟢 |
| [Infix Operator "/"](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Infix_Operator_DIV) | 🟢 |
| [Infix Operator "^"](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Infix_Operator_POW) | 🟢 |
| [Infix Operator "="](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Infix_Operator_EQ) | 🟢 |
| [Infix Operator "<>"](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Infix_Operator_NE) | 🟢 |
| [Postfix Operator "%"](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Postfix_Operator_PERCENT) | 🟢 |
| [Prefix Operator "+"](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Prefix_Operator_PLUS) | 🟢 |
| [Prefix Operator "-"](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Prefix_Operator_MINUS) | 🟢 |
| [Infix Operator Reference Intersection ("!")](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Infix_Operator_Reference_Intersection) | 🔴 |
| [Infix Operator Reference Range (":")](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Infix_Operator_Reference_Range) | 🟡 |
| **Functions**   |                |
| [Functions as defined in 2.3.2 E)](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#__RefHeading__711846_826425813) | 🟡 (79 missing) |

Missing functions can be checked with:

```rust
cargo test funcs_missing_small -- --ignored
```

### Medium Group Evaluator

| Specification   | Status         |
| --------------- | -------------- |
| **Functions**   |                |
| [Functions as defined in 2.3.3 A)](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#__RefHeading__711848_826425813) | 🟡 (241 missing) |
| **Operators**   |                |
| [Infix Operator Reference Concatenation ("~") (aka Union)](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Infix_Operator_Reference_Union) | 🔴 |
| [References with more than one area] | 🔴 |

```rust
cargo test funcs_missing_medium -- --ignored
```

### Large Group Evaluator

| Specification   | Status         |
| --------------- | -------------- |
| **Syntax**      |                |
| [Inline Arrays](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Inline_Arrays) | 🔴 |
| [Automatic Intersection](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#AutomaticIntersection) | 🔴 |
| [External Named Expressions](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Named_Expressions) | 🔴 |
| **Types**       |                |
| [Complex Number Type](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#Complex_Number) | 🔴 |
| **Functions**   |                |
| [Functions as defined in 2.3.4 B)](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#__RefHeading__711850_826425813) | 🟡 (357 missing) |

```rust
cargo test funcs_missing_large -- --ignored
```

## Tests

```rust
cargo test
```

### Tests against OpenOffice/LibreOffice files

As calculated values are stored alongside their formula in ods files, a simple test framework is implemented in [/src/eval.rs](./src/eval.rs).
The test `test_ods` will load any ods file located in [/fixtures](./fixtures), parse each formula, evaluate it and check against the saved
calculated value.

This way it is very easy to check against other spreadsheet engines by just adding more ods files.

## Benchmarks

A very simple minimal test benchmark is currently implemented using [Gungraun](https://crates.io/crates/gungraun).

```rust
cargo bench
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
