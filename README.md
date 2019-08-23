rags
=========

[![docs.rs/rags-rs](https://docs.rs/rags-rs/badge.svg)](https://docs.rs/rags-rs)
[![travis-ci.org/zmarcantel/rags](https://api.travis-ci.org/zmarcantel/rags.svg?branch=master)](travis-ci.org/zmarcantel/rags)

`rags` is an easy to use argument parsing library for Rust that provides pretty help-printing.

For consistency, this README is kept lean. See the [documentation](https://docs.rs/rags-rs) for
up-to-date documentation. You may also look at the examples contained in this repo.

`rags` allows defining arguments in the same tree-like manner that users and developers expect.
This leads to efficient parsing as we can efficiently eliminate work based on the state of the
parsing. Once an argument has been matched it will never be inspected again.


usage
===========

This crate is available from [crates.io](https://crates.io/crates/rags-rs):

```toml
# Cargo.toml
[dependencies]
rags-rs = "^0.1.2"
```

Your application then can create a parser, define your args, and keep on going:

```rust
extern crate rags_rs as rags;

fn main() {
    let mut parser = rags::Parser::from_args();
    ...
    ...
}
```
