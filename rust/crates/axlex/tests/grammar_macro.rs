#![feature(lazy_cell, const_type_name, const_option)]

use axlex::grammar;

pub fn main() {
  grammar! {
    example<()> {
      ALL: [],
      init: [ alpha("[a-z]+"), dot(r"\.") ],
      second: [ num("[0-9]+") ]
    };
  }
}
