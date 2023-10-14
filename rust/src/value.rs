use core::fmt;
use indexmap::IndexMap;

/// An atto value is an atom, a list or a document.
///
/// ```
/// # use atto::Value;
/// let atom = Value::Atom("atom".to_owned());
/// ```
#[derive(Clone, Eq, PartialEq)]
pub enum Value {
    Atom(String),
    List(Vec<Value>),
    Document(IndexMap<String, Value>),
}

impl fmt::Debug for Value {
    /// Debug format an atto value as a string.
    ///
    /// ```
    /// # use atto::Value::{self, Atom, List};
    /// let value = List(vec![ Atom("a".to_owned()), List(vec![]) ]);
    ///
    /// assert_eq!(format!("{value:?}"), "List [Atom(a), List []]");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
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
    /*
    /// Display an atto value as a string.
    ///
    /// ```
    /// # use atto::Value::{self, Atom, List};
    /// let value = List(vec![ Atom("a".to_owned()), List(vec![]) ]);
    ///
    /// assert_eq!(format!("{value}"), "(a ())");
    /// ```
     */
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {}
