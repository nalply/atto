#![feature(
  const_option,
  const_type_name,
  fn_traits,
  lazy_cell,
  trait_alias,
  type_alias_impl_trait,
  type_name_of_val,
  unboxed_closures
)]
#![forbid(unsafe_code)]

pub mod rule;
pub mod token;

use crate::token::Token;

#[allow(unused_imports)]
use axlog::*;
use std::fmt;

pub trait StateBounds = fmt::Debug + 'static;

#[derive(Clone, Debug)]
pub struct TokenIterator<'i, S: StateBounds> {
  input:    &'i [u8],
  index:    usize,
  grammar:  &'static rule::Grammar<S>,
  group_id: u16,
  state:    S,
}

impl<'i, S: StateBounds> TokenIterator<'i, S> {
  pub fn start(
    input: &'i [u8],
    grammar: &'static rule::Grammar<S>,
    state: S,
  ) -> TokenIterator<'i, S> {
    TokenIterator { input, index: 0, grammar, group_id: 1, state }
  }
}

impl<'i, S: StateBounds> Iterator for TokenIterator<'i, S> {
  type Item = Token;

  fn next(&mut self) -> Option<Self::Item> {
    return None;

    /*
    {
      let group_id = self.group_id;
      let index = self.index;
      debug!("next(): group_id={group_id} index={index}");
    }

    if self.index == self.input.len() {
      if self.group_id == 1 {
        trace!("end of input");
        return None;
      } else {
        trace!("unexpected_end");
        self.group_id = 1; // this ends the iterator on the next iteration
        return Some(Token {
          index: self.index,
          id:    self.rules.unexpected_end().id,
          data:  vec![],
        });
      }
    }

    for rule in self.rules.group(self.group_id) {
      let rx = &rule.rx;
      trace!("{rule:?}");
      if let Some(found) = rx.find_at(self.input, self.index) {
        let mut token = Token {
          id:    rule.id,
          index: self.index,
          data:  found.as_bytes().to_vec(),
        };
        let mut state = &mut self.state;
        trace!("match  {token}");

        if let Some(token) = (rule.action)(&mut token, &mut state) {
          // todo self.index += state.token_len().unwrap_or(token.data.len())
          self.index += token.data.len();
          trace!("return {token} index {}", self.index);

          return Some(token.clone()); // todo remove clone()
        }
        trace!("rejected in action");
      } else {
        trace!("no match");
      }
    }

    // todo add catch_all to all groups
    let rule = self.rules.catch_all();
    let input = String::from_utf8_lossy(self.input);
    trace!("catch-all {rule:?} input {input} @{}", self.index);
    if let Some(found) = rule.rx.find_at(self.input, self.index) {
      let token = Token {
        id:    rule.id,
        index: self.index,
        data:  found.as_bytes().to_vec(),
      };
      self.index += token.data.len();
      trace!("return catch-all {token:#} index {}", self.index);
      return Some(token);
    }
    */
    unreachable!("catch all rule should have caught invalid input");
  }
}

/*
#[cfg(test)]
mod tests {
  use super::*;

  fn token(id: u16, data: &[u8], index: usize) -> Token {
    let data = data.to_vec();
    Token { id, data, index }
  }

  #[test]
  fn test_simple_grammar() {
    crate::grammar! {
      pub mod grammar where type State = () {
        ALL: [],
        INIT: [
          ALPHA: Rule{ group: 1, r"[a-z]+" }
        ],
        SECOND: [
          DIGIT: Rule{ r"[0-9]+" },
          DOT: Rule{ r"\." },
        ]
      };
    };

    axlog::init("T");
    let tokens: Vec<Token> =
      TokenIterator::start(b"test.0", rules::create(), ()).collect();
    debug!("tokens: {tokens:?}");

    assert_eq!(tokens[0], token(rules::ALPHA, b"test", 0));
    assert_eq!(tokens[1], token(rules::CATCH_ALL, b".0", 4));
  }
}
*/
