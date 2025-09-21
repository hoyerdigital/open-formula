use enum_as_inner::EnumAsInner;
use num_enum::{IntoPrimitive, TryFromPrimitive};

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

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Num(f64),
    String(String),
    Bool(bool),
    EmptyCell,
    Ref(Ref),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Comp {
    Equal,
    NotEqual,
    Lower,
    Greater,
    LowerEqual,
    GreaterEqual,
}

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

#[derive(Debug, Clone, EnumAsInner, PartialEq)]
pub enum Ref {
    CellRef(usize, usize),
    ColumnRange(usize, usize),
    RowRange(usize, usize),
    CellRange((usize, usize), (usize, usize)),
}

impl Expr {
    pub fn refs(&self) -> Box<dyn Iterator<Item = Ref>> {
        match self {
            Expr::Perc(a) | Expr::Neg(a) => a.refs(),
            Expr::Add(a, b) => Box::new(a.refs().chain(b.refs())),
            Expr::Ref(r) => Box::new(std::iter::once(r.clone())),
            _ => Box::new(std::iter::empty()),
        }
    }
}
