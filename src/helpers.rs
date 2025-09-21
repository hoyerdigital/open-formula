use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub enum RefError {
    EmptyReference,
    MalformedReference,
}

impl Display for RefError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyReference => write!(f, "empty reference"),
            Self::MalformedReference => write!(f, "malformed reference"),
        }
    }
}

pub fn column_to_id<S: AsRef<str>>(col: S) -> Result<usize, RefError> {
    let col = col.as_ref();
    let len = col.len();
    if len == 0 {
        return Err(RefError::EmptyReference);
    }
    let mut sum = 0;
    for c in col.chars() {
        if !c.is_ascii_uppercase() {
            return Err(RefError::MalformedReference);
        }
        sum *= 26;
        sum += (c as u8) as usize - 65 + 1;
    }
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
        assert_eq!(column_to_id("AB"), Ok(27));
        assert_eq!(column_to_id("XFD"), Ok(16383));

        assert_eq!(column_to_id(""), Err(RefError::EmptyReference));
        assert_eq!(column_to_id("%"), Err(RefError::MalformedReference));
        assert_eq!(column_to_id("A0F"), Err(RefError::MalformedReference));
        assert_eq!(column_to_id("Aa"), Err(RefError::MalformedReference));
        assert_eq!(column_to_id("aA"), Err(RefError::MalformedReference));
        assert_eq!(column_to_id("ab"), Err(RefError::MalformedReference));
    }
}
