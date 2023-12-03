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

pub use crate::token::Token;
use axlog::*;
use std::fmt;

pub trait StateBounds = fmt::Debug + 'static;

#[derive(Clone, Debug)]
pub struct TokenIterator<'i, S: StateBounds> {
  input:    &'i [u8],
  index:    usize,
  lexer:    &'static rule::Lexer<S>,
  group_id: u8,
  state:    S,
}

impl<'i, S: StateBounds> TokenIterator<'i, S> {
  pub fn start(
    input: &'i [u8],
    lexer: &'static rule::Lexer<S>,
    state: S,
  ) -> TokenIterator<'i, S> {
    let group_id = rule::START_GROUP_ID;
    TokenIterator { input, index: 0, lexer, group_id, state }
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

    let start_id = rule::START_GROUP_ID;

    if self.index == self.input.len() {
      if self.group_id == start_id {
        trace!("end of input");
        return None;
      } else {
        trace!("unexpected_end");
        let group_id = self.group_id;
        self.group_id = start_id; // this ends the iterator on the next iteration
        return Some(Token {
          index: self.index,
          rule_id: self.lexer.unexpected_end.rule_id,
          group_id,
          data: vec![],
        });
      }
    }

    let group = self.lexer.groups[self.group_id as usize].iter();
    let all = self.lexer.groups[0].iter();
    let catch_all = std::iter::once(&self.lexer.catch_all);
    for rule in group.chain(all).chain(catch_all) {
      let rx = &rule.lazy_regex;
      trace!("{rule}");
      if let Some(found) = rx.find(&self.input[self.index..]) {
        let data = found.as_bytes().to_vec();
        let group_id = rule.to_group_id.unwrap_or(self.group_id);
        let token = Token {
          rule_id: rule.rule_id,
          group_id,
          index: self.index + data.len(),
          data: found.as_bytes().to_vec(),
        };
        trace!("found  {token} index={}", self.index);

        let state = &mut self.state;
        if let Some(token) = (rule.action_fn)(token, state) {
          self.index = token.index;
          trace!("return {token} index={}", self.index);

          self.group_id = token.group_id;

          return Some(token);
        }
        trace!("rejected in action");
      } else {
        trace!("no match");
      }
    }

    self.index = self.input.len();
    Some(Token {
      rule_id:  self.lexer.unexpected_end.rule_id,
      group_id: self.group_id,
      data:     vec![],
      index:    self.index,
    })
    // unreachable!("catch all rule should have caught invalid input");
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn token(rule_id: u16, group_id: u8, data: &[u8], index: usize) -> Token {
    let data = data.to_vec();
    Token { rule_id, group_id, data, index }
  }

  #[test]
  fn test_lexer() {
    fn action_test(token: Token, _: &mut ()) -> Option<Token> {
      let group_id = G_ID_init;
      let b = token.data;
      let data = if b.as_slice() == b"0" { b"zero".to_vec() } else { b };
      Some(Token { group_id, data, ..token })
    }

    crate::lexer! {
      lexer<()> {
        ALL: [],
        init: [
          alpha("[a-z_]+", to=second),
        ],
        second: [
          digit("[0-9]+", action=test),
          dot(r"\." ),
        ],
      };
    };

    fn start(data: &[u8]) -> Vec<Token> {
      TokenIterator::start(data, &LEXER, ()).collect()
    }

    fn dbg(t: &[Token]) {
      let s = t.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", ");
      debug!("tokens: {s}");
    }

    axlog::init("T");

    let tokens = start(b"test.0");
    dbg(&tokens);
    assert_eq!(tokens[0], token(R_ID_alpha, G_ID_second, b"test", 4));
    assert_eq!(tokens[1], token(R_ID_dot, G_ID_second, b".", 5));
    assert_eq!(tokens[2], token(R_ID_digit, G_ID_init, b"zero", 6));
    assert_eq!(tokens.len(), 3);

    let tokens = start(b"some_text42");
    dbg(&tokens);
    assert_eq!(tokens[0], token(R_ID_alpha, G_ID_second, b"some_text", 9));
    assert_eq!(tokens[1], token(R_ID_digit, G_ID_init, b"42", 11));
    assert_eq!(tokens.len(), 2);
  }
}
