#![no_std]

#[macro_use]
extern crate alloc;

#[rustversion::attr(nightly, feature(assert_matches, test))]

#[rustversion::nightly]
#[cfg(test)]
extern crate test;

#[rustversion::nightly]
#[cfg(test)]
#[macro_use]
extern crate assert_matches;

extern crate wee_alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

//pub mod ast;
pub mod lexer;
