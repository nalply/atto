#![feature(lazy_cell, const_type_name, const_option)]

use axlex::lexer;

#[derive(Clone, Copy)]
pub struct X(u32);

pub fn main() {
  lexer! {
    example<()> {
      ALL: [],
      init: [ alpha("[a-z]+"), dot(r"\.") ],
      second: [ num("[0-9]+") ]
    };
  }
}
