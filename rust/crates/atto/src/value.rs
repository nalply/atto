use core::fmt;
use indexmap::IndexMap;

pub type Atom = String;

pub type List = Vec<Value>;

pub type Key = String;

pub type Document = IndexMap<Key, Value>;

/// An atto value is an atom, a list or a document.
///
/// ```
/// # use atto::Value;
/// let atom = Value::Atom("atom".to_owned());
/// ```
#[derive(Clone, Eq, PartialEq)]
pub enum Value {
  Nil,
  Atom(Atom),
  List(List),
  Document(Document),
}

impl Value {
  fn format(&self) -> String {
    match self {
      Value::Nil => "#nil".to_owned(),
      Value::Atom(string) => format_atom(string),
      Value::List(vec) => format_list(vec),
      Value::Document(map) => format_document(map),
    }
  }
}

impl fmt::Debug for Value {
  /// Debug format an atto value as a string.
  ///
  /// ```
  /// # use atto::Value::{self, Atom, List};
  /// let value = List(vec![Atom("a".to_owned()), List(vec![])]);
  ///
  /// assert_eq!(format!("{value:?}"), "List [Atom(a), List []]");
  /// ```
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Value::Nil => write!(f, "Nil"),
      Value::Atom(string) => write!(f, "Atom({string})"),
      Value::List(vec) => {
        f.write_str("List ")?;
        fmt::Debug::fmt(vec, f)
      }
      Value::Document(map) => {
        f.write_str("Document ")?;
        fmt::Debug::fmt(map, f)
      }
    }
  }
}

// Display: A nested empty document is re-read as an empty list
impl fmt::Display for Value {
  /// Display an atto value as a string.
  ///
  /// ```
  /// # use atto::Value::{self, Atom, List};
  /// let value = List(vec![Atom("a".to_owned()), List(vec![])]);
  ///
  /// assert_eq!(format!("{value}"), "(a ())");
  /// ```
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(&self.format())
  }
}

fn format_atom(atom: &str) -> String { atom.to_owned() }

fn format_list(list: &[Value]) -> String {
  let list = list.iter().map(|v| v.format());
  let list = list.collect::<Vec<String>>().join(" ");

  format!("({list})")
}

fn format_entry(key: &str, value: &Value) -> String {
  let key = format_atom(key);
  let value = value.format();

  format!("{key}: {value}")
}

fn format_document(document: &IndexMap<String, Value>) -> String {
  let entries = document.iter().map(|(k, v)| format_entry(k, v));
  let entries = entries.collect::<Vec<String>>().join(" ");

  format!("({entries}")
}

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct Token {
  pub id:   usize,
  pub data: Vec<u8>,
  index:    u32,
  // line:   u32,
  // col: usize,
}

pub trait Update<T> = for<'t> FnOnce<(&'t mut T,)>;

#[allow(dead_code)]
impl Token {
  pub fn with_id(mut self, id: usize) -> Token {
    self.id = id;
    self
  }

  pub fn with_data<V: Into<Vec<u8>>>(mut self, data: V) -> Token {
    self.data = data.into();
    self
  }

  pub fn mut_id(mut self, f: impl Update<usize>) -> Token {
    f(&mut self.id);
    self
  }

  pub fn mut_data(mut self, f: impl Update<Vec<u8>>) -> Token {
    f(&mut self.data);
    self
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

    write!(f, "{}`{s}`", self.id)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_token_with_x() {
    let token = Token::default().with_id(1);
    assert_eq!(token.id, 1);

    let token = token.with_data(b"test");
    assert_eq!(token.data.as_slice(), b"test");

    assert_eq!(token.to_string(), "1`test`");
  }

  #[test]
  fn test_token_mut_x() {
    let token = Token::default().with_id(2).with_data(b"test");
    let token = token.mut_id(|x: &mut usize| *x *= 3);
    assert_eq!(token.id, 6);

    let token = token.mut_data(|data: &mut Vec<u8>| {
      data.as_mut_slice().iter_mut().for_each(|b| *b += 1)
    });
    assert_eq!(token.data.as_slice(), b"uftu");

    assert_eq!(token.to_string(), "6`uftu`");
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
