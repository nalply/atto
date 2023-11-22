#![allow(dead_code, unused_imports, unused_macros)]

use crate::action::{ActFn, ActRet, Action};
use crate::token::Token;
use regex::{bytes, escape};
use std::fmt;

/// Lexer rule with a byte regex for matching, an action for additional logic
/// with state of generic type S and optionally a group to switch to.
#[derive(Clone)]
pub struct Rule<S> {
  pub rx:     bytes::Regex,
  pub action: Action<S>,
  pub id:     u16,
  pub group:  Option<u8>,
}

impl<S> fmt::Debug for Rule<S> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Display::fmt(self, f)
  }
}

impl<S> fmt::Display for Rule<S> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let id = self.id;
    let rx = &self.rx.to_string();
    let action = &self.action.name;
    let action_type = std::any::type_name::<S>();
    let group = if let Some(group_id) = self.group {
      format!(" group={group_id}")
    } else {
      String::new()
    };
    write!(f, "Rule({id} /{rx}/ <{action_type}>::{action}(){group})")
  }
}

impl<S> PartialEq for Rule<S> {
  fn eq(&self, other: &Self) -> bool { self.id == other.id }
}

impl<S> Eq for Rule<S> {}

pub type Group<S> = [Rule<S>];

#[derive(Clone, Debug)]
pub struct Rules<S, const R: usize, const G: usize> {
  pub rules:         [Rule<S>; R],
  pub group_indices: [usize; G],
}

impl<S, const R: usize, const G: usize> Rules<S, R, G> {
  pub fn group(&self, group_id: u16) -> &[Rule<S>] {
    let first = self.group_indices[group_id as usize];
    let last = self.group_indices[(group_id + 1) as usize];
    &self.rules[first..last]
  }

  pub fn catch_all(&self) -> &Rule<S> { &self.rules[self.rules.len() - 2] }

  pub fn unexpected_end(&self) -> &Rule<S> { &self.rules[self.rules.len() - 1] }
}

#[macro_export]
macro_rules! indices {
  { $( $idents:ident, )* } => {
    $crate::indices! { @recurse 0, $( $idents, )* }
  };

  { @recurse $index:expr, } => {};

  { @recurse $index:expr, $head:ident, $( $tail:ident, )* } => {
    #[allow(unused)]
    pub const $head: u16 = $index as u16;

    $crate::indices! { @recurse $index + 1, $( $tail, )* }
  };
}

#[macro_export]
macro_rules! count {
  { $( $idents:ident, )* } => {
    $crate::count! { @recurse 0, $( $idents, )* }
  };

  { @recurse $accum:expr, } => { $accum };

  { @recurse $accum:expr, $head:ident, $( $tail:ident, )* } => {
    $crate::count! { @recurse $accum + 1, $( $tail, )* }
  };
}

#[macro_export]
macro_rules! grammar {
  {
    pub $rules:ident<$state:ty> {
      ALL: [
        $( $all_name:ident: Rule { $( $all:tt )+ } $(,)? ),*
      ],
      $(
        $group_name:ident: [ $(
          $rule_name:ident: Rule { $( $rule:tt )+ } $(,)?
        ),+ ]
      ),+
    };
  } => {
    pub mod $rules {
      // Define the group indices 0..__GROUP_COUNT
      $crate::indices!(ALL, $( $group_name, )+ );

     // Define the rule indices from 2
      $crate::indices!($( $all_name, )* $( $( $rule_name, )+ )+ );

      pub const ALL_COUNT: usize = $crate::count!($( $all_name, )* );

      pub const RULE_COUNT: usize
        = 2 + $crate::count!($( $all_name, )* $( $( $rule_name, )+ )+ );

      pub const GROUP_COUNT: usize = 1 + $crate::count!($( $group_name, )+ );

      pub const GROUP_INDEX_COUNT: usize = GROUP_COUNT + 1;

      pub const CATCH_ALL: u16 = (RULE_COUNT - 2) as u16;

      pub const UNEXPECTED_END: u16 = (RULE_COUNT - 1) as u16;

      pub type State = $state;

      pub type Rules = $crate::rules::Rules<State, RULE_COUNT, GROUP_INDEX_COUNT>;

      pub fn create() -> Rules {
        let group_sizes: [usize; GROUP_COUNT] = [
          ALL_COUNT,
          $(
            $crate::count!($( $rule_name, )+ ),
          )+
        ];

        let mut group_indices = [0; GROUP_COUNT + 1];
        for i in 1..group_indices.len() {
          group_indices[i] = group_indices[i - 1] + group_sizes[i - 1];
        }

        $crate::rules::Rules {
          group_indices,
          rules: [
            $(
              $crate::rule!{ $all_name => $( $all ) + },
            )*
            $( $(
              $crate::rule!{ $rule_name => $( $rule )+ },
            )+ )+
            $crate::rule!{ CATCH_ALL => ".+" },
            $crate::rule!{ UNEXPECTED_END => "[&&]" },
          ],
        } as Rules
      }
    }
  };
}

#[macro_export]
macro_rules! rule {
  { $id:expr => action $name:expr => $action:expr; $group:expr, $rx:literal } => {
    $crate::rules::Rule {
      id:     $id,
      rx:     ::regex::bytes::Regex::new($rx).unwrap(),
      action: $crate::action::Action { action_fn: $action, name: $name },
      group:  $group,
    }
  };

  { $id:expr => group: $group:expr, $rx:literal } => {
    $crate::rule!{ $id => action "id" => |token, _| Some(token); Some($group), $rx }
  };

  { $id:expr => action: $action:expr, $rx:literal } => {
    $crate::rule!{ $id => action stringify!($action) => $action; None, $rx }
  };

  { $id:expr => action: $action:expr, group: $group:expr, $rx:literal } => {
    $crate::rule!{ $id => action stringify!($action) => $action; Some($group), $rx }
  };

  { $id:expr => $rx:literal } => {
    $crate::rule!{ $id => action "id" => |token, _| Some(token); None, $rx }
  };

}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_rule_macro() {
    #![allow(clippy::explicit_auto_deref)]

    let rule1: Rule<()> = rule! { 1 => "[a-z]" };
    assert_eq!(format!("{rule1:?}"), r"Rule(1 /[a-z]/ <()>::id())");

    fn f<'t>(_: &'t mut Token, _: &'_ mut u8) -> ActRet<'t> { None }
    let rule2 = rule! { 2 => action: f, "rxtest" };
    assert_eq!(format!("{rule2:?}"), "Rule(2 /rxtest/ <u8>::f())");

    const F3: ActFn<u8> = f;
    let rule3 = rule! { 3 => action: F3, group: 7, "rxtest43" };
    assert_eq!(
      format!("{rule3:?}"),
      "Rule(3 /rxtest43/ <u8>::F3() group=7)"
    );

    let rule4: Rule<()> = rule! { 4 => group: 8, "rxtest44" };
    assert_eq!(
      format!("{rule4:?}"),
      "Rule(4 /rxtest44/ <()>::id() group=8)"
    );
  }

  #[test]
  fn test_rules() {
    grammar! {
      pub rules<()> {
        ALL: [ WS: Rule{ r"[ \t]" } ],
        INIT: [
          ALPHA: Rule{ "[a-z]" }
        ],
        SECOND: [
          DIGIT: Rule{ "[0-9]" },
          DOT: Rule{ "." },
        ]
      };
    }

    let grammar = rules::create();

    use rules::*;

    assert_eq!(INIT, 1);
    assert_eq!(SECOND, 2);

    assert_eq!(WS, 0);
    assert_eq!(grammar.group(ALL)[0].id, WS);
    assert_eq!(ALPHA, 1);
    assert_eq!(grammar.group(INIT)[0].id, ALPHA);
    assert_eq!(DIGIT, 2);
    assert_eq!(grammar.group(SECOND)[0].id, DIGIT);
    assert_eq!(DOT, 3);
    assert_eq!(grammar.group(SECOND)[1].id, DOT);

    assert_eq!(
      grammar
        .rules
        .iter()
        .map(|rule| rule.rx.to_string())
        .collect::<Vec<_>>()
        .join(";"),
      "[ \\t];[a-z];[0-9];.;.+;[&&]"
    );
  }
}
