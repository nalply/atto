#![allow(dead_code)]

use crate::rx::{catch_all, unexpected_end, Rules};
use crate::token::Token;

use axlog::*;
use std::fmt;

pub trait StateBounds = fmt::Debug + 'static;

#[derive(Clone, Debug)]
pub struct TokenIterator<'i, S: StateBounds> {
  input:    &'i [u8],
  index:    usize,
  rules:    &'static Rules<S>,
  group_id: usize,
  state:    S,
}

impl<'i, S: StateBounds> TokenIterator<'i, S> {
  pub fn start(
    input: &'i [u8],
    rules: &'static Rules<S>,
    state: S,
  ) -> TokenIterator<'i, S> {
    TokenIterator {
      input,
      index: 0,
      rules,
      group_id: 0,
      state,
    }
  }
}

impl<'i, S: StateBounds> Iterator for TokenIterator<'i, S> {
  type Item = Token;

  fn next(&mut self) -> Option<Self::Item> {
    {
      let group_id = self.group_id;
      let index = self.index;
      debug!("next(): group_id={group_id} index={index}");
    }

    if self.index == self.input.len() {
      if self.group_id == 0 {
        trace!("end of input");
        return None;
      } else {
        trace!("unexpected_end");
        self.group_id = 0;
        return Some(Token::default().with_id(unexpected_end().id));
      }
    }

    for rule in self.rules.group(self.group_id) {
      let rx = &rule.rx;
      trace!("{rule:?}");
      if let Some(found) = rx.find_at(self.input, self.index) {
        let mut token = Token::new(rule.id, found.as_bytes());
        let mut state = &mut self.state;
        trace!("match  {token:#}");

        if let Some(token) = (rule.action)(&mut token, &mut state) {
          self.index += token.data.len(); // todo? index in action()
          trace!("return {token:#} index {}", self.index);

          return Some(token.clone()); // todo remove clone()
        }
        trace!("rejected in action");
      } else {
        trace!("no match");
      }
    }

    let rule = catch_all();
    let input = String::from_utf8_lossy(self.input);
    trace!("catch-all {rule:?} input {input} @{}", self.index);
    if let Some(found) = rule.rx.find_at(self.input, self.index) {
      let token = Token::new(rule.id, found.as_bytes());
      self.index += token.data.len();
      trace!("return catch-all {token:#} index{}", self.index);
      return Some(token);
    }

    unreachable!("catch all rule should have caught invalid input");
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_simple_lex() {
    crate::rules! {
      #[allow(non_upper_case_globals)]
      static rules: Rules<()> = {
        @all: [],
        init: [
          alpha: Rule{ group: 1, r"[a-z]+" }
        ],
        second: [
          digit: Rule{ r"[0-9]+" },
          dot: Rule{ r"\." },
        ]
      };
    };

    axlog::init("T");
    let tokens: Vec<Token> =
      TokenIterator::start(b"test.0", &rules, ()).collect();

    println!("{tokens:?}");
  }
}
