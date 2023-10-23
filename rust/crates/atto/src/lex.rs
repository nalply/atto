#![allow(dead_code)]

use regex::bytes::Regex;
use std::fmt;
use std::sync::OnceLock;

pub const INVALID: usize = 0; // second last in group ALL

pub const UNEXPECTED_END: usize = 1; // last in group ALL

pub const ALL: usize = 0;

pub const INIT: usize = 1;

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct Token {
  rule_id:  usize,
  droup_id: usize,
  data:     Vec<u8>,
  line:     u32,
  col:      u32,
  index:    usize,
}

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct ActionToken {
  rule_id:  usize,
  group_id: usize,
  data:     Vec<u8>,
}

// fn pointer because Fn traits include closures capturing non-Sync variables
type ActionFn = fn(Token) -> Option<ActionToken>;

pub struct ActionStruct(ActionFn, &'static str);

pub type Action = &'static ActionStruct;

impl fmt::Debug for ActionStruct {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Action({})", self.1)
  }
}

impl PartialEq for ActionStruct {
  fn eq(&self, other: &Self) -> bool { self.0 == other.0 }
}

macro_rules! once {
    { $x:ident: $t:ty; } => {  static $x: OnceLock<$t> = OnceLock::new(); }
}
once! { IDENTITY: ActionStruct; }
once! { RX_INVALID: Regex; }
once! { RX_UNEXPECTED_END: Regex; }

pub fn identity() -> Action {
  fn id(token: Token) -> Option<ActionToken> {
    let Token {
      rule_id,
      group_id,
      data,
      ..
    } = token;
    Some(ActionToken {
      rule_id,
      group_id,
      data,
    })
  }

  IDENTITY.get_or_init(|| ActionStruct(id as ActionFn, "identity"))
}

#[derive(Clone, Debug)]
pub struct Rule {
  rx:     Regex,
  action: Action,
  push:   Option<usize>,
  pop:    bool,
}

// Regex does not implement PartialEq
impl PartialEq for Rule {
  fn eq(&self, other: &Self) -> bool {
    self.rx.as_str() == other.rx.as_str()
      && self.action == other.action
      && self.push == other.push
      && self.pop == other.pop
  }
}

pub fn rule_invalid() -> Rule {
  let rx = RX_INVALID
    .get_or_init(|| Regex::new("^.+").unwrap())
    .clone();
  let action = identity();

  Rule {
    rx,
    action,
    push: None,
    pop: false,
  }
}

pub fn rule_unexpected_end() -> Rule {
  let rx = RX_UNEXPECTED_END
    .get_or_init(|| Regex::new("[&&]").unwrap())
    .clone();
  let action = identity();

  Rule {
    rx,
    action,
    push: None,
    pop: false,
  }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Lexer {
  rule_groups:   Vec<Vec<Rule>>,
  input:         Vec<u8>,
  curr_group_id: usize,
}

impl Lexer {
  fn new(rule_groups: Vec<Vec<Rule>>) -> Lexer {
    if rule_groups.len() < 2 {
      panic!("expect at least two groups");
    }
    if rule_groups[0].len() < 2 {
      panic!("expect at least two rules in GROUP_ALL");
    }
    for group in rule_groups.iter() {
      if group.is_empty() {
        panic!("expect at least one rule in group");
      }
    }

    let input = Vec::new();
    let curr_group_id = INIT;
    Lexer {
      rule_groups,
      input,
      curr_group_id,
    }
  }
}

impl Iterator for Lexer {
  type Item = Token;

  fn next(&mut self) -> Option<Self::Item> {
    if self.input.is_empty() {
      if self.curr_group_id == INIT {
        return None;
      } else {
        return Some(Token {
          rule_id:  INVALID,
          group_id: self.curr_group_id,
          data:     vec![],
          index:    0, // todo
          line:     0,
          col:      0,
        });
      }
    }

    Some(Token {
      rule_id:  2,
      group_id: ALL,
      data:     vec![],
      index:    0, // todo
      line:     0,
      col:      0,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn simple() {
    let mut lexer = Lexer::new(vec![
      vec![rule_invalid(), rule_unexpected_end()],
      vec![Rule {
        rx:     Regex::new("[a-z]+").unwrap(),
        action: identity(),
        push:   None,
        pop:    false,
      }],
    ]);
    lexer.input = b"test".to_vec();

    println!("{:?}", identity());

    let token = lexer.next().unwrap();
    assert_eq!(token.rule_id, 2);
    assert_eq!(token.group_id, ALL);

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
