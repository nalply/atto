use crate::Level;
use regex::Regex;
use std::fmt;

// format as:
// L YYMMDD HHMMSSsss    shortloc (16) text and full-loc (50)
// example:
// T 231022 144406861     axlog:lib206 trace level at src/lib.rs:206 axlog::test
// or:
//
pub fn format<W: fmt::Write>(
  f: &mut W,
  itself: bool,
  level: Level,
  module_path: &str,
  file: &str,
  line: u32,
  text: &str,
) -> fmt::Result {
  let topic = Topic::from(level);
  let letter = letter(level);
  let letter = styled(topic, letter);
  write!(f, "{letter}")?;
  write!(f, " ")?;

  let epoch = libc_strftime::epoch();
  let format = "%g%m%d %H%M%S"; // yymmdd hhmmss
  let date_time = libc_strftime::strftime_local(format, epoch);
  let date_time = styled(topic, &date_time);
  write!(f, "{date_time}")?;

  let ms = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap_or_default() // time went backwards, so what, use 0 then
    .as_millis()
    % 1000;
  let ms = ms as u16; // no need for u128 for integers 0..1000
  let ms = format!("{ms:03}");
  let ms = styled(Plain, &ms.to_string());
  write!(f, "{ms} ")?;

  let loc = short_loc(itself, module_path, file, line);
  let loc = styled(Plain, &loc);
  write!(f, "{loc} ")?;

  let mut text = text.to_owned();
  let index = module_path.find("::").map(|i| i + 2).unwrap_or_default();
  let only_modules = &module_path[index..];
  let full_loc = format!(" at {file}:{line} {only_modules}");
  let len = text.len() + full_loc.len();
  if len > 50 && !text.ends_with('\n') {
    text.push('\n');
  }
  let cont = "\n  | ";
  let rx = Regex::new(r"( *\n)+").expect("Regex::new(some_hardcoded_regex)");
  let mut args = rx.replace_all(&text, cont).to_string();
  if args.contains("\n") {
    let ends_with_nl = args.ends_with(cont);
    args.push_str(if ends_with_nl { "" } else { cont });
    args.push_str("                               ");
  }
  let args = styled(Accent, &args.to_string());
  write!(f, "{args}")?;

  if !itself {
    write!(f, "{full_loc}")?;
  }
  writeln!(f)?;

  Ok(())
}

fn shorten(s: &str, max_len: usize) -> String {
  let len = s.chars().count();
  if len > max_len {
    let end_len = max_len / 2 - 1;
    let start = s.chars().take(max_len / 2).collect::<String>();
    let end = s
      .chars()
      .skip(len - end_len)
      .take(end_len)
      .collect::<String>();
    format!("{}-{}", start, end)
  } else {
    s.to_owned()
  }
}

// Create crat·ted∷very_·mple102 from module crate_truncated::ignored
// file src/ignored/very_long_example.rs at line 102. Logs at line > 9999 are
// reported as 9999 (you are stupid if you have more than ten thousand lines).
// Using chars() because grapheme clusters aren't relevant here.
fn short_loc(itself: bool, module_path: &str, file: &str, line: u32) -> String {
  let pad_len = 16;

  let line = std::cmp::min(9999, line);
  let line_len = if line >= 1000 {
    4
  } else if line >= 100 {
    3
  } else if line >= 10 {
    2
  } else if line >= 1 {
    1
  } else {
    0
  };

  let module_max_len = 9;
  let module = module_path.split("::").next().unwrap_or("?");
  let short_module = shorten(&module, module_max_len);
  let module_len = short_module.chars().count();

  let file_max_len = pad_len - line_len - module_len - 1;
  let file = file.strip_prefix(module).unwrap_or(file);
  let module = module.replace("_", "-");
  let file = file.strip_prefix(&module).unwrap_or(file);
  let file = file.strip_prefix("src/").unwrap_or(file);
  let file = file.strip_suffix(".rs").unwrap_or(file);
  let short_file = shorten(file, file_max_len);

  let log_record = if itself {
    "((axlog))".to_owned()
  } else {
    format!("{short_module}:{short_file}{line}")
  };

  format!("{log_record:>pad_len$}")
}

fn styled(topic: Topic, s: &str) -> String {
  let on = topic.color();
  let off = Default.color();
  format!("{on}{s}{off}")
}

use Level::{Debug as D, Error as E, Info as I, Trace as T, Warn as W};

fn letter(level: Level) -> &'static str {
  match level {
    E => "E",
    W => "W",
    I => "I",
    D => "D",
    T => "T",
  }
}

#[derive(Clone, Copy)]
enum Topic {
  ErrorLevel,
  WarnLevel,
  InfoLevel,
  DebugLevel,
  TraceLevel,
  Plain,
  Accent,
  Default,
  #[allow(dead_code)]
  Itself,
}
use Topic::*;

impl Topic {
  fn color(&self) -> &'static str {
    match self {
      ErrorLevel => "\x1b[1;91m", // bold bright red
      WarnLevel => "\x1b[1;93m",  // bold yellow
      InfoLevel => "\x1b[1;92m",  // bold bright green
      DebugLevel => "\x1b[1;96m", // bold bright cyan
      TraceLevel => "\x1b[1;37m", // bold plain
      Plain => "\x1b[0;37m",      // light gray
      Accent => "\x1b[1;97m",     // bold bright white
      Default => "\x1b[0m",
      Itself => "\x1b[1;93m", // bold yellow
    }
  }
}

impl From<Level> for Topic {
  fn from(level: Level) -> Topic {
    match level {
      E => ErrorLevel,
      W => WarnLevel,
      I => InfoLevel,
      D => DebugLevel,
      T => TraceLevel,
    }
  }
}
