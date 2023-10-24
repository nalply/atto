#![forbid(unsafe_code)]
#![feature(trait_alias)]

//! # Log facade and implementation
//!
//! ## Usage
//!
//! Have `axlog` but not `log` as dependency in `Cargo.toml`, then:
//!
//! ```
//! use axlog::*;
//!
//! // Init logging with spec (see `env_logger` for details)
//! init("trace");
//!
//! // Same as `log`
//! trace!("from log::trace!()");
//! debug!("from log::debug!()");
//! info!("from log::info!()");
//! warn!("from log::warn!()");
//! error!("from log::error!()");
//!
//! // Log expressions
//! let x = t!(14.0 / 3.0);
//! let x = d!("x".to_owned());
//! let x = i!(Some(5));
//! let x = w!("You have been warned");
//! let x = e!(Result::<(), &str>::Err("failure"));
//! ```
//!
//! With `log = { package = "axlog", ... }` in `Cargo.toml` you can omit
//! `use axlog::*;` but you need to prefix the macros with `log::`, like
//! this: `log::info!("some information");` or `let x = log::i!(42);`.

mod filter;
mod format;
pub mod macros;

use crate::filter::Filter;
use crate::format::format;
use parking_lot::RwLock;
use std::env;
use std::fmt::Arguments;
use std::io::{self, Write};
use std::sync::OnceLock;

pub use rust_lang_log as log;
pub use rust_lang_log::*;

// Todo comment
pub fn init<S: Into<String>>(s: S) { init_with_string(s.into()); }

static LOGGER: OnceLock<&'static AxLog> = OnceLock::new();

struct AxLog {
  filter:        RwLock<Filter>,
  itself_target: &'static str,
  // a special string to indicate that the log is from itself
}

const VAR_NAME: &str = "AXLOG";

enum ItselfLevel {
  Warn,
  Info,
}

fn itself_target() -> &'static str {
  use std::time::{SystemTime, UNIX_EPOCH};
  let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_millis()
    & 0xFFFFFFFF;

  let s = format!("axlog-{timestamp:08X}");
  // println!("{s}");
  Box::leak(s.into_boxed_str())
}

impl AxLog {
  pub fn spec(&self) -> String { self.filter.read().spec().to_owned() }

  pub fn max_level(&self) -> LevelFilter { self.filter.read().max_level() }

  pub fn set_spec(&self, spec: &str) { self.filter.write().parse(spec); }

  pub fn init(spec: &str) -> Self {
    let itself_target = itself_target();
    AxLog {
      filter: RwLock::new(Filter::new(spec)),
      itself_target,
    }
  }

  pub fn log_itself(&self, level: ItselfLevel, args: Arguments) {
    let level = match level {
      ItselfLevel::Warn => Level::Warn,
      ItselfLevel::Info => Level::Info,
    };
    self.log(
      &RecordBuilder::new()
        .level(level)
        .target(self.itself_target)
        .args(args)
        .build(),
    );
  }
}

impl Log for AxLog {
  fn enabled(&self, _metadata: &Metadata) -> bool { unimplemented!() }

  fn log(&self, record: &Record) {
    let itself = record.target() == self.itself_target;

    let filter = self.filter.read();
    // println!("filter: {filter:?}");
    let level = record.level();
    if !itself && level > filter.max_level() {
      return;
    }

    let module_path = record.module_path().unwrap_or_default();
    let file = record.file().unwrap_or_default();
    let line = record.line().unwrap_or_default();
    let target = record.target();
    let text = record.args().to_string();

    if !filter.find(level, module_path, file, target, &text) {
      return;
    }

    let mut s = String::new();
    if format(&mut s, itself, level, module_path, file, line, &text).is_err() {
      eprintln!("******* axlog FAIL-1: formatting error *******");
    }

    if let Err(err) = io::stderr().write(s.as_bytes()) {
      eprintln!("******* axlog FAIL-2: {err} *******");
    }
  }

  fn flush(&self) { io::stderr().flush().expect("flush fail"); }
}

fn init_with_string(spec: String) {
  // todo log taken from env
  // todo filter itself if level is info or higher

  if let Some(&logger) = LOGGER.get() {
    let old_spec = logger.spec();
    logger.set_spec(&spec);
    let spec = logger.spec();
    logger.log_itself(
      ItselfLevel::Warn,
      format_args!("reinitialized spec: {old_spec} -> {spec}"),
    );
    return;
  }
  // todo think about this: should a second init() override the env var?

  // init() will go here only once
  let spec = env::var(VAR_NAME).unwrap_or(spec);
  let logger = Box::leak(Box::new(AxLog::init(&spec)));
  let result = LOGGER.set(logger);
  if result.is_err() {
    let msg = "OnceLock::set() didn't return Ok(()), retrying...";
    eprintln!("******* axlog FAIL-3: {msg} *******");

    let millis = 500 + std::process::id() % 1000;
    std::thread::sleep(std::time::Duration::from_millis(millis as u64));
    init_with_string(spec);
    return;
  }

  let spec = if let Some(&logger) = LOGGER.get() {
    logger.spec()
  } else {
    let msg = "init probably failed";
    eprintln!("******* axlog FAIL-4: {msg} *******");
    return;
  };

  let max_level = logger.max_level();
  let result = set_logger(logger);
  if let Err(err) = result {
    eprintln!("******* axlog FAIL-5: {err} *******");
    return;
  }

  let level_letter = max_level.to_string().chars().next().unwrap();
  let env = if env::var(VAR_NAME).is_ok() {
    format!("environment variable {VAR_NAME}")
  } else {
    "argument passed to axlog::init()".to_owned()
  };
  logger.log_itself(
    ItselfLevel::Info,
    format_args!("started; spec: {spec} max_level: {level_letter} from {env}"),
  );
  set_max_level(max_level);
}

#[cfg(test)]
mod test {
  use crate::*;

  #[test]
  fn exercise() {
    init("trace");
    std::thread::sleep(std::time::Duration::from_millis(101));
    let _ = e!(format!("{}", 12 + 30));
    std::thread::sleep(std::time::Duration::from_millis(101));
    error!("error level");
    std::thread::sleep(std::time::Duration::from_millis(101));
    warn!("warn level");
    std::thread::sleep(std::time::Duration::from_millis(101));
    info!("info level");
    std::thread::sleep(std::time::Duration::from_millis(101));
    debug!("debug level");
    std::thread::sleep(std::time::Duration::from_millis(101));
    trace!("trace level");
    std::thread::sleep(std::time::Duration::from_millis(101));
    trace!("01234567890123456789012345678901");
    std::thread::sleep(std::time::Duration::from_millis(101));
    trace!("012345678901234567890123456789012");
    std::thread::sleep(std::time::Duration::from_millis(101));
    trace!("trailing nl\n");
    std::thread::sleep(std::time::Duration::from_millis(101));
    trace!("multiline\n1\n2\n3");
    std::thread::sleep(std::time::Duration::from_millis(101));
    trace!("multiline\nwith trailing nl\n");
  }

  // I didn't manage to test this in tests/test_log.rs because there the
  // logger is initialized differently and I want to test the initialization.
  #[test]
  fn reinit() {
    init("trace");
    init("debug");
    init("trace");
    init("trace");
    trace!("trace");
  }
}
