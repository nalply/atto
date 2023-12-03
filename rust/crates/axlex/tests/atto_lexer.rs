#![feature(lazy_cell, const_type_name, const_option)]

use axlex::lexer;
use std::mem::size_of_val;

macro_rules! regex_static_strings {
  ( $( $id:ident = $( $parts:expr )+ ; )+ ) => {
    $(
      const $id: &str = const_format::concatcp!( $( $parts ),+ );
      if TRACE {
        eprintln!("{:>18} /{}/", stringify!($id), $id);
      }
    )+
  }
}

#[test]
pub fn test() {
  #![warn(clippy::invalid_regex)]
  use axlex::Token;

  const TRACE: bool = true;

  regex_static_strings! {

    // regex part strings
    SPC =                r" ";
    TAB =                r"\t";
    LF_NL =              r"\n\r";
    WS =                 SPC TAB LF_NL;
    DQU =                r#"""#;
    BSL =                r"\\";
    HASH =               r"#";
    INVALID_BARE =       r"\(\):" DQU BSL HASH;
    INVALID_CATS =       r"\p{Cc}\p{Cn}\p{Co}\pZ";
    GD_ID =              "[_0-9]{0,9}";

    // character class strings
    CC_WS =              "[" WS "]";
    CC_SPC_TAB =         "[" SPC TAB "]";
    CC_DQU_BSL =         "[" DQU BSL "]";
    CC_BSL_HASH =        "[" BSL HASH "]";
    CC_INVALID_BARE =    "[" INVALID_BARE "]";
    CC_HEX =             "[a-fA-F0-0]";
    CC_INVALID =         "[[" INVALID_CATS "]&&[^" WS "]]";
    CC_VALID_NOT_WS =    "[^" INVALID_CATS "]";
    CC_VALID =           "[" CC_VALID_NOT_WS  CC_WS "]";
    CC_VALID_SPC_TAB =   "[" CC_VALID_NOT_WS  CC_SPC_TAB "]";
    CC_BARE =            "[" CC_VALID_NOT_WS "--" CC_INVALID_BARE "]";
    CC_STRING =          "[" CC_VALID_SPC_TAB "--" CC_DQU_BSL "]";
    CC_VALID_NOT_HEX =   "[" CC_VALID "--" CC_HEX "]";
    CC_GD_STRING =       CC_STRING;

    // regex strings
    RX_WS =              CC_WS "+";
    RX_COMMENT =         "#+" CC_SPC_TAB CC_VALID_SPC_TAB "+";
    RX_BARE =            CC_BARE "+";
    RX_GD_START =        HASH GD_ID DQU;
    RX_GD_END =          DQU GD_ID HASH;
    RX_INVALID_INIT =    CC_BSL_HASH CC_VALID "{1,20}";
    RX_STRING =          CC_STRING "+";
    RX_GD_STRING =       CC_GD_STRING "+";
    RX_INVALID_STR =     DQU HASH;
    RX_ESC =             "[" DQU "enrt0" "]";
    RX_X_ESC =           "x" CC_HEX "{2}";
    RX_INVALID_X_ESC =   "x" CC_VALID_NOT_WS "{0,2}";
    RX_U_ESC =           "u" r"\{" CC_HEX "{3,5}" r"\}";
    RX_INVALID_U_ESC =   "u" CC_VALID_NOT_WS "{0,5}";
    RX_INVALID_ESC =     CC_VALID "?";
  }

  lexer! {
    large<State> {
      ALL: [ ],
      init: [
        ws(RX_WS),
        comment(RX_COMMENT),
        bare(RX_BARE),
        start_gd_string(RX_GD_START, action=save_guard, to=gd_str),
        start_string(DQU, to=str),
        colon(":"),
        open_paren(r"\("),
        close_paren(r"\)"),
        invalid_init(RX_INVALID_INIT),
      ],
      str: [
        string(RX_STRING),
        start_esc(BSL, to=esc),
        end_string(DQU, to=init),
        invalid_str(RX_INVALID_STR, to=init),
      ],
      esc: [
        simple_esc(RX_ESC, to=str),
        x_esc(RX_X_ESC, to=str),
        invalid_x_esc(RX_INVALID_X_ESC, to=str),
        u_esc(RX_U_ESC, to=str),
        invalid_u_esc(RX_INVALID_U_ESC, to=str),
        invalid_esc(RX_INVALID_ESC, to=str),
      ],
      gd_str: [
        gd_string(RX_GD_STRING),
        end_gd_string(RX_GD_END, action=check_guard, to=init),
      ],
    };
  }

  eprintln!("rule count: {}", RULES.len());
  for rule in RULES {
    eprintln!("{:>18} /{}/", rule.rule_name, &*rule.lazy_regex);
  }
  eprintln!("rule list size: {}", size_of_val(&RULES));
  let size_of_rule_names =
    RULES.iter().fold(0, |acc, rule| acc + size_of_val(rule.rule_name));
  eprintln!("size of rule names: {size_of_rule_names}");

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
}
