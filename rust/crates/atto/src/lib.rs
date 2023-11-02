#![feature(
  fn_traits,
  lazy_cell,
  trait_alias,
  type_name_of_val,
  unboxed_closures
)]

mod action;
mod lex;
mod parser;
mod rx;
mod token;
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
