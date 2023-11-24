#![allow(dead_code, unused_imports, unused_macros)]

use crate::token::Token;
use regex::{bytes, escape};
use std::any::type_name;
use std::fmt;
use std::num::NonZeroU8;
use std::sync::LazyLock;

pub type ActionFn<S> =
  for<'t, 's> fn(&'t mut Token, &'s mut S) -> Option<&'t mut Token>;

pub type ActionId = u8;
pub type GroupId = NonZeroU8;
pub type RuleId = u16;

#[derive(Clone, Copy)]
pub struct Rule<S: 'static> {
  pub mark:        &'static str,
  pub id:          RuleId,
  pub name:        &'static str,
  pub rx:          &'static LazyLock<bytes::Regex>,
  pub action_fn:   ActionFn<S>,
  pub action:      &'static str,
  pub group:       Option<GroupId>,
  pub group_descr: &'static str,
}

impl<S> PartialEq for Rule<S> {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id && self.mark == other.mark
  }
}

impl<S> fmt::Debug for Rule<S> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Display::fmt(self, f)
  }
}

impl<S> fmt::Display for Rule<S> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mark = self.mark;
    let name = self.name;
    let rx = &self.rx.to_string();
    let empty = ("", "");
    let action = self.action;
    let (action, a) = if action == "ID" { empty } else { (action, " action=") };
    let group = self.group_descr;
    let (group, g) = if group == "_" { empty } else { (group, " group=") };

    if f.alternate() {
      write!(f, "{mark}-{name}(/{rx}/{action}{group})")
    } else {
      write!(f, "Rule({mark}-{name} /{rx}/{a}{action}{g}{group})")
    }
  }
}

pub type Name = &'static str;
pub type RuleIterator<'r, S: 'static> = impl Iterator<Item = &'r Rule<S>>;

#[derive(Clone, Copy, Debug)]
pub struct Grammar<S: 'static> {
  pub mark:   &'static str,
  pub rules:  &'static [&'static Rule<S>],
  pub groups: &'static [&'static [&'static Rule<S>]],
}

impl<S: 'static> Grammar<S> {
  pub fn ext_group(&self, group_id: GroupId) -> RuleIterator<S> {
    let group = self.groups[group_id.get() as usize];
    let all = self.groups[0];

    group.iter().chain(all.iter()).copied()
  }

  pub fn catch_all(&self) -> &Rule<S> { self.rules[self.rules.len() - 2] }

  pub fn unexpected_end(&self) -> &Rule<S> { self.rules[self.rules.len() - 1] }
}

#[macro_export]
macro_rules! static_rule {
  {
    @ $state:ty, $rx:literal, $id:ident, $action:ident,
    $group:expr, $group_descr:pat
  } => {
    {
      use std::sync::LazyLock;
      use ::regex::bytes::Regex;

      static RX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new($rx).unwrap());

      static RULE: $crate::rule::Rule<$state> = $crate::rule::Rule {
        mark: MARK,
        id: rule::$id,
        name: stringify!($id),
        rx: &RX,
        action_fn: action::$action,
        action: stringify!($action),
        group: $group,
        group_descr: stringify!($group_descr),
      };

      &RULE
    }
  };

  { $id:ident<$state:ty> =>: $rx:literal } => {
    $crate::static_rule!{ @ $state, $rx, $id, ID, None, _ }
  };

  { $id:ident<$state:ty> =>: $rx:literal group $group:ident } => {
    $crate::static_rule!{ @ $state, $rx, $id, ID, Some(group_id::$group), $group }
  };

  { $id:ident<$state:ty> =>: $rx:literal action $action:ident } => {
    $crate::static_rule!{ @ $state, $rx, $id, $action, None, _ }
  };

  {
    $id:ident<$state:ty> =>:
    $rx:literal group $group:ident action $action:ident
  } => {
    $crate::static_rule!{
      @ $state, $rx, $id, $action, Some(group_id::$group), $group
    }
  };

}

#[macro_export]
macro_rules! grammar {
  {
    $grammar:ident<$state:ty> {
      $(
        action: {
          $( $action_name:ident: $action:expr $(,)? ),*
        },
      )?
      ALL: [
        $( $all_name:ident( $( $all:tt )+ ) $(,)? ),*
      ],
      $(
        $group_name:ident: [ $(
          $rule_name:ident ( $( $rule:tt )+ ) $(,)?
        ),+ ]
      ),+
    };
  } => {
    pub mod $grammar {
      pub const RULE_COUNT: usize =
        $crate::count!(2 =>: $( $all_name, )* $( $( $rule_name, )+ )+ );

      pub const GROUP_COUNT: usize = $crate::count!(2 =>: $( $group_name, )+ );

      pub mod group {
        #![allow(non_upper_case_globals)]

        $crate::indices!(0 =>: u8: SPECIAL, ALL, $( $group_name, )+ );
      }

      pub mod group_id {
        #![allow(non_upper_case_globals)]

        $(
          pub const $group_name: $crate::rule::GroupId =
            std::num::NonZeroU8::new(super::group::$group_name).unwrap();
         )+
      }

      type State = $state;

      type Action = $crate::rule::ActionFn<State>;

      pub mod action {
        #![allow(non_upper_case_globals)]

        use super::Action;

        pub static ID: Action = |token, _| Some(token);

        $( pub static $action_name: Action = $action; )*
      }

      pub mod rule {
        #![allow(non_upper_case_globals)]

        $crate::indices!(0 =>: u16: $( $all_name, )* $( $( $rule_name, )+ )+ );

        pub const CATCH_ALL: u16 = (super::RULE_COUNT - 2) as u16;

        pub const UNEXPECTED_END: u16 = (super::RULE_COUNT - 1) as u16;
      }

      type Rule = $crate::rule::Rule<State>;

      const MARK: &'static str = concat!(
        stringify!($grammar), "<", stringify!($state), ">@",
        module_path!(), "@", file!(), ":", line!(), ":", column!(),
      );

      pub static RULES: [&Rule; RULE_COUNT] = [
        $(
          $crate::static_rule!{ $all_name<$state> =>: $( $all )+ },
        )*

        $( $(
          $crate::static_rule!{ $rule_name<$state> =>: $( $rule )+ },
        )+ )+

        $crate::static_rule!{ CATCH_ALL<$state> =>: ".+" },

        $crate::static_rule!{ UNEXPECTED_END<$state> =>: "[&&]" },
      ];

      pub const GROUP_SIZES: [usize; GROUP_COUNT] = [
        2,
        $crate::count!(0 =>: $( $all_name, )* ),
        $( $crate::count!(0 =>: $( $rule_name, )+ ), )+
      ];

      pub static GROUPS: [&'static[&'static Rule]; GROUP_COUNT] = [
        {
          static GROUP: [&Rule; GROUP_SIZES[0]] = [
            &RULES[rule::CATCH_ALL as usize],
            &RULES[rule::UNEXPECTED_END as usize]
          ];
          &GROUP
        },
        {
          static GROUP: [&Rule; GROUP_SIZES[group::ALL as usize]] = [
            $( &RULES[rule::$all_name as usize], )*
          ];
          &GROUP
        },
        $(
          {
            static GROUP: [&Rule; GROUP_SIZES[group::$group_name as usize]] = [
              $( &RULES[rule::$rule_name as usize], )*
            ];
            &GROUP
          },
        )+
      ];

      pub type Grammar = $crate::rule::Grammar<State>;

      pub static GRAMMAR: Grammar = Grammar {
        mark: MARK,
        rules: &RULES,
        groups: &GROUPS,
      };
    }
  };
}

#[macro_export]
macro_rules! indices {
  { $index:expr =>: $ty:ty: } => {};

  { $index:expr =>: $ty:ty: $head:ident, $( $tail:ident, )* } => {
    #[allow(unused)]
    pub const $head: $ty = $index as $ty;

    $crate::indices! { $index + 1 =>: $ty: $( $tail, )* }
  };
}

#[macro_export]
macro_rules! count {
  { $accum:expr =>: } => { $accum };

  { $accum:expr =>: $head:ident, $( $tail:ident, )* } => {
    $crate::count! { $accum + 1 =>: $( $tail, )* }
  };
}

#[cfg(test)]
mod test {
  pub use super::*;

  #[test]
  fn test_static_rule() {
    #![allow(clippy::explicit_auto_deref)]

    mod rule {
      use super::RuleId;

      pub const R1: RuleId = 1;
      pub const R2: RuleId = 2;
      pub const R3: RuleId = 3;
      pub const R4: RuleId = 4;
    }

    mod action {
      pub static ID: super::ActionFn<()> = |_, _| None;
      pub static F: super::ActionFn<i8> = |_, _| None;
    }

    mod group_id {
      use super::GroupId;
      use std::num::NonZeroU8;

      pub const G1: GroupId = NonZeroU8::new(7).unwrap();
      pub const G2: GroupId = NonZeroU8::new(8).unwrap();
    }
    const MARK: &str = "test";

    let rule1 = static_rule! { R1<()> =>: "[a-z]" };
    assert_eq!(format!("{rule1:?}"), r"Rule(test-R1 /[a-z]/)");

    let rule2 = static_rule! { R2<i8> =>: "rxtest" action F };
    assert_eq!(format!("{rule2:?}"), "Rule(test-R2 /rxtest/ action=F)");

    let rule3 = static_rule! { R3<()> =>: "rx" group G1 };
    assert_eq!(format!("{rule3:?}"), "Rule(test-R3 /rx/ group=G1)");

    let rule4 = static_rule! { R4<i8> =>: "." group G2 action F};
    assert_eq!(format!("{rule4:?}"), "Rule(test-R4 /./ action=F group=G2)");
  }

  #[test]
  fn test_grammar() {
    grammar! {
      defs<()> {
        ALL: [
          ws(r"[ \t]"),
        ],
        init: [
          alpha("[a-z]" group second),
        ],
        second: [
          digit("[0-9]"),
          dot(r"\." group init),
        ]
      };
    }

    use defs::group_id::*;
    use defs::rule::*;
    let indices = [ws, alpha, digit, dot, CATCH_ALL, UNEXPECTED_END];
    let names = ["ws", "alpha", "digit", "dot", "CATCH_ALL", "UNEXPECTED_END"];
    let groups = [None, Some(second), None, Some(init), None, None];
    let group_descr = ["_", "second", "_", "init", "_", "_"];
    let rx = [r"[ \t]", "[a-z]", "[0-9]", r"\.", ".+", "[&&]"];
    assert_eq!(indices.len(), defs::RULES.len());
    assert_eq!(indices.len(), names.len());
    assert_eq!(indices.len(), rx.len());
    assert_eq!(indices.len(), groups.len());
    assert_eq!(indices.len(), group_descr.len());

    let mark_start = "defs<()>@axlex::rule::test::defs@src/rule.rs:";
    for rule in defs::RULES {
      let index = rule.id as usize;
      assert_eq!(index, indices[index] as usize, "@{index}");
      assert_eq!(rule, defs::RULES[index], "@{index}");

      let mark = rule.mark;
      assert!(mark.starts_with(mark_start), "@{index} mark={mark}");

      assert_eq!(rule.name, names[index], "@{index}");
      assert_eq!(rule.rx.to_string(), rx[index], "@{index}");
      assert_eq!(rule.action, "ID", "@{index}");
      assert_eq!(rule.action_fn, defs::action::ID, "@{index}");

      assert_eq!(rule.group, groups[index], "@{index}");
      assert_eq!(rule.group_descr, group_descr[index], "@{index}");
    }

    assert_eq!(defs::group::ALL, 1);
    assert_eq!(defs::group::init, 2);
    assert_eq!(defs::group::second, 3);
    assert_eq!(defs::GROUP_SIZES, [2, 1, 1, 2]);

    assert_eq!(defs::GROUPS.len(), 4);
    assert_eq!(defs::GROUPS[0].len(), 2);
    assert_eq!(defs::GROUPS[0][0].id, CATCH_ALL);
    assert_eq!(defs::GROUPS[0][1].id, UNEXPECTED_END);
    assert_eq!(defs::GROUPS[1].len(), 1);
    assert_eq!(defs::GROUPS[1][0].id, ws);
    assert_eq!(defs::GROUPS[2].len(), 1);
    assert_eq!(defs::GROUPS[2][0].id, alpha);
    assert_eq!(defs::GROUPS[3].len(), 2);
    assert_eq!(defs::GROUPS[3][0].id, digit);
    assert_eq!(defs::GROUPS[3][1].id, dot);
  }
}
