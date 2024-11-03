#[derive(Debug, PartialEq, derive_more::Display, derive_more::Error)]
pub enum Error {
    EmptyReference,
    MalformedReference,
}

pub fn column_to_id<S: AsRef<str>>(col: S) -> Result<usize, Error> {
    let col = col.as_ref();
    let len = col.len();
    if len == 0 {
        return Err(Error::EmptyReference);
    }
    let col = col.to_uppercase().chars().collect::<String>();
    if !col.chars().all(|x| char::is_ascii_uppercase(&x)) {
        return Err(Error::MalformedReference);
    }
    let mut sum = 0;
    for c in col.chars() {
        sum *= 26;
        sum += (c as u8) as usize - 65 + 1;
    }
    Ok(sum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_ids() {
        assert_eq!(column_to_id("A"), Ok(1));
        assert_eq!(column_to_id("B"), Ok(2));
        assert_eq!(column_to_id("C"), Ok(3));
        assert_eq!(column_to_id("AA"), Ok(27));
        assert_eq!(column_to_id("Aa"), Ok(27));
        assert_eq!(column_to_id("aA"), Ok(27));
        assert_eq!(column_to_id("ab"), Ok(28));
        assert_eq!(column_to_id("AB"), Ok(28));
        assert_eq!(column_to_id("XFD"), Ok(16384));

        assert_eq!(column_to_id(""), Err(Error::EmptyReference));
        assert_eq!(column_to_id("%"), Err(Error::MalformedReference));
        assert_eq!(column_to_id("A0F"), Err(Error::MalformedReference));
    }
}
