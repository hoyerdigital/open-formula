use enum_as_inner::EnumAsInner;

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
    CellRef(String, usize),
    CellRange((String, usize), (String, usize)),
}
