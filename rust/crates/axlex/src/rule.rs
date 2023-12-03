use crate::token::Token;
use regex::bytes;
use std::fmt;
use std::sync::LazyLock;

pub use axlog::log;

pub type ActionFn<S> = for<'s> fn(Token, &'s mut S) -> Option<Token>;

#[derive(Clone, Copy)]
pub struct LazyRegex(pub &'static LazyLock<bytes::Regex>);

impl fmt::Debug for LazyRegex {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let regex_string = self.0.to_string();
    if f.alternate() {
      f.debug_tuple("LazyRegex").field(&regex_string).finish()
    } else {
      write!(f, "/{regex_string}/")
    }
  }
}

impl std::ops::Deref for LazyRegex {
  type Target = bytes::Regex;

  fn deref(&self) -> &Self::Target { self.0 }
}

impl PartialEq for LazyRegex {
  fn eq(&self, other: &Self) -> bool {
    self.0.to_string() == other.0.to_string()
  }
}

#[derive(Clone, Copy, Debug)]
pub struct Rule<S: 'static> {
  pub lexer_mark:    &'static str,
  pub rule_name:     &'static str,
  pub lazy_regex:    LazyRegex,
  pub action_fn:     ActionFn<S>,
  pub action_name:   &'static str,
  pub group_name:    &'static str,
  pub to_group_name: &'static str,
  pub rule_id:       u16,
  pub to_group_id:   Option<u8>,
  pub group_id:      u8,
}

impl<S> PartialEq for Rule<S> {
  fn eq(&self, other: &Self) -> bool {
    self.rule_id == other.rule_id && self.lexer_mark == other.lexer_mark
  }
}

impl<S> fmt::Display for Rule<S> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mark = self.lexer_mark;
    let short_mark =
      if let Some(at) = mark.find('@') { &mark[..at] } else { mark };
    let name = self.rule_name;
    let rx = &self.lazy_regex.to_string()[1..];
    let group = &self.group_name;
    let empty = ("", "");
    let act = self.action_name;
    let (act, a) = if act == "identity" { empty } else { (act, " action=") };
    let to = self.to_group_name;
    let (to, t) = if to == "_" { empty } else { (to, " to=") };

    if f.alternate() {
      write!(f, "Rule[{mark}]({group}.{name} /{rx}/{a}{act}{t}{to})")
    } else {
      write!(f, "{short_mark}{group}.{name}(/{rx}/{act}{t}{to})")
    }
  }
}

#[derive(Clone, Copy, Debug)]
pub struct Lexer<S: 'static> {
  pub mark:           &'static str,
  pub rules:          &'static [&'static Rule<S>],
  pub groups:         &'static [&'static [&'static Rule<S>]],
  pub catch_all:      &'static Rule<S>,
  pub unexpected_end: &'static Rule<S>,
}

pub const START_GROUP_ID: u8 = 0;

#[macro_export]
macro_rules! g_id {
  { $ident:ident } => { paste::paste!( [< G_ID_ $ident >] ) };
}

#[macro_export]
macro_rules! r_id {
  { $ident:ident } => { paste::paste!( [< R_ID_ $ident >] ) };
}

#[macro_export]
macro_rules! fn_pointer {
  { $ident:ident } => { super :: paste::paste!( [< action_ $ident >] ) };
}

#[macro_export]
macro_rules! static_rule {
  {
    @ $state:ty, $rx:expr, $id:ident, $action:ident,
    $to:expr, $to_name:pat, $group:ident
  } => {
    {
      use std::sync::LazyLock;
      use ::regex::bytes::Regex;
      use ::const_format::concatcp;

      static LAZY_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(concatcp!("^", $rx)).unwrap());

      paste::paste! {
        static RULE: $crate::rule::Rule<$state> = $crate::rule::Rule {
          lexer_mark: MARK,
          rule_id: $crate::r_id!($id),
          rule_name: stringify!($id),
          group_id: $crate::g_id!($group),
          group_name: stringify!($group),
          lazy_regex: $crate::rule::LazyRegex(&LAZY_REGEX),
          action_fn: [< action_ $action >],
          action_name: stringify!($action),
          to_group_id: $to,
          to_group_name: stringify!($to_name),
        };
      }

      &RULE
    }
  };

  { $id:ident<$state:ty> $group:ident =>: $rx:expr } => {
    $crate::static_rule!{ @ $state, $rx, $id, identity, None, _, $group }
  };

  { $id:ident<$state:ty> $group:ident =>: $rx:expr, to=$to:ident } => {
    $crate::static_rule!{
      @ $state, $rx, $id, identity, Some($crate::g_id!($to)), $to, $group
    }
  };

  { $id:ident<$state:ty> $group:ident =>: $rx:expr, action=$action:ident } => {
    $crate::static_rule!{
      @ $state, $rx, $id, $action, None, _, $group
    }
  };

  {
    $id:ident<$state:ty> $group:ident =>:
    $rx:expr, action=$action:ident, to=$to:ident
  } => {
    $crate::static_rule!{
      @ $state, $rx, $id, $action, Some($crate::g_id!($to)), $to, $group
    }
  };

}

#[macro_export]
macro_rules! lexer {
  {
    $lexer:ident<$state:ty> {
      ALL: [
        $( $all_name:ident( $( $all:tt )+ ) $(,)? ),*
      ],
      $(
        $group_name:ident: [ $(
          $rule_name:ident ( $( $rule:tt )+ ) $(,)?
        ),+ ]
      ),+ $(,)?
    } $(;)?
  } => {
    pub const RULE_COUNT: usize =
      $crate::count!(2 =>: $( $all_name, )* $( $( $rule_name, )+ )+ );

    pub const GROUP_COUNT: usize = $crate::count!(2 =>: $( $group_name, )+ );

    $crate::indices!(G_ID 0 =>: u8: $( $group_name, )+ ALL, SPECIAL, );

    #[allow(dead_code)]
    type Action = $crate::rule::ActionFn<$state>;

    pub fn action_identity(
      token: $crate::Token, _: &mut $state
    ) -> Option<$crate::Token> {
       Some(token)
    }

    pub const R_ID_CATCH_ALL: u16 = 0u16;

    pub const R_ID_UNEXPECTED_END: u16 = 1u16;

    $crate::indices!(R_ID 2 =>: u16: $( $all_name, )* $( $( $rule_name, )+ )+ );

    type Rule = $crate::rule::Rule<$state>;

    const MARK: &'static str = concat!(
      stringify!($lexer), "<", stringify!($state), ">@",
      module_path!(), "@", file!(), ":", line!(), ":", column!(),
    );

    pub static RULES: [&Rule; RULE_COUNT] = [
      $crate::static_rule!{ CATCH_ALL<$state> SPECIAL =>: ".+" },

      $crate::static_rule!{ UNEXPECTED_END<$state> SPECIAL =>: "[&&]" },

      $(
        $crate::static_rule!{ $all_name<$state> ALL =>: $( $all )+ },
      )*

      $( $(
        $crate::static_rule!{ $rule_name<$state> $group_name =>: $( $rule )+ },
      )+ )+
    ];

    pub const GROUP_SIZES: [usize; GROUP_COUNT] = [
      $( $crate::count!(0 =>: $( $rule_name, )+ ), )+
      $crate::count!(0 =>: $( $all_name, )* ),
      2,
    ];

    pub static GROUPS: [&'static[&'static Rule]; GROUP_COUNT] = [
      $(
        {
          const GROUP_ID: usize = $crate::g_id!{$group_name} as usize;
          static GROUP: [&Rule; GROUP_SIZES[GROUP_ID]] = [
            $( &RULES[$crate::r_id!($rule_name) as usize], )*
          ];
          &GROUP
        },
      )+
      {
        static GROUP: [&Rule; GROUP_SIZES[G_ID_ALL as usize]] = [
          $( &RULES[$crate::r_id!($all_name) as usize], )*
        ];
        &GROUP
      },
      {
        static GROUP: [&Rule; GROUP_SIZES[G_ID_SPECIAL as usize]] = [
          &RULES[R_ID_CATCH_ALL as usize],
          &RULES[R_ID_UNEXPECTED_END as usize]
        ];
        &GROUP
      },
    ];

    #[allow(dead_code)]
    pub type Lexer = $crate::rule::Lexer<$state>;

    #[allow(dead_code)]
    pub static LEXER: Lexer = Lexer {
      mark: MARK,
      rules: &RULES,
      groups: &GROUPS,
      catch_all: &RULES[R_ID_CATCH_ALL as usize],
      unexpected_end: &RULES[R_ID_UNEXPECTED_END as usize],
    };
  };
}

#[macro_export]
macro_rules! indices {
  { $pre:ident $index:expr =>: $ty:ty: } => {};

  { $pre:ident $index:expr =>: $ty:ty: $head:ident, $( $tail:ident, )* } => {
    paste::paste! {
      #[allow(non_upper_case_globals)]
       pub const [<$pre _ $head>]: $ty = $index as $ty;
    }

    $crate::indices! { $pre $index + 1 =>: $ty: $( $tail, )* }
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
mod tests {
  #[test]
  fn test_action() {
    use crate::Token;

    fn action_x(_: Token, _: &mut ()) -> Option<Token> { None }

    lexer! {
      action<()> {
        ALL: [],
        init: [ one(".*", action=x) ],
      }
    }
  }

  #[test]
  fn test_static_rule() {
    use crate::Token;

    {
      fn action_identity(t: Token, _: &mut ()) -> Option<Token> { Some(t) }
      const MARK: &str = "mark;";
      const R_ID_R1: u16 = 1;
      const R_ID_R3: u16 = 3;
      const G_ID_G1: u8 = 7;
      const G_ID_G2: u8 = 8;

      let rule1 = static_rule! { R1<()> G1 =>: "[a-z]" };
      assert_eq!(format!("{rule1}"), r"mark;G1.R1(/[a-z]/)");

      let rule3 = static_rule! { R3<()> G1 =>: "rx", to=G2 };
      assert_eq!(format!("{rule3}"), "mark;G1.R3(/rx/ to=G2)");
    }

    {
      fn action_f(_: Token, _: &mut i8) -> Option<Token> { None }

      const MARK: &str = "mark;";
      const R_ID_R2: u16 = 2;
      const R_ID_R4: u16 = 4;
      const G_ID_G1: u8 = 5;
      const G_ID_G2: u8 = 6;

      let rule2 = static_rule! { R2<i8> G2 =>: "rxtest", action=f };
      assert_eq!(format!("{rule2}"), "mark;G2.R2(/rxtest/f)");

      let rule4 = static_rule! { R4<i8> G2 =>: ".", action=f, to=G1 };
      assert_eq!(format!("{rule4}"), "mark;G2.R4(/./f to=G1)");
    }
  }

  #[test]
  fn test_lexer() {
    lexer! {
      defs<()> {
        ALL: [
          ws(r"[ \t]"),
        ],
        init: [
          alpha("[a-z]", to=second),
        ],
        second: [
          digit("[0-9]"),
          dot(r"\.", to=init),
        ]
      };
    }

    let rule_ids = [
      R_ID_CATCH_ALL,
      R_ID_UNEXPECTED_END,
      R_ID_ws,
      R_ID_alpha,
      R_ID_digit,
      R_ID_dot,
    ];
    let names = ["CATCH_ALL", "UNEXPECTED_END", "ws", "alpha", "digit", "dot"];
    let group_ids = [
      G_ID_SPECIAL,
      G_ID_SPECIAL,
      G_ID_ALL,
      G_ID_init,
      G_ID_second,
      G_ID_second,
    ];
    let group_names = ["SPECIAL", "SPECIAL", "ALL", "init", "second", "second"];
    let to_groups =
      [None, None, None, Some(G_ID_second), None, Some(G_ID_init)];
    let to_group_names = ["_", "_", "_", "second", "_", "init"];
    let rx = ["^.+", "^[&&]", r"^[ \t]", "^[a-z]", "^[0-9]", r"^\."];
    assert_eq!(rule_ids.len(), RULES.len());
    assert_eq!(rule_ids.len(), names.len());
    assert_eq!(rule_ids.len(), rx.len());
    assert_eq!(rule_ids.len(), group_ids.len());
    assert_eq!(rule_ids.len(), group_names.len());
    assert_eq!(rule_ids.len(), to_groups.len());
    assert_eq!(rule_ids.len(), to_group_names.len());

    for rule in RULES {
      eprintln!("checking {rule}");
      let index = rule.rule_id as usize;
      assert_eq!(index, rule_ids[index] as usize, "@{index}");
      assert_eq!(rule, RULES[index], "@{index}");

      let mark = rule.lexer_mark;
      let mark_start = "defs<()>@axlex::rule::tests@src/rule.rs";
      assert!(
        mark.starts_with(mark_start),
        "@{index} mark={mark} mark_start={mark_start}"
      );

      assert_eq!(rule.rule_name, names[index], "@{index}");
      assert_eq!(rule.lazy_regex.to_string(), rx[index], "@{index}");
      assert_eq!(rule.action_name, "identity", "@{index}");
      assert_eq!(rule.action_fn as usize, action_identity as usize, "@{index}");

      assert_eq!(rule.group_id, group_ids[index], "@{index}");
      assert_eq!(rule.group_name, group_names[index], "@{index}");
      assert_eq!(rule.to_group_id, to_groups[index], "@{index}");
      assert_eq!(rule.to_group_name, to_group_names[index], "@{index}");
    }

    const CATCH_ALL_ID: usize = R_ID_CATCH_ALL as usize;
    const UNEXPECTED_END_ID: usize = R_ID_UNEXPECTED_END as usize;
    assert_eq!(LEXER.catch_all, RULES[CATCH_ALL_ID]);
    assert_eq!(LEXER.unexpected_end, RULES[UNEXPECTED_END_ID]);

    assert_eq!(G_ID_init, 0);
    assert_eq!(G_ID_second, 1);
    assert_eq!(G_ID_ALL, 2);
    assert_eq!(G_ID_SPECIAL, 3);
    assert_eq!(GROUP_SIZES, [1, 2, 1, 2]);

    assert_eq!(GROUPS.len(), 4);
    assert_eq!(GROUPS[0].len(), 1);
    assert_eq!(GROUPS[0][0].rule_id, R_ID_alpha);
    assert_eq!(GROUPS[1].len(), 2);
    assert_eq!(GROUPS[1][0].rule_id, R_ID_digit);
    assert_eq!(GROUPS[1][1].rule_id, R_ID_dot);
    assert_eq!(GROUPS[2].len(), 1);
    assert_eq!(GROUPS[2][0].rule_id, R_ID_ws);
    assert_eq!(GROUPS[3].len(), 2);
    assert_eq!(GROUPS[3][0].rule_id, R_ID_CATCH_ALL);
    assert_eq!(GROUPS[3][1].rule_id, R_ID_UNEXPECTED_END);
  }
}
