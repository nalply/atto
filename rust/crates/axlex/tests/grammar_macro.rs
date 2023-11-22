#![feature(lazy_cell)]

use axlex::grammar;

pub fn main() {
  grammar! {
    pub grammar<()> {
      ALL: [],
      INIT: [ ALPHA: Rule{ "[a-z]+" }, DOT: Rule { r"\."} ],
      SECOND: [ NUM: Rule{ "[0-9]+" } ]
    };
  }
}
