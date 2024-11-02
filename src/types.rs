use enum_as_inner::EnumAsInner;

pub enum Mode {
    Default,
    OpenDocument,
    GoogleSheets,
    Excel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    // OpenDocument / Google Sheets / Excel
    Null,
    Div0,
    Value,
    Ref,
    Name,
    Num,
    NotAvailable,
    // Excel
    GettingData,
    // Custom
    Unimplemented,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Num(f64),
    String(String),
    Bool(bool),
    Err(Error),
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

// FIXME: replace string with numbers?
#[derive(Debug, Clone, EnumAsInner, PartialEq)]
pub enum Ref {
    CellRef(String, usize),
    ColumnRange(String, String),
    RowRange(usize, usize),
    CellRange((String, usize), (String, usize)),
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
