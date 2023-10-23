use crate::filter::ParseError::*;
use crate::{Level, LevelFilter};
use std::fmt;

// Rule: level=crate::module::..::module/file/text/../text
//
// Shortcuts: You can replace:
// - complete rule by level
// - level by its first letter
// - level= by nothing
// - crate by nothing
// - ::module::..::module by nothing
// - one module by :~: or ::~::
// - final module by ~
// - final modules (one or more modules separated by ::) by ~~
// - one or more modules by ::: or :::: or ::~~::
// - /file by /
// - /text by nothing
// - substrings of zero or more characters by tilde (~)
//
// Matching description:
// - for each rule if one matches print text and continue
// - if there are no rules don't match
// - default level is I (info)
// - file is the basename of the source code file without extension
// - Double tilde (~~) matches as many modules as needed
// - Slash in the text is replaced by colon
// - Tilde in the text is replaced by dash
// - Whitespaces in the text is replaced by single underscore
//
// Errors in rules:
// - more than 50 characters
// - wrong level name or wrong one-letter level code
// - module separators contain an odd number of colons (except :~:)
// - crate, modules and files don't contain only ASCII alphanumeric characters
//   and tilde and underscore
//
// Example pattern: w=~aci::tx//~aborted~
// warn log if matched aborted in text in module tx in crate ending with aci

struct Haystack<'a> {
  crate_:  &'a str,
  modules: Vec<&'a str>,
  file:    &'a str,
  text:    &'a str,
}

impl<'a> Haystack<'a> {
  fn new(
    module_path: Vec<&'a str>,
    file: &'a str,
    text: &'a str,
  ) -> Haystack<'a> {
    let _crate = module_path.get(0).unwrap_or(&"");
    let start = (module_path.len() > 0) as usize;
    let modules = module_path[start..].to_vec();

    Haystack {
      crate_: _crate,
      modules,
      file,
      text,
    }
  }

  #[cfg(test)]
  fn parse(s: &'a str) -> Haystack<'a> {
    let mid = s.find('/').unwrap_or(s.len());
    let (module_path, remainder) = s.split_at(mid);
    let module_path = module_path.split("::").collect::<Vec<&str>>();

    let remainder = remainder.strip_prefix("/").unwrap_or(remainder);
    let mid = remainder.find('/').unwrap_or(remainder.len());
    let (file, text) = remainder.split_at(mid);
    let text = text.strip_prefix("/").unwrap_or(text);

    Haystack::new(module_path, file, text)
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParseError {
  Empty,
  TooMany,
  TooLong,
  BadLevel,
  BadCrate,
  BadModule,
  BadFile,
  BadSeparator,
}

impl fmt::Display for ParseError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Empty => write!(f, "rule empty"),
      TooMany => write!(f, "too many rules"),
      TooLong => write!(f, "rule too long"),
      BadLevel => write!(f, "invalid level"),
      BadCrate => write!(f, "invalid character in crate name"),
      BadModule => write!(f, "invalid character in module name"),
      BadFile => write!(f, "invalid character in file name"),
      BadSeparator => write!(f, "invalid character in module separator"),
    }
  }
}

impl std::error::Error for ParseError {}

#[derive(Debug, PartialEq)]
struct Rule {
  pub level:   Level,
  pub crate_:  String, // can be empty => match all crates
  pub modules: Vec<String>, // can be empty => match all modules
  pub file:    String, // can be empty => match all files
  pub texts:   Vec<String>, // can be empty => match all texts
}

impl Rule {
  const MAX_LEN: usize = 50;

  #[allow(unused)] // TODO till check recognizes format capture
  pub fn unparse(&self) -> String {
    let level = self.level.as_str()[0..1].to_uppercase();
    let crate_ = &self.crate_;
    let modules = self.modules.join("::");
    let sep = if modules.is_empty() { "" } else { "::" };
    let mut rule = format!("{level}={crate_}{sep}{modules}");

    if !self.file.is_empty() {
      rule.push('/');
      rule.push_str(&self.file);
    }

    let texts = self.texts.join("/");
    if !texts.is_empty() {
      if self.file.is_empty() {
        rule.push('/')
      }
      rule.push('/');
      rule.push_str(&texts);
    }

    rule
  }

  pub fn parse(s: &str) -> Result<Rule, ParseError> {
    if s == "" {
      return Err(Empty);
    }
    if s.chars().count() > Self::MAX_LEN {
      return Err(TooLong);
    }

    // special cases: the name of the level or its first letter only
    let level = s.to_uppercase();
    let levels = ["ERROR", "WARN", "INFO", "DEBUG", "TRACE"];
    let is_level = levels.contains(&level.as_ref());
    let first = s.as_bytes().get(0).unwrap_or(&0u8);
    let is_level_letter = s.len() == 1 && b"ewidtEWIDT".contains(first);

    let mut s = s.to_owned();
    if is_level || is_level_letter {
      s.push('=')
    };

    // split off level and parse it
    let mid = s.find('=').unwrap_or_default();
    let (level, remainder) = s.split_at(mid);
    let letter = level.trim_start().chars().next().unwrap_or('I');
    let level = match letter {
      'T' | 't' => Level::Trace,
      'D' | 'd' => Level::Debug,
      'W' | 'w' => Level::Warn,
      'E' | 'e' => Level::Error,
      'I' | 'i' => Level::Info,
      _ => return Err(ParseError::BadLevel),
    };

    // split remainder by slash but strip colon first
    let mut remainder = remainder
      .strip_prefix("=")
      .unwrap_or(&remainder)
      .split("/")
      .map(str::to_owned);

    fn check(s: &str) -> bool {
      s.chars()
        .all(|c| c.is_ascii_alphanumeric() || ":_~".contains(c))
    }

    let mut modules = remainder
      .next()
      .unwrap_or_default()
      .replace("::::", "::~~::")
      .replace(":::", "::~~::")
      .replace(":~:", "::~::")
      .split("::")
      .map(|s| if check(s) { s } else { "!" })
      .map(str::to_owned)
      .collect::<Vec<String>>();
    let crate_ = modules.remove(0);
    let file = remainder.next().unwrap_or_default().clone();
    let texts = remainder.collect::<Vec<String>>();

    if crate_.contains(':') {
      return Err(BadSeparator);
    }
    if !check(&file) {
      return Err(BadCrate);
    }
    if modules.join("").contains(':') {
      return Err(BadSeparator);
    }
    if modules.join("").contains('!') {
      return Err(BadModule);
    }
    if !check(&file) || file.contains(':') {
      return Err(BadFile);
    }

    Ok(Rule {
      level,
      crate_,
      modules,
      file,
      texts,
    })
  }

  fn find(&self, haystack: &Haystack) -> bool {
    let modules = self
      .modules
      .iter()
      .map(|s| s.as_str())
      .collect::<Vec<&str>>();

    let mut result = true;
    result &= find_part_or_empty(haystack.crate_, &self.crate_);
    result &= find_parts(&haystack.modules, modules.as_slice());
    result &= find_part_or_empty(haystack.file, &self.file);

    for pattern in &self.texts {
      result &= find_part(haystack.text, pattern);
    }

    result
  }
}

#[derive(Debug, PartialEq)]
pub struct Filter {
  rules:     Vec<Rule>,
  max_level: LevelFilter,
  spec:      String,
}

impl Filter {
  const MAX_RULES: usize = 10;

  pub fn new(spec: &str) -> Filter {
    let mut filter = Filter {
      rules:     Vec::new(),
      max_level: LevelFilter::Trace, // todo
      spec:      spec.to_owned(),
    };
    filter.parse(spec);
    filter
  }

  pub fn find(
    &self,
    level: Level,
    module_path: &str,
    file: &str,
    target: &str,
    text: &str,
  ) -> bool {
    if target == crate::LOGGER.get().unwrap().itself_target {
      return true;
    }

    let module_path = module_path.split("::").collect::<Vec<&str>>();
    let haystack = Haystack::new(module_path, file, text);
    for rule in self.rules.iter() {
      if level > rule.level {
        continue;
      }

      if rule.find(&haystack) {
        return true;
      }
    }
    false
  }

  pub fn parse_result(spec: &str) -> Result<Filter, ParseError> {
    if spec.is_empty() {
      return Self::parse_result("I");
    }

    let mut rules = Vec::new();
    for rule_str in spec.split(',') {
      if rules.len() >= Self::MAX_RULES {
        return Err(TooMany);
      }
      rules.push(Rule::parse(rule_str)?);
    }

    let spec = rules
      .iter()
      .map(Rule::unparse)
      .collect::<Vec<String>>()
      .join(",");

    let max_level = LevelFilter::Trace; // todo

    Ok(Filter {
      rules,
      spec,
      max_level,
    })
  }

  pub fn parse(&mut self, spec: &str) {
    *self = Filter::parse_result(spec).unwrap_or_else(|err| {
      let msg = format!("spec parse error: {err}");
      eprintln!("******* axlog FAIL-0: {msg} *******");

      let rules = vec![];
      let spec = format!("({msg})");
      let max_level = LevelFilter::Off;
      Filter {
        rules,
        spec,
        max_level,
      }
    });
  }

  pub fn spec(&self) -> &str { &self.spec }

  pub fn max_level(&self) -> LevelFilter { self.max_level }
}

fn find_parts(parts: &[&str], patterns: &[&str]) -> bool {
  let mut wildcard = false;

  if !parts.is_empty() {
    let mut parts = parts.iter();
    for &pattern in patterns {
      if let Some(&part) = parts.next() {
        if pattern == "~~" {
          wildcard = true;
          continue;
        }

        let found = find_part(part, pattern);
        if wildcard {
          wildcard = !found
        } else if !found {
          return false;
        }
      } else {
        if !wildcard {
          return false;
        }
      }
    }
  }
  true
}

fn find_part_or_empty(part: &str, pattern: &str) -> bool {
  return pattern.is_empty() || find_part(part, pattern);
}

fn find_part(mut part: &str, mut pattern: &str) -> bool {
  if pattern == "~" {
    return true;
  }
  if pattern.is_empty() {
    return part.is_empty();
  }

  let mark_repeat = "~".repeat(pattern.len());
  if pattern == mark_repeat {
    return true;
  }

  let mut jump = pattern.starts_with('~');
  let rest_ok = pattern.ends_with('~');
  pattern = pattern.strip_prefix('~').unwrap_or(pattern);
  pattern = pattern.strip_suffix('~').unwrap_or(pattern);

  for needle in pattern.split('~') {
    if !jump && !part.starts_with(needle) {
      return false;
    }

    if let Some(part_remainder) = part.splitn(2, needle).skip(1).next() {
      part = part_remainder;
      jump = true;
    } else {
      return false;
    }
  }

  part.is_empty() || rest_ok
}

#[cfg(test)]
mod test {
  use super::*;

  // It is easy to miss the negation in assert!(!truthy);
  macro_rules! refute { ( $( $tt:tt )* ) => { assert!(!( $( $tt )* )); };  }

  #[test]
  fn test_find() {
    let haystack = Haystack::parse("::a::b/c/d");
    assert_eq!(haystack.crate_, "");
    assert_eq!(haystack.modules, ["a", "b"]);
    assert_eq!(haystack.file, "c");
    assert_eq!(haystack.text, "d");

    let rule = Rule::parse("::a::b/c/d").unwrap();
    assert!(rule.find(&haystack));
    refute!(rule.find(&Haystack::parse("")));
    refute!(rule.find(&Haystack::parse("::a::b")));
    refute!(rule.find(&Haystack::parse("a/c")));
    assert!(rule.find(&Haystack::parse("a/c/d")));

    let rule = Rule::parse("a").unwrap();
    refute!(rule.find(&haystack));
    refute!(rule.find(&Haystack::parse("")));
    assert!(rule.find(&Haystack::parse("a")));

    let rule = Rule::parse("//").unwrap();
    refute!(rule.find(&haystack));
    assert!(rule.find(&Haystack::parse("")));
    assert!(rule.find(&Haystack::parse("a")));
    assert!(rule.find(&Haystack::parse("::b")));
    assert!(rule.find(&Haystack::parse("/")));
    assert!(rule.find(&Haystack::parse("/d")));
    refute!(rule.find(&Haystack::parse("//d")));
  }

  #[test]
  fn test_parse_rule() {
    const EMPTY: [&str; 0] = [];

    let rule = Rule::parse(" trace=whatever").unwrap();
    assert_eq!(rule.level, Level::Trace);
    assert_eq!("T=whatever", rule.unparse());
    assert_eq!(rule.crate_, "whatever");
    assert_eq!(rule.modules, EMPTY);
    assert_eq!(rule.file, "");
    assert_eq!(rule.texts, EMPTY);

    let rule = Rule::parse(" D=alpha::beta::gamma").unwrap();
    assert_eq!(rule.level, Level::Debug);
    assert_eq!("D=alpha::beta::gamma", rule.unparse());
    assert_eq!(rule.crate_, "alpha");
    assert_eq!(rule.modules, ["beta", "gamma"]);

    let rule = Rule::parse("::a:::b").unwrap();
    assert_eq!(rule.level, Level::Info);
    assert_eq!("I=::a::~~::b", rule.unparse());
    assert_eq!(rule.crate_, "");
    assert_eq!(rule.modules, ["a", "~~", "b"]);

    let rule = Rule::parse("WARN=a::::b").unwrap();
    assert_eq!(rule.level, Level::Warn);
    assert_eq!("W=a::~~::b", rule.unparse());
    assert_eq!(rule.crate_, "a");
    assert_eq!(rule.modules, ["~~", "b"]);

    let rule = Rule::parse("errOR=a::~~::b").unwrap();
    assert_eq!(rule.level, Level::Error);
    assert_eq!("E=a::~~::b", rule.unparse());
    assert_eq!(rule.crate_, "a");
    assert_eq!(rule.modules, ["~~", "b"]);

    let long = "a".repeat(50);
    let too_long = "a".repeat(51);
    assert_eq!(Rule::parse(&long).unwrap().unparse(), format!("I={long}"));
    assert_eq!(Rule::parse(&too_long), Err(TooLong));
    assert_eq!(Rule::parse("x="), Err(BadLevel));
    assert_eq!(Rule::parse("=").unwrap().unparse(), "I=");
    assert_eq!(Rule::parse("i=").unwrap().unparse(), "I=");
    assert_eq!(Rule::parse("W:"), Err(BadSeparator));
    assert_eq!(Rule::parse("/:"), Err(BadFile));
    assert_eq!(Rule::parse("//:ä李").unwrap().unparse(), "I=//:ä李");
    assert_eq!(Rule::parse("E").unwrap().unparse(), "E=");
    assert_eq!(Rule::parse("W").unwrap().unparse(), "W=");
    assert_eq!(Rule::parse("i").unwrap().unparse(), "I=");
    assert_eq!(Rule::parse("I").unwrap().unparse(), "I=");
    assert_eq!(Rule::parse("D").unwrap().unparse(), "D=");
    assert_eq!(Rule::parse("T").unwrap().unparse(), "T=");
    assert_eq!(Rule::parse("Error").unwrap().unparse(), "E=");
    assert_eq!(Rule::parse("WARN").unwrap().unparse(), "W=");
    assert_eq!(Rule::parse("info").unwrap().unparse(), "I=");
    assert_eq!(Rule::parse("deBuG").unwrap().unparse(), "D=");
    assert_eq!(Rule::parse("TrAcE").unwrap().unparse(), "T=");

    let rule = Rule::parse("w=a::b/x").unwrap();
    assert_eq!(rule.crate_, "a");
    assert_eq!(rule.modules, ["b"]);
    assert_eq!(rule.file, "x");
    assert_eq!(rule.texts, EMPTY);

    let rule = Rule::parse("w=/x").unwrap();
    assert_eq!(rule.crate_, "");
    assert_eq!(rule.modules, EMPTY);
    assert_eq!(rule.file, "x");
    assert_eq!(rule.texts, EMPTY);

    let rule = Rule::parse("w=a::b//x").unwrap();
    assert_eq!(rule.crate_, "a");
    assert_eq!(rule.modules, ["b"]);
    assert_eq!(rule.file, "");
    assert_eq!(rule.texts, ["x"]);

    let rule = Rule::parse("//ä李").unwrap();
    assert_eq!(rule.crate_, "");
    assert_eq!(rule.modules, EMPTY);
    assert_eq!(rule.file, "");
    assert_eq!(rule.texts, ["ä李"]);

    let rule = Rule::parse("/x/ä李").unwrap();
    assert_eq!(rule.crate_, "");
    assert_eq!(rule.modules, EMPTY);
    assert_eq!(rule.file, "x");
    assert_eq!(rule.texts, ["ä李"]);

    let rule = Rule::parse("/").unwrap();
    assert_eq!(rule.crate_, "");
    assert_eq!(rule.modules, EMPTY);
    assert_eq!(rule.file, "");
    assert_eq!(rule.texts, EMPTY);

    let rule = Rule::parse("//").unwrap();
    assert_eq!(rule.crate_, "");
    assert_eq!(rule.modules, EMPTY);
    assert_eq!(rule.file, "");
    assert_eq!(rule.texts, [""]);
  }

  #[test]
  fn test_find_in_part() {
    assert!(find_part("", ""));
    assert!(find_part("", "~"));
    assert!(find_part("~", "~"));
    assert!(find_part("blah", "blah"));
    assert!(find_part("blah", "~"));
    assert!(find_part("blah", "~blah"));
    assert!(find_part("blah", "blah~"));
    assert!(find_part("blah", "~blah~"));

    assert!(find_part("blahblub", "blah~"));
    assert!(find_part("blahblub", "~blah~"));
    assert!(find_part("blahblub", "~blub"));
    assert!(find_part("blahblub", "~blub~"));

    assert!(find_part("blahblubblah", "blah~blah"));
    assert!(find_part("blahblubblah", "~blah~blah"));
    assert!(find_part("blahblubblah", "blah~blah~"));
    assert!(find_part("blahblubblah", "~blah~blah~"));
    assert!(find_part("blahblubblah", "~blub~"));

    assert!(find_part("axbyc", "a~b~c"));
    assert!(find_part("a012b345c678d", "a~b~c~d"));

    refute!(find_part("", "blah"));
    refute!(find_part("blah", ""));
    refute!(find_part("a", "b"));

    refute!(find_part("blahblub", "blah"));
    refute!(find_part("blahblub", "~blah"));
    refute!(find_part("blahblub", "blub"));
    refute!(find_part("blahblub", "blub~"));

    refute!(find_part("blahblubblah", "blahblah"));
    refute!(find_part("blahblubblah", "~blahblah"));
    refute!(find_part("blahblubblah", "blahblah~"));
    refute!(find_part("blahblubblah", "~blahblah~"));
    refute!(find_part("blahblubblah", "~blub"));
    refute!(find_part("blahblubblah", "blub~"));
  }
}
