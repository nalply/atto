#![allow(dead_code, unused_imports, unused_macros)]

use std::fmt;
use std::sync::LazyLock;

use regex::{bytes, escape};

use crate::action::{ActFn, ActRet, Action};
use crate::token::Token;

/// Lexer rule with a byte regex for matching, an action for additional logic
/// with state of generic type S and optionally a group to switch to.
#[derive(Clone)]
pub struct Rule<S> {
  pub id:     usize,
  pub rx:     bytes::Regex,
  pub action: Action<S>,
  pub group:  Option<usize>,
}

impl<S> fmt::Debug for Rule<S> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let s = self.to_string();
    write!(f, "{s}")
  }
}

impl<S> fmt::Display for Rule<S> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let id = &self.id.to_string();
    let id = match self.id {
      usize::MAX => "MAX",
      _ if self.id == usize::MAX - 1 => "MAX-1",
      _ => id,
    };
    let rx = &self.rx.to_string();
    let action = &self.action.name;
    let action_type = std::any::type_name::<S>();
    let group = if let Some(group_id) = self.group {
      format!(" group={group_id}")
    } else {
      String::new()
    };
    write!(f, "Rule({id} /{rx}/ {action}::<{action_type}>{group})")
  }
}

impl<S> PartialEq for Rule<S> {
  fn eq(&self, other: &Self) -> bool { self.id == other.id }
}

impl<S> Eq for Rule<S> {}

type Group<S> = &'static [&'static LazyLock<Rule<S>>];

#[derive(Clone, Debug)]
pub struct Rules<S: 'static> {
  pub groups: &'static [Group<S>],
  pub all:    Group<S>,
}

pub trait GroupIterator<S: 'static> = Iterator<Item = &'static Rule<S>>;

pub fn unexpected_end() -> &'static Rule<()> {
  LazyLock::force(&UNEXPECTED_END)
}

pub fn catch_all() -> &'static Rule<()> { LazyLock::force(&CATCH_ALL) }

impl<S> Rules<S> {
  pub fn group(&self, group_id: usize) -> impl GroupIterator<S> {
    self.groups[group_id]
      .iter()
      .chain(self.all.iter())
      .map(|rule| LazyLock::force(rule))
  }
}

#[macro_export]
macro_rules! rules {
  { @indices $_index:expr, } => {};

  { @indices $index:expr, $head:ident, $( $tail:ident, )* } => {
    #[allow(non_upper_case_globals)]
    pub const $head: usize = $index;

    $crate::rules! { @indices $index + 1, $( $tail, )* }
  };

  { @count $accum:expr, } => { $accum };

  { @count $accum:expr, $head:ident, $( $tail:ident, )* } => {
    $crate::rules! { @count $accum + 1, $( $tail, )* }
  };

  {
    $( #[ $meta:meta ] )*
    static $rules:ident: Rules<$ty:ty> = {
      @all: [
        $( $all_rule_name:ident: Rule { $( $all_rule:tt )+ } $(,)? ),*
      ],
      $(
        $group_name:ident: [ $(
          $rule_name:ident: Rule { $( $rule:tt )+ } $(,)?
        ),+ ]
      ),+
    };
  } => {
    $crate::rules! { @indices 0, $( $group_name, )+ }

    $crate::rules! {
      @indices 0, $( $all_rule_name, )* $( $( $rule_name, )+ )+
    }

    $( #[ $meta ] )*
    static $rules: $crate::rx::Rules<$ty> = {
      use std::sync::LazyLock;
      use $crate::rule;
      use $crate::rx;

      const GROUP_COUNT: usize =
        $crate::rules! { @count 0, $( $group_name, )+ }
      ;

      const ALL_COUNT: usize =
        $crate::rules! { @count 0, $( $all_rule_name, )* }
      ;

      static GROUPS: [&[&LazyLock<rx::Rule<$ty>>]; GROUP_COUNT] = [ $(
        { let _ = stringify!($group_name); &[ $( {
          static RULE: LazyLock<rx::Rule<$ty>> = rule!{
            $rule_name => $( $rule )+
          };
          &RULE
        }, )+ ] },
      )+ ];

      static ALL: [&LazyLock<rx::Rule<$ty>>; ALL_COUNT] = [ $( {
        static ALL_RULE: LazyLock<rx::Rule<$ty>> = rule!{
        $all_rule_name => $( $all_rule )+ };
        &ALL_RULE
      }, )* ];

      rx::Rules {
        groups: &GROUPS,
        all: &ALL,
      }
    };
  };
}

#[macro_export]
macro_rules! rule {
  { $id:expr => action $name:expr => $action:expr; $group:expr, $rx:literal } => {
    std::sync::LazyLock::new(|| $crate::rx::Rule {
      id:     $id,
      rx:     ::regex::bytes::Regex::new($rx).unwrap(),
      action: $crate::action::Action { action_fn: $action, name: $name },
      group:  $group,
    })
  };

  { $id:expr => group: $group:expr, $rx:literal } => {
    rule!{ $id => action "id" => |token, _| Some(token); Some($group), $rx }
  };

  { $id:expr => action: $action:expr, $rx:literal } => {
    rule!{ $id => action stringify!($action) => $action; None, $rx }
  };

  { $id:expr => action: $action:expr, group: $group:expr, $rx:literal } => {
    rule!{ $id => action stringify!($action) => $action; Some($group), $rx }
  };

  { $id:expr => $rx:literal } => {
    rule!{ $id => action "id" => |token, _| Some(token); None, $rx }
  };

}

static CATCH_ALL: LazyLock<Rule<()>> = rule! { usize::MAX - 1 => ".+" };

static UNEXPECTED_END: LazyLock<Rule<()>> = rule! { usize::MAX => "[&&]" };

#[cfg(test)]
mod test {
  use super::*;
  use std::ops::Deref;

  #[test]
  fn test_rule_macro() {
    #![allow(clippy::explicit_auto_deref)]

    let rule1: &Rule<()> = &*rule! { 1 => "[a-z]" };
    assert_eq!(format!("{rule1:?}"), r"Rule(1 /[a-z]/ id::<()>)");

    fn f<'t>(_: &'t mut Token, _: &'_ mut u8) -> ActRet<'t> { None }
    let rule2 = &*rule! { 2 => action: f, "rxtest" };
    assert_eq!(format!("{rule2:?}"), "Rule(2 /rxtest/ f::<u8>)");

    const F3: ActFn<u8> = f;
    let rule3 = &*rule! { 3 => action: F3, group: 7, "rxtest43" };
    assert_eq!(format!("{rule3:?}"), "Rule(3 /rxtest43/ F3::<u8> group=7)");

    let rule4: &Rule<()> = &*rule! { 4 => group: 8, "rxtest44" };
    assert_eq!(format!("{rule4:?}"), "Rule(4 /rxtest44/ id::<()> group=8)");
  }

  #[test]
  fn test_rules() {
    rules! {
      #[allow(non_upper_case_globals)]
      static rules: Rules<()> = {
        @all: [ ws: Rule{ r"[ \t]" } ],
        init: [
          alpha: Rule{ "[a-z]" }
        ],
        second: [
          digit: Rule{ "[0-9]" },
          dot: Rule{ "." },
        ]
      };
    }

    assert_eq!(init, 0);
    assert_eq!(second, 1);

    assert_eq!(ws, 0);
    assert_eq!(rules.all[0].id, ws);
    assert_eq!(alpha, 1);
    assert_eq!(rules.groups[init][0].id, alpha);
    assert_eq!(digit, 2);
    assert_eq!(rules.groups[second][0].id, digit);
    assert_eq!(dot, 3);
    assert_eq!(rules.groups[second][1].id, dot);

    assert_eq!(
      rules
        .groups
        .iter()
        .map(|rs| rs
          .iter()
          .map(|rule| rule.rx.to_string())
          .collect::<Vec<_>>()
          .join(","))
        .collect::<Vec<_>>()
        .join(";"),
      "[a-z];[0-9],."
    );
    assert_eq!(
      rules
        .all
        .iter()
        .map(|rule| rule.rx.to_string())
        .collect::<Vec<_>>()
        .join(","),
      r"[ \t]"
    );
    assert_eq!(
      rules
        .group(init)
        .map(|rule| rule.rx.to_string())
        .collect::<Vec<_>>()
        .join(","),
      r"[a-z],[ \t]"
    );
  }
}
