#[doc(hidden)]
#[macro_export]
macro_rules! logv {
  (
    $level:expr, $pre:literal $expr:expr; $fmt:literal, $( $args:expr ),*
  ) => {
    match $expr {
      v => {
        $crate::log!($level, "{} {}: {:?}; {}",
          $pre, stringify!($expr), &v, format!($fmt, $( $args ),* )
        );
        v
      }
    }
  };

  ($level:expr, $pre:literal @only_value $expr:expr ) => {
    match $expr {
      v => {
        $crate::log!($level, "{} {:?}", $pre, &v);
        v
      }
    }
  };

  ($level:expr, $pre:literal $expr:expr ) => {
    match $expr {
      v => {
        $crate::log!($level, "{} {}: {:?}", $pre, stringify!($expr), &v);
        v
      }
    }
  };

  ($level:expr, $expr:expr; $fmt:literal, $( $args:expr ),* ) => {
    match $expr {
      v => {
        $crate::log!($level, "{}: {:?}; {}", stringify!($expr), &v,
          format!($fmt, $( $args ),* )
        );
        v
      }
    }
  };

  ($level:expr, @only_value $expr:expr ) => {
    match $expr {
      v => {
        $crate::log!($level, "{:?}", &v);
        v
      }
    }
  };

  ($level:expr, $expr:expr ) => {
    match $expr {
      v => {
        $crate::log!($level, "{}: {:?}", stringify!($expr), &v);
        v
      }
    }
  };
}

/// A debug macro to log the expression it contains. It can be used inside
/// a more complicated expression. Optionally you can
/// - prefix some static text: add a string literal inside the macro before the
///   expression (see third example)
/// - put additional text and formatted arguments: add a semicolon after the
///   expression then the format string and arguments like in `format!()` (see
///   example 4)
/// - combine prefix and text (see example 5)
/// - omit the stringified expression (see example 6)
///
/// # Examples:
/// ```
/// #[macro_use]
/// use axlog::*;
///
/// // Example 1: log line: "40 + 2 => 42"
/// let x = d!(40 + 2);
/// assert_eq!(x, 42);
///
/// // Example 2: log line: "2 + 2 => 4"
/// let x = [d!(2 + 2); 2];
/// assert_eq!(x, [4; 2]);
///
/// // Example 3: log line: "The ultimate answer is 40 + 2 => 42"
/// let x = d!("The ultimate answer is" 40 + 2);
/// assert_eq!(x, 42);
///
/// // Example 4: log line: "40 + 2 => 42; 42 is the sum of 40 and 2"
/// let x = d!(40 + 2; "{} is a sum of {} and {}", 42, 40, 2);
/// assert_eq!(x, 42);
///
/// // Example 5: log line: "answer 40 + 2 => 42; enough Douglas Adams now"
/// // TODO (doesn't work yet)
///
/// // Example 6: log line: "4"
/// let x = d!( @only_value 2 + 2 );
/// assert_eq!(x, 4);
/// ```
#[macro_export]
macro_rules! e {
  ( $( $args:tt )+ ) => ( $crate::logv!($crate::Level::Error, $( $args )+ ) );
}

#[macro_export]
macro_rules! w {
  ( $( $args:tt )+ ) => ( $crate::logv!($crate::Level::Warn, $( $args )+ ) );
}

#[macro_export]
macro_rules! i {
  ( $( $args:tt )+ ) => ( $crate::logv!($crate::Level::Info, $( $args )+ ) );
}

#[macro_export]
macro_rules! d {
  ( $( $args:tt )+ ) => ( $crate::logv!($crate::Level::Debug, $( $args )+ ) );
}

/// A trace macro to log the expression it contains. Only enabled if the
/// trace level is active. See [`d`] for a detailed description.
#[macro_export]
macro_rules! t {
  ( $( $args:tt )+ ) => ( $crate::logv!($crate::Level::Trace, $( $args )+ ) );
}
