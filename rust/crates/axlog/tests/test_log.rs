use std::sync::{Arc, Mutex};

use axlog::*;

struct Logger(Arc<Mutex<String>>);

impl Log for Logger {
  fn enabled(&self, _: &log::Metadata) -> bool { true }

  fn log(&self, record: &log::Record) {
    *self.0.lock().unwrap() = record.args().to_string();
  }

  fn flush(&self) {}
}

fn init_logger() -> impl Fn() -> String {
  let state = Arc::new(Mutex::new(String::new()));
  set_logger(Box::leak(Box::new(Logger(state.clone())))).unwrap();
  move || state.lock().unwrap().clone()
}

// captured() is an artifact of this testing harness. Usually logging writes
// to stdout and it gets difficult to test the output. Thanks to captured()
// we can test what would be output by the logger. However there's global
// state and the Rust compiler is austere in requiring the correct approach.
// There are two problems: first the global state and second the test
// functions run concurrently.
//
// So I decided to have ONE large test function instead.
#[test]
fn test_util_log() {
  let captured = init_logger();

  test_logging_harness(&captured);
  test_dg_tr(&captured);
  test_syntax_corner_cases(&captured);
  test_chained_expression(&captured);
  test_borrowed(&captured);
}

fn test_logging_harness(captured: impl Fn() -> String) {
  assert_eq!("", captured());

  set_max_level(LevelFilter::Warn);
  info!("test 1 from info!()");
  assert_eq!("", captured());

  set_max_level(LevelFilter::Info);
  info!("test 2 from info!()");
  assert_eq!("test 2 from info!()", captured());

  info!("");
  assert_eq!("", captured());
}

#[allow(clippy::eq_op)]
fn test_dg_tr(captured: impl Fn() -> String) {
  set_max_level(LevelFilter::Debug);
  assert_eq!(t!("a"), "a");
  assert_eq!("", captured()); // i. e. not enabled!

  assert_eq!(d!(40 + 2), 42);
  assert_eq!("40 + 2: 42", captured());

  assert!(d!("prefix" Option::<()>::None).is_none());
  assert_eq!("prefix Option::<()>::None: None", captured());

  // Change log level to Trace and test dg! and tr!
  set_max_level(LevelFilter::Trace);
  assert_eq!(d!(Vec::<u8>::new()).len(), 0);
  assert_eq!("Vec::<u8>::new(): []", captured());

  assert_eq!(t!(2 + 2 == 4; "additional: 4 {:?}", Some(4.0)), true);
  assert_eq!("2 + 2 == 4: true; additional: 4 Some(4.0)", captured());

  assert_eq!(t!("2D" vec![vec![1, 2], vec![]]).len(), 2);
  assert_eq!("2D vec![vec! [1, 2], vec! []]: [[1, 2], []]", captured());
}

fn test_syntax_corner_cases(captured: impl Fn() -> String) {
  assert_eq!(t!(0; "y",), 0);
  assert_eq!("0: 0; y", captured());
  assert_eq!(t!(0; "y{}", "z"), 0);
  // assert_eq!(tr!(0; "y{}", "z",), 0);
  // assert_eq!(tr!("x" 0; "y"), 0);
  // assert_eq!(tr!("x" 0; "y",), 0);
  // assert_eq!(tr!("x" 0; "y{}", "z"), 0);
  // assert_eq!(tr!("x" 0; "y{}", "z",), 0);

  // Test expression not stringify
  assert_eq!(d!(@only_value "a"), "a");
  assert_eq!(r#""a""#, captured());
  assert_eq!(d!("test" @only_value b"ab"[1]), b'b');
  assert_eq!("test 98", captured());
}

#[derive(PartialEq, Eq, Debug)]
struct TestExample;

impl TestExample {
  fn a(&self) -> Self { TestExample }

  fn b(&self) -> bool { true }
}

fn test_chained_expression(captured: impl Fn() -> String) {
  assert_eq!(d!(TestExample), TestExample);
  assert_eq!("TestExample: TestExample", captured());
  assert_eq!(d!(TestExample).a().b(), true);
  assert_eq!("TestExample: TestExample", captured());
  assert_eq!(d!(TestExample.a()).b(), true);
  assert_eq!("TestExample.a(): TestExample", captured());
  assert_eq!(d!(TestExample.a().b()), true);
  assert_eq!("TestExample.a().b(): true", captured());
}

fn test_borrowed(captured: impl Fn() -> String) {
  // Note: d!() and t!() get a problem with the borrow checker if their
  // result is not used. That's why there's a let mut x and an assign.
  // The macros evaluate the expression once, move the value and return the
  // moved value, but if you don't use the moved value, you lost the input.
  let mut x = "borrow test".to_owned();
  x = d!(x);
  assert_eq!(x, "borrow test");
  assert_eq!("x: \"borrow test\"", captured());
  assert_eq!(d!(x), "borrow test");
  assert_eq!("x: \"borrow test\"", captured());
}
