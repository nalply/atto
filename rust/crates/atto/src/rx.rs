#![allow(dead_code, unused_imports, unused_macros)]

use std::fmt;
use std::sync::LazyLock;

use regex::{bytes, escape};

use crate::action::{ActFn, ActRet, Action};
use crate::token::Token;

#[derive(Clone, Eq, PartialEq)]
pub struct RuleId(*const ());

impl RuleId {
  pub fn from_usize(id: usize) -> Self { RuleId((id * 8) as *const ()) }

  fn radix(&self) -> String {
    const LOOKUP: &str =
      "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz$+-.@_";
    const BASE: u64 = LOOKUP.len() as u64;

    let mut num = self.0 as u64 / 8;
    if num == 0 {
      return "0".to_string();
    }

    let mut result = String::new();

    while num > 0 {
      let digit = (num % BASE) as usize;
      result.insert(0, LOOKUP.chars().nth(digit).unwrap());
      num /= BASE;
    }

    result
  }
}

impl fmt::Debug for RuleId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.radix())
  }
}

impl Default for RuleId {
  #[allow(clippy::zero_ptr)]
  fn default() -> Self { RuleId(0 as *const ()) }
}

#[derive(Clone)]
pub struct Rule<S> {
  pub rx:     bytes::Regex,
  pub action: Action<S>,
  pub group:  Option<usize>,
}

impl<S> Rule<S> {
  pub const fn id(&self) -> RuleId {
    RuleId(self as *const Rule<_> as *const ())
  }
}

impl<S> fmt::Debug for Rule<S> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let rx = escape(&self.rx.to_string()[1..]);
    let action = &self.action.name;
    let action_type = std::any::type_name::<S>();
    let group = if let Some(group_id) = self.group {
      format!(" group={group_id}")
    } else {
      String::new()
    };
    write!(f, "Rule(/{rx}/ {action}::<{action_type}>{group})")
  }
}

impl<S> fmt::Display for Rule<S> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let id = self.id();
    let rx = escape(&self.rx.to_string()[1..]);
    let action = &self.action.name;
    let action_type = std::any::type_name::<S>();
    let group = if let Some(group_id) = self.group {
      format!(" group={group_id}")
    } else {
      String::new()
    };
    write!(f, "Rule({id:?} /{rx}/ {action}::<{action_type}>{group})")
  }
}

impl<S> PartialEq for Rule<S> {
  fn eq(&self, other: &Self) -> bool {
    self.rx.as_str() == other.rx.as_str()
      // && self.action == other.action // todo
      && self.group == other.group
  }
}

impl<S> Eq for Rule<S> {}

type StaticStrs = &'static [&'static str];
type Group<S> = &'static [&'static LazyLock<Rule<S>>];

#[derive(Clone, Debug)]
pub struct Rules<S: 'static> {
  pub(crate) rule_names:  &'static [StaticStrs],
  pub(crate) group_names: StaticStrs,
  pub(crate) groups:      &'static [Group<S>],
  pub(crate) all:         Group<S>,
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
  {
    &'static Rules<$ty:ty> from {
      @all: [
        $( $all_rule_name:ident: rule! { $( $all_rule:tt )+ } $(,)? ),*
      ],
      $(
        $group_name:ident: [ $(
          $rule_name:ident: rule!{ $( $rule:tt )+ } $(,)?
        ),+ ]
      ),+
    }
  } => {{

    use std::sync::LazyLock;
    use $crate::rule;

    #[allow(unused_variables)]
    const GROUP_COUNT: usize = 0 $(
      + { let $group_name = (); 1 }
    )+;

    #[allow(unused_variables)]
    const ALL_COUNT: usize = 0 $(
      + { let $all_rule_name = (); 1 }
    )*;

    static GROUP_NAMES: [&str; GROUP_COUNT] = [ $(
      stringify!($group_name),
    )+ ];

    #[allow(unused_variables)]
    static RULE_NAMES: [&[&str]; GROUP_COUNT] = [ $(
      { let $group_name = (); &[ $(
        stringify!($rule_name),
      )+ ] },
    )+ ];

    #[allow(unused_variables)]
    static GROUPS: [&[&LazyLock<Rule<$ty>>]; GROUP_COUNT] = [ $(
      { let $group_name = (); &[ $( {
        static RULE: LazyLock<Rule<$ty>> = rule!{ $( $rule )+ <$ty> };
        &RULE
      }, )+ ] },
    )+ ];

    static ALL: [&LazyLock<Rule<$ty>>; ALL_COUNT] = [ $( {
      static ALL_RULE: LazyLock<Rule<$ty>> = rule!{ $( $all_rule )+ <$ty> };
      &ALL_RULE
    }, )* ];

    static RULES: Rules<$ty> = Rules {
      group_names: &GROUP_NAMES,
      rule_names: &RULE_NAMES,
      groups: &GROUPS,
      all: &ALL,
    };

    &RULES
  }};
}

#[macro_export]
macro_rules! rule {
  {
    action $name:expr => $action:expr; $group:expr, $rx:literal <$ty:ty>
  } => {
    std::sync::LazyLock::new(|| {
      use regex::bytes::Regex;
      use $crate::action::Action;

      let rule: Rule<$ty> = Rule {
        rx:     Regex::new(concat!('^', $rx)).unwrap(),
        action: Action { action_fn: $action, name: $name },
        group:  $group,
      };
      rule
    })
  };

  { id $group:expr, $rx:literal <$ty:ty> } => {
    rule!{ action "id" => |token, _| Some(token); $group, $rx<$ty> }
  };

  { group: $group:expr, $rx:literal <$ty:ty> } => {
    rule!{ id Some($group), $rx<$ty> }
  };

  { action: $action:expr, $rx:literal <$ty:ty> } => {
    rule!{ action stringify!($action) => $action; None, $rx<$ty> }
  };

  { action: $action:expr, group: $group:expr, $rx:literal <$ty:ty> } => {
    rule!{ action stringify!($action) => $action; Some($group), $rx<$ty> }
  };

  { $rx:literal <$ty:ty> } => {
    rule!{ id None, $rx<$ty> }
  };

}

static CATCH_ALL: LazyLock<Rule<()>> = rule! { ".+"<()> };

static UNEXPECTED_END: LazyLock<Rule<()>> = rule! { "[&&]"<()> };

#[cfg(test)]
mod test {
  use super::*;
  use std::ops::Deref;

  #[test]
  fn test_print_ruleid() {
    assert_eq!(format!("{:?}", RuleId::default()), "0");
    assert_eq!(format!("{:?}", RuleId::from_usize(42)), "g");
  }

  #[test]
  fn test_rule_macro() {
    let rule1 = &*rule! { "[a-z]"<()> };
    assert_eq!(format!("{rule1:?}"), r"Rule(/\[a\-z\]/ id::<()>)");

    fn f<'t>(_: &'t mut Token, _: &'_ mut u8) -> ActRet<'t> { None }
    let rule2 = &*rule! { action: f, "rxtest"<u8> };
    assert_eq!(format!("{rule2:?}"), "Rule(/rxtest/ f::<u8>)");

    const F3: ActFn<u8> = f;
    let rule3 = &*rule! { action: F3, group: 7, "rxtest43"<u8> };
    assert_eq!(format!("{rule3:?}"), "Rule(/rxtest43/ F3::<u8> group=7)");

    let rule4 = &*rule! { group: 8, "rxtest44"<()> };
    assert_eq!(format!("{rule4:?}"), "Rule(/rxtest44/ id::<()> group=8)");
  }

  #[test]
  fn test_rules() {
    let rules = rules! {
      &'static Rules<()> from {
        @all: [ ws: rule!{ r"[ \t]" } ],
        init: [
          alpha: rule!{ "[a-z]" }
        ],
        second: [
          digit: rule!{ "[0-9]" },
          dot: rule!{ "." },
        ]
      }
    };

    assert_eq!(rules.group_names.join(","), "init,second");
    assert_eq!(
      rules
        .rule_names
        .iter()
        .map(|names| names.join(","))
        .collect::<Vec<_>>()
        .join(";"),
      "alpha;digit,dot"
    );
    assert_eq!(
      rules
        .groups
        .iter()
        .map(|rules| rules
          .iter()
          .map(|rule| rule.rx.to_string())
          .collect::<Vec<_>>()
          .join(","))
        .collect::<Vec<_>>()
        .join(";"),
      "^[a-z];^[0-9],^."
    );
    assert_eq!(
      rules
        .all
        .iter()
        .map(|rule| rule.rx.to_string())
        .collect::<Vec<_>>()
        .join(","),
      r"^[ \t]"
    );
    assert_eq!(
      rules
        .group(0)
        .map(|rule| rule.rx.to_string())
        .collect::<Vec<_>>()
        .join(","),
      r"^[a-z],^[ \t]"
    );
  }
}
