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

#[cfg(test)]
mod tests {}
