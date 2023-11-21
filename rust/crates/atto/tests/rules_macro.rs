#![feature(lazy_cell)]

use atto::rules;

pub fn main() {
  rules! {
    static RULES: Rules<()> = {
      @all: [],
      init: [ alpha: Rule{ "[a-z]+" }, dot: Rule { r"\."} ],
      second: [ num: Rule{ "[0-9]+" } ]
    };
  }
}
