use std::fmt;

#[derive(Clone, Default, Eq, PartialEq)]
pub struct Token {
  pub id:   u16,
  pub data: Vec<u8>,
  index:    u32,
}

pub trait Update<T> = for<'t> FnOnce<(&'t mut T,)>;

#[allow(dead_code)]
impl Token {
  pub fn new<V: Into<Vec<u8>>>(id: u16, data: V) -> Token {
    let data = data.into();
    Token { id, data, index: 0 }
  }

  pub fn with_id(mut self, id: u16) -> Token {
    self.id = id;
    self
  }

  pub fn with_data<V: Into<Vec<u8>>>(mut self, data: V) -> Token {
    self.data = data.into();
    self
  }

  pub fn mut_id(mut self, f: impl Update<u16>) -> Token {
    f(&mut self.id);
    self
  }

  pub fn mut_data(mut self, f: impl Update<Vec<u8>>) -> Token {
    f(&mut self.data);
    self
  }
}

impl fmt::Debug for Token {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Display::fmt(self, f)
  }
}

impl fmt::Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fn lossy(v: &[u8]) -> std::borrow::Cow<'_, str> {
      String::from_utf8_lossy(v)
    }

    let data = self.data.as_slice();
    let len = data.len();
    let s = if len > 20 {
      format!("{}..{}", lossy(&data[..10]), lossy(&data[len - 8..]))
    } else {
      lossy(data).to_string()
    };
    let s: String = s
      .chars()
      .map(|c| if c.is_control() { '\u{fffd}' } else { c })
      .collect();
    let id = self.id as i64;

    if f.alternate() {
      write!(f, "Token({id:?} {s})")
    } else {
      write!(f, "{id}`{s}`")
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_token_with_x() {
    let token = Token::default();
    let token = token.with_id(1).with_data(b"test");
    assert_eq!(token.data.as_slice(), b"test");

    assert_eq!(token.to_string(), "1`test`");
  }

  #[test]
  fn test_token_mut_x() {
    let token = Token::default().with_id(1).with_data(b"test");
    let token = token.mut_id(|x: &mut u16| {
      assert_eq!(x, &1);
      *x = 42
    });
    assert_eq!(token.id, 42);

    let token = token.mut_data(|data: &mut Vec<u8>| {
      data.as_mut_slice().iter_mut().for_each(|b| *b += 1)
    });
    assert_eq!(token.data.as_slice(), b"uftu");

    assert_eq!(token.to_string(), "42`uftu`");
  }

  #[test]
  fn test_token_display() {
    let token = Token::default().with_data(b"01234567890123456789");
    assert_eq!(format!("{token}"), "0`01234567890123456789`");
    let token = Token::default().with_data(b"012345678901234567890");
    assert_eq!(format!("{token}"), "0`0123456789..34567890`");
    let token =
      Token::default().with_data(b"\0a\xa0b\xe2\x80\x8dc\xef\xbb\xbfd");
    assert_eq!(
      format!("{token}"),
      "0`\u{fffd}a\u{fffd}b\u{200d}c\u{feff}d`"
    );
  }
}
