#![feature(
  fn_traits,
  lazy_cell,
  trait_alias,
  type_name_of_val,
  unboxed_closures
)]

pub mod action;
pub mod lex;
pub mod parser;
pub mod rx;
pub mod token;
pub mod value;

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
