//! OpenFormula [types](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#__RefHeading__1017876_715980110) mapped to rust enums.

use enum_as_inner::EnumAsInner;
use num_enum::{IntoPrimitive, TryFromPrimitive};

/// OpenFormula [Error](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#__RefHeading__1017900_715980110) type.
#[derive(Debug, Clone, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Error {
    // OpenDocument / Google Sheets / Excel
    Null = 1,
    Div0 = 2,
    Value = 3,
    Ref = 4,
    Name = 5,
    Num = 6,
    NotAvailable = 7,
    // Excel
    GettingData = 8,
    // Custom
    Unimplemented = 9,
    Args = 10,
}

/// A result type that uses an OpenFormula [Error] type.
pub type Result<T> = std::result::Result<T, Error>;

/// An OpenFormula value.
///
/// Any type that is *not* a [pseudo type](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#__RefHeading__1017910_715980110).
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Num(f64),
    String(String),
    Bool(bool),
    EmptyCell,
    Ref(Ref),
}

/// A comparison operator.
///
/// Comparion operators used in expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum Comp {
    Equal,
    NotEqual,
    Lower,
    Greater,
    LowerEqual,
    GreaterEqual,
}

/// An OpenFomula [expression](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#__RefHeading__1017930_715980110).
#[derive(Debug, Clone, EnumAsInner, PartialEq)]
pub enum Expr {
    Num(f64),
    Bool(bool),
    String(String),
    Range(Box<Self>, Box<Self>),
    RefIntersection(Box<Self>, Box<Self>),
    RefUnion(Box<Self>, Box<Self>),
    Perc(Box<Self>),
    Neg(Box<Self>),
    Add(Box<Self>, Box<Self>),
    Sub(Box<Self>, Box<Self>),
    Mul(Box<Self>, Box<Self>),
    Div(Box<Self>, Box<Self>),
    Pow(Box<Self>, Box<Self>),
    Concat(Box<Self>, Box<Self>),
    Cond(Comp, Box<Self>, Box<Self>),
    Func(String, Vec<Self>),
    Ref(Ref),
}

/// An OpenFomula [reference](https://docs.oasis-open.org/office/OpenDocument/v1.4/csd01/part4-formula/OpenDocument-v1.4-csd01-part4-formula.html#__RefHeading__74715_1363921367).
#[derive(Debug, Clone, EnumAsInner, PartialEq)]
pub enum Ref {
    CellRef(usize, usize),
    ColumnRange(usize, usize),
    RowRange(usize, usize),
    CellRange((usize, usize), (usize, usize)),
}

impl Expr {
    /// Returns all references that are used in an expression.
    pub fn refs(&self) -> Box<dyn Iterator<Item = Ref>> {
        match self {
            Expr::Perc(a) | Expr::Neg(a) => a.refs(),
            Expr::Add(a, b) => Box::new(a.refs().chain(b.refs())),
            Expr::Ref(r) => Box::new(std::iter::once(r.clone())),
            _ => Box::new(std::iter::empty()),
        }
    }
}
