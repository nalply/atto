use std::fmt;

#[derive(Clone, Default, Eq, PartialEq)]
pub struct Token {
  pub id:    u16,
  pub data:  Vec<u8>,
  pub index: usize,
}

impl fmt::Debug for Token {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // Toggle alternate, i.e. Display alternate == Debug !alternate
    if f.alternate() {
      write!(f, "{self}")
    } else {
      write!(f, "{self:#}")
    }
  }
}

impl fmt::Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let id = self.id;
    let index = self.index;

    // Shorten to length 20 and display (using replacement character)
    let len = self.data.len();
    let data = self.data.as_slice();
    let data = if len > 20 {
      let head = String::from_utf8_lossy(&data[..10]);
      let tail = String::from_utf8_lossy(&data[len - 8..]);
      format!("{head}..{tail}")
    } else {
      String::from_utf8_lossy(data).to_string()
    };
    let data: String = data
      .chars()
      .map(|c| if c.is_control() { '\u{fffd}' } else { c })
      .collect();

    if f.alternate() {
      write!(f, "Token(#{id} @{index} `{data}`)")
    } else {
      write!(f, "`{data}`#{id}@{index}")
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn token(data: &[u8]) -> Token {
    let data = data.to_vec();
    Token { data, ..Token::default() }
  }

  #[test]
  fn test_token_display() {
    let tok = token(b"01234567890123456789");
    assert_eq!(format!("{tok}"), "`01234567890123456789`#0@0");
    assert_eq!(format!("{tok:?}"), "Token(#0 @0 `01234567890123456789`)");
    assert_eq!(format!("{tok:#}"), "Token(#0 @0 `01234567890123456789`)");
    assert_eq!(format!("{tok:#?}"), "`01234567890123456789`#0@0");
    let tok = token(b"012345678901234567890");
    assert_eq!(format!("{tok}"), "`0123456789..34567890`#0@0");
    let tok = token(b"\0a\xa0b\xe2\x80\x8dc\xef\xbb\xbfd");
    assert_eq!(format!("{tok}"), "`\u{fffd}a\u{fffd}b\u{200d}c\u{feff}d`#0@0");
  }
}
