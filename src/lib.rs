use enum_as_inner::EnumAsInner;

pub mod parser;
pub mod sql;

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

pub fn column_to_id<S: AsRef<str>>(col: S) -> Result<usize, ()> {
    let col = col.as_ref();
    let len = col.len();
    if len == 0 {
        return Err(());
    }
    let col = col.to_uppercase().chars().collect::<String>();
    if !col.chars().all(|x| char::is_ascii_uppercase(&x)) {
        return Err(());
    }
    let mut sum = 0;
    for c in col.chars() {
        sum *= 26;
        sum += (c as u8) as usize - 65 + 1;
    }
    // return sum - 1 to let idx start at 0
    Ok(sum - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_ids() {
        assert_eq!(column_to_id("A"), Ok(0));
        assert_eq!(column_to_id("B"), Ok(1));
        assert_eq!(column_to_id("C"), Ok(2));
        assert_eq!(column_to_id("AA"), Ok(26));
        assert_eq!(column_to_id("Aa"), Ok(26));
        assert_eq!(column_to_id("aA"), Ok(26));
        assert_eq!(column_to_id("ab"), Ok(27));
        assert_eq!(column_to_id("AB"), Ok(27));
        assert_eq!(column_to_id("XFD"), Ok(16383));

        assert_eq!(column_to_id(""), Err(()));
        assert_eq!(column_to_id("%"), Err(()));
        assert_eq!(column_to_id("A0F"), Err(()));
    }
}
