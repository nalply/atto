#![feature(lazy_cell, const_type_name, const_option)]

macro_rules! regex_static_strings {
  ( trace $trace:expr; $( $id:ident = $( $parts:expr )+ ; )+ ) => {
    $(
      const $id: &str = const_format::concatcp!( $( $parts ),+ );
      if $trace {
        eprintln!("{:>18} /{}/", stringify!($id), $id);
      }
    )+
  }
}

macro_rules! atto_lexer {
  (trace $trace:expr) => {
    regex_static_strings! {
      trace $trace;

      // regex parts
      SPC =                r" ";
      TAB =                r"\t";
      LF_NL =              r"\n\r";
      DQU =                r#"""#;
      BSL =                r"\\";
      HASH =               r"#";
      INVALID_BARE =       r"\(\):" DQU BSL HASH;
      INVALID_CATS =       r"\p{Cc}\p{Cn}\p{Co}\pZ";
      GD_ID =              "[_0-9]{0,9}";

      // character classes
      CC_WS =              "[" SPC TAB LF_NL "]";
      CC_SPC_TAB =         "[" SPC TAB "]";
      CC_DQU_BSL =         "[" DQU BSL "]";
      CC_BSL_HASH =        "[" BSL HASH "]";
      CC_INVALID_BARE =    "[" INVALID_BARE "]";
      CC_HEX =             "[a-fA-F0-0]";
      CC_INVALID =         "[[" INVALID_CATS "]--" CC_WS "]";
      CC_VALID_NOT_WS =    "[^" INVALID_CATS "]";
      CC_VALID =           "[" CC_VALID_NOT_WS  CC_WS "]";
      CC_VALID_SPC_TAB =   "[" CC_VALID_NOT_WS  CC_SPC_TAB "]";
      CC_BARE =            "[" CC_VALID_NOT_WS "--" CC_INVALID_BARE "]";
      CC_STRING =          "[" CC_VALID_SPC_TAB "--" CC_DQU_BSL "]";
      CC_VALID_NOT_HEX =   "[" CC_VALID "--" CC_HEX "]";
      CC_GD_STRING =       CC_STRING;

      // token regexen
      WS =                 CC_WS "+";
      COMMENT =            "#+" CC_SPC_TAB CC_VALID_SPC_TAB "+";
      BARE =               CC_BARE "+";
      GD_START =           HASH GD_ID DQU;
      GD_END =             DQU GD_ID HASH;
      INVALID_INIT =       CC_BSL_HASH CC_VALID "{1,20}";
      STRING =             CC_STRING "+";
      GD_STRING =          CC_GD_STRING "+";
      INVALID_STR =        DQU HASH;
      ESC =                "[" DQU BSL "enrt0" "]";
      X_ESC =              "x" CC_HEX "{2}";
      INVALID_X_ESC =      "x" CC_VALID_NOT_WS "{0,2}";
      U_ESC =              "u" r"\{" CC_HEX "{3,5}" r"\}";
      INVALID_U_ESC =      "u" CC_VALID_NOT_WS "{0,5}";
      INVALID_ESC =        CC_VALID "?";
    }

    axlex::lexer! {
      atto<State> {
        ALL: [ ],
        init: [
          ws(WS),
          comment(COMMENT),
          bare(BARE),
          start_gd_string(GD_START, action=save_guard, to=gd_str),
          start_string(DQU, to=str),
          colon(":"),
          open_paren(r"\("),
          close_paren(r"\)"),
          invalid_init(INVALID_INIT),
        ],
        str: [
          string(STRING),
          start_esc(BSL, to=esc),
          end_string(DQU, to=init),
          invalid_str(INVALID_STR, to=init),
        ],
        esc: [
          simple_esc(ESC, to=str),
          x_esc(X_ESC, to=str),
          invalid_x_esc(INVALID_X_ESC, to=str),
          u_esc(U_ESC, to=str),
          invalid_u_esc(INVALID_U_ESC, to=str),
          invalid_esc(INVALID_ESC, to=str),
        ],
        gd_str: [
          gd_string(GD_STRING),
          end_gd_string(GD_END, action=check_guard, to=init),
        ],
      };
    }

    if $trace {
      use std::mem::size_of_val;

      eprintln!("rule count: {}", RULES.len());
      for rule in RULES {
        eprintln!("{:>18} /{}/", rule.rule_name, &*rule.lazy_regex);
      }
      eprintln!("rule list size: {}", size_of_val(&RULES));
      let size_of_rule_names =
        RULES.iter().fold(0, |acc, rule| acc + size_of_val(rule.rule_name));
      eprintln!("size of rule names: {size_of_rule_names}");
    }

    #[derive(Clone, Copy, Debug)]
    struct State {
      guard: [u8; 9],
    }

    impl State {
      pub fn set_guard(&mut self, s: &[u8]) {
        // panics if s.len() > 9
        self.guard[..s.len()].copy_from_slice(s);
      }
    }

    fn action_save_guard(token: Token, state: &mut State) -> Option<Token> {
      state.set_guard(&token.data);
      Some(token)
    }

    fn action_check_guard(token: Token, state: &mut State) -> Option<Token> {
      if token.data == state.guard {
        Some(token)
      } else {
        None
      }
    }

    fn start(data: &[u8]) -> axlex::TokenIterator<State> {
      let state = State { guard: [0u8; 9] };
      axlex::TokenIterator::start(data, &LEXER, state)
    }
  };
}

use axlex::{rule_of, Token};

#[test]
pub fn bare() {
  const TRACE: bool = true;

  atto_lexer! { trace TRACE };

  let to_string = |token: Token| token.to_string();
  let tokens = start(b"a:x");
  let token_strings =
    tokens.clone().map(to_string).collect::<Vec<_>>().join(", ");
  let tokens = tokens.collect::<Vec<_>>();

  if TRACE {
    eprintln!("tokens: {token_strings}");
    eprintln!("first token {}", rule_of(&LEXER, tokens[0].rule_id).rule_name);
  }

  assert_eq!(tokens[0].rule_id, R_ID_bare);
  assert_eq!(tokens[0].data, b"a");
  assert_eq!(tokens[1].rule_id, R_ID_colon);
  assert_eq!(tokens[1].data, b":");
  assert_eq!(tokens[2].rule_id, R_ID_bare);
  assert_eq!(tokens[2].data, b"x");
  assert_eq!(tokens.len(), 3);
}

#[test]
pub fn empty() {
  atto_lexer! { trace false };

  assert_eq!(start(br#""#).next(), None);
}

macro_rules! test_rule {
  ($tokens:expr, $rule_id:expr, $data:expr) => {
    let rule_name = rule_of(&LEXER, $rule_id).rule_name;
    if let Some(token) = $tokens.next() {
      let rule_name_left = rule_of(&LEXER, token.rule_id).rule_name;
      assert_eq!(token.rule_id, $rule_id, "{rule_name_left} <> {rule_name}");
      assert_eq!(token.data, $data, "{rule_name_left} <> {rule_name}");
    } else {
      assert!(false, "expect token for rule {rule_name}");
    }
  };
}

#[test]
pub fn string_entry() {
  atto_lexer! { trace false };

  let mut tokens = start(br##" "key" : "value" "##);
  test_rule!(tokens, R_ID_ws, b" ");
  test_rule!(tokens, R_ID_start_string, br#"""#);
  test_rule!(tokens, R_ID_string, b"key");
  test_rule!(tokens, R_ID_end_string, br#"""#);
  test_rule!(tokens, R_ID_ws, b" ");
  test_rule!(tokens, R_ID_colon, b":");
  test_rule!(tokens, R_ID_ws, b" ");
  test_rule!(tokens, R_ID_start_string, br#"""#);
  test_rule!(tokens, R_ID_string, b"value");
  test_rule!(tokens, R_ID_end_string, br#"""#);
  test_rule!(tokens, R_ID_ws, b" ");

  assert_eq!(tokens.next(), None);
}

#[test]
pub fn string_simple_escapes() {
  atto_lexer! { trace false };

  let mut tokens = start(br##""x\e-\n.\r:\t,\0;\\'\"_""##);
  test_rule!(tokens, R_ID_start_string, br#"""#);
  test_rule!(tokens, R_ID_string, b"x");
  test_rule!(tokens, R_ID_start_esc, b"\\");
  test_rule!(tokens, R_ID_simple_esc, b"e");
  test_rule!(tokens, R_ID_string, b"-");
  test_rule!(tokens, R_ID_start_esc, b"\\");
  test_rule!(tokens, R_ID_simple_esc, b"n");
  test_rule!(tokens, R_ID_string, b".");
  test_rule!(tokens, R_ID_start_esc, b"\\");
  test_rule!(tokens, R_ID_simple_esc, b"r");
  test_rule!(tokens, R_ID_string, b":");
  test_rule!(tokens, R_ID_start_esc, b"\\");
  test_rule!(tokens, R_ID_simple_esc, b"t");
  test_rule!(tokens, R_ID_string, b",");
  test_rule!(tokens, R_ID_start_esc, b"\\");
  test_rule!(tokens, R_ID_simple_esc, b"0");
  test_rule!(tokens, R_ID_string, b";");
  test_rule!(tokens, R_ID_start_esc, b"\\");
  test_rule!(tokens, R_ID_simple_esc, b"\\");
  test_rule!(tokens, R_ID_string, b"'");
  test_rule!(tokens, R_ID_start_esc, b"\\");
  test_rule!(tokens, R_ID_simple_esc, b"\"");
  test_rule!(tokens, R_ID_string, b"_");
  test_rule!(tokens, R_ID_end_string, br#"""#);

  // todo: test x and u escapes
  // todo test invalid escapes

  assert_eq!(tokens.next(), None);
}
