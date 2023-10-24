mod lex;
mod parser;
mod value;

pub use value::Value;

/// Dummy test to exercise test infrastructure
#[cfg(test)]
mod tests {
  use super::value::Value::*;

  #[test]
  fn test_test() {
    let atom = Atom("atom".to_owned());
    assert!(matches!(atom, Atom(string) if &string == "atom"));
  }
}
