use std::fmt;

/// Tokens a lexer has matched from a text
#[derive(Clone, Default, Eq, PartialEq)]
pub struct Token {
  /// The rule id the lexer has used to match this token
  pub rule_id: u16,

  /// The group id the lexer will switch to (except unexpected_end)
  pub group_id: u8,

  /// The token's data, usually an utf-8 substring of the tokenized text
  pub data: Vec<u8>,

  /// The index into the text for the next token
  pub index: usize,
}

fn split_point(width: usize, len: usize) -> (usize, usize) {
  ((width + 1) / 2, len - width / 2)
}

fn parts(data: &[u8], width: Option<usize>, len: usize) -> (&[u8], &[u8]) {
  let width = width.unwrap_or(0);
  let (head, tail) = split_point(width, len);
  (&data[..head], &data[tail..])
}

// Shorten to width and display (using replacement character)
fn shorten_lossy(data: &[u8], width: Option<usize>) -> String {
  let len = data.len();
  let (head, tail) = parts(data, width, len);
  let head = String::from_utf8_lossy(head);
  let tail = String::from_utf8_lossy(tail);
  let data = match width {
    Some(width) if len > width => format!("{head}⠤{tail}"),
    _ => String::from_utf8_lossy(data).to_string(),
  };
  data.chars().map(|c| if c.is_control() { '\u{fffd}' } else { c }).collect()
}

struct Short<'a>(&'a [u8]);

impl fmt::Debug for Short<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let width = f.precision();
    if f.alternate() {
      let len = self.0.len();
      let (head, tail) = parts(self.0, width, len);
      match width {
        Some(width) if len > width => write!(f, "{head:?} .. {tail:?}"),
        _ => write!(f, "{:?}", self.0),
      }
    } else {
      write!(f, "`{}`", shorten_lossy(self.0, width))
    }
  }
}

impl fmt::Debug for Token {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Token")
      .field("rule_id", &self.rule_id)
      .field("group_id", &self.group_id)
      .field("data", &Short(&self.data))
      .field("index", &self.index)
      .finish()
  }
}

impl fmt::Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let id = self.rule_id;
    let empty = "".to_string();
    let to = self.group_id;
    let to = if to == 0 { empty } else { format!(" to={to}") };
    let index = self.index;
    let data = shorten_lossy(&self.data, f.precision());

    if f.alternate() {
      write!(f, "Token(#{id} `{data}` @{index}{to})")
    } else {
      write!(f, "#{id}`{data}`@{index}{to}")
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn token(data: &[u8]) -> Token {
    let data = data.to_vec();
    Token { data, ..Token::default() }
  }

  #[test]
  fn test_token_format() {
    let tok = token(b"01234567890123456789");
    assert_eq!(format!("{tok}"), "#0`01234567890123456789`@0");
    assert_eq!(format!("{tok:#}"), "Token(#0 `01234567890123456789` @0)");
    assert_eq!(
      format!("{tok:?}"),
      "Token { rule_id: 0, group_id: 0, data: `01234567890123456789`, index: 0 }"
    );
    let tok = token(b"\0a\xa0b\xe2\x80\x8dc\xef\xbb\xbfd");
    assert_eq!(format!("{tok}"), "#0`\u{fffd}a\u{fffd}b\u{200d}c\u{feff}d`@0");
  }

  #[test]
  fn test_token_format_ext() {
    macro_rules! res {
      ($result:literal) => {
        concat!(
          "Token {\n    rule_id: 0,\n    group_id: 0,\n    data: ",
          $result,
          ",\n    index: 0,\n}",
        )
      };
    }

    #[allow(unused_variables)]
    let t = token(b"01234567");
    assert_eq!(format!("{t:#?}"), res!("[48, 49, 50, 51, 52, 53, 54, 55]"));
    assert_eq!(format!("{t:#.0?}"), res!("[] .. []"));
    assert_eq!(format!("{t:#.1?}"), res!("[48] .. []"));
    assert_eq!(format!("{t:#.2?}"), res!("[48] .. [55]"));
    assert_eq!(format!("{t:#.3?}"), res!("[48, 49] .. [55]"),);
    assert_eq!(format!("{t:#.4?}"), res!("[48, 49] .. [54, 55]"),);
    assert_eq!(format!("{t:#.7?}"), res!("[48, 49, 50, 51] .. [53, 54, 55]"),);
    assert_eq!(format!("{t:#.8?}"), res!("[48, 49, 50, 51, 52, 53, 54, 55]"),);
  }
}
