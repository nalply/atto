#![allow(dead_code)]

/*
use action::Action;
use value::Token;

use axlog::*;
use lazy_regex::lazy_regex;
use lazy_regex::regex::bytes;
use once_cell::sync::Lazy;
use std::fmt;


pub const INVALID: usize = 0; // second last in group ALL

pub const UNEXPECTED_END: usize = 1; // last in group ALL

pub const ALL: usize = 0;

pub const INIT: usize = 1;

#[derive(Clone)]
pub struct LexRegex(&'static bytes::Regex);

impl fmt::Debug for LexRegex {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "/{}/", self.0.as_str())
  }
}

impl LexRegex {
  pub fn new(rx: &'static bytes::Regex) -> LexRegex { LexRegex(rx) }
}

static RX_INVALID: Lazy<bytes::Regex> = lazy_regex!(r".+"B);

static RX_UNEXPECTED_END: Lazy<bytes::Regex> = lazy_regex!(r"^[&&]"B);

impl Default for LexRegex {
  fn default() -> Self { LexRegex::new(&RX_INVALID) }
}

impl std::ops::Deref for LexRegex {
  type Target = &'static bytes::Regex;

  fn deref(&self) -> &&'static bytes::Regex { &self.0 }
}

impl PartialEq for LexRegex {
  fn eq(&self, other: &Self) -> bool { self.0 as *const _ == other.0 }
}

impl Eq for LexRegex {}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Rule {
  id:     usize,
  rx:     LexRegex,
  action: Action,
  push:   Option<usize>,
  pop:    bool,
}

impl Rule {
  pub fn from_rx(id: usize, rx: &'static bytes::Regex) -> &'static Rule {
    Box::leak(Box::new(Rule {
      id,
      rx: LexRegex(Box::leak(Box::new(rx))),
      action: Action::default(),
      push: None,
      pop: false,
    }))
  }

  pub fn unexpected_end() -> &'static Rule {
    Rule::from_rx(UNEXPECTED_END, &RX_UNEXPECTED_END)
  }

  pub fn invalid() -> &'static Rule { Rule::from_rx(INVALID, &RX_INVALID) }
}

type RuleGroup = &'static [&'static Rule];

type Rules = &'static [RuleGroup];

#[macro_export]
macro_rules! rule_group {
  ( $( $rule:expr ),+ ) => {
    Box::leak(Box::new([
      $( $rule ),+
    ]))
  }
}

pub fn start(
  rules: Rules,
  input: &[u8],
  initial_state: State,
) -> TokenIterator {
  if rules.len() < 2 {
    panic!("expect at least two groups");
  }
  if rules[0].len() < 2 {
    panic!("expect at least two rules in first group");
  }
  for group in rules.iter().enumerate() {
    if group.1.is_empty() {
      panic!("expect at least one rule in group #{}", group.0);
    }
  }

  TokenIterator {
    input,
    lex: Lex {
      state: initial_state,
      index: 0,
      group: INIT,
    },
    rules,
  }
}

pub struct TokenIterator<'i> {
  input: &'i [u8],
  lex:   Lex,
  rules: Rules,
}

impl<'i> Iterator for TokenIterator<'i> {
  type Item = Token;

  fn next(&mut self) -> Option<Self::Item> {
    let group = self.lex.group;
    let index = self.lex.index;
    debug!("next(): group={group} index={index}");

    if index == self.input.len() {
      if group == INIT {
        trace!("end of input");
        return None;
      } else {
        trace!("invalid input");
        return Some(Token {
          id: INVALID,
          data: vec![],
          index,
          // line: 0,
          // col: 0,
        });
      }
    }

    let group = self.rules[self.lex.group].iter();
    for rule in group.chain(self.rules[ALL].iter()) {
      let rx = &rule.rx;
      trace!("rule={rule:?}");
      if let Some(found) = rx.find_at(self.input, index) {
        let id = rule.id;
        let token = Token {
          id,
          data: found.as_bytes().to_owned(),
          ..Token::default()
        };

        let action = rule.action.0.f;
        let state = &mut self.lex.state;
        trace!("rule matched");
        if let Some(token) = action(token, state) {
          self.lex.index += token.data.len(); // todo? index in action()
          return Some(token);
        }
        trace!("rule rejected in action");
      } else {
        trace!("rule did not match");
      }
    }

    unreachable!("fallback rule INVALID should have caught invalid input");
  }
}

#[derive(Debug)]
pub struct Lex {
  state: State,
  index: usize,
  group: usize,
}

use std::sync::atomic::{AtomicUsize, Ordering};
static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn inc_counter() -> usize { COUNTER.fetch_add(1, Ordering::Relaxed) }

#[macro_export]
macro_rules! rules {
  {
    @ $rule:ident: $rx: expr
  } => {
    {
      static RX: Lazy<bytes::Regex> = $rx;
      Rule::from_rx(inc_counter(), &RX)
    }
  };

  {
    $rules:ident = { $(
      $group:ident: {
       $rule1:ident: $rx1:expr
        $( , $rule:ident: $rx:expr )* $( , )?
      }
    )+ }
  } => {
    #[allow(non_upper_case_globals)]
    static $rules: Rules = [
      group![ Rule::invalid(), Rule::unexpected_end() ],
      $( group![
        rules!( @ $rule1: $rx1),
        $( rules!( @ $rule: $rx ), )*
      ], )*
    ];
  };
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rules_macro1() {
    rules! {
      rules = {
        init: {
          alpha: lazy_regex!(r"[a-z]+"B),
          excl: lazy_regex!(r"!"B),
        }
      }
    }

    assert_eq!(rules.len(), 2);
  }

  fn rules1() -> Rules {
    static RX_ALPHA: Lazy<bytes::Regex> = lazy_regex!(r"[a-z]+"B);
    &[
      rule_group!(Rule::invalid(), Rule::unexpected_end()),
      rule_group!(Rule::from_rx(3, &RX_ALPHA)),
    ]
  }

  #[test]
  fn simple() {
    axlog::init("error");
    let mut lexer = start(rules1(), b"text", State::new());
    let token = lexer.next().unwrap();
    assert_eq!(token.id, 2);
    assert_eq!(token.data.as_slice(), b"text");
    let token = lexer.next();
    assert!(token.is_none());
  }

  #[test]
  fn error1() {
    axlog::init("error");
    let mut lexer = start(rules1(), b"error!", State::new());

    let token = lexer.next().unwrap();
    assert_eq!(token.id, 2);
    assert_eq!(token.data.as_slice(), b"error");

    let token = lexer.next().unwrap();
    assert_eq!(token.id, INVALID);
    assert_eq!(token.data.as_slice(), b"!");

    let token = lexer.next();
    assert!(token.is_none());
  }
}

#[allow(unused_macros, clippy::items_after_test_module)]
macro_rules! const_assert {
  ($x:expr $(,)?) => {
    const _: [();
      0 - !{
        const ASSERT: bool = $x;
        ASSERT
      } as usize] = [];
  };
}

*/
