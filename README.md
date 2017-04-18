[![Build Status](https://travis-ci.org/TedDriggs/from_variants.svg?branch=master)](https://travis-ci.org/TedDriggs/from_variants)

# Newtype Variant Conversions
Rust macro crate to automatically generate conversions from variant types into the target enum.

This crate requires Rust 1.15 or above to compile on stable.

```rust
#[macro_use]
extern crate from_variants;

#[derive(Debug, Clone, PartialEq, Eq, FromVariants)]
pub enum Lorem {
    Str(String),
    Num(u16),
}

fn main() {
    assert_eq!(Lorem::num(10), Lorem::from(10));
}
```

## Features

* **Variant opt-out**: To skip a variant, add `#[from_variants(skip)]` to a specific variant.
* **no_std support**: To generate conversions using `core::convert::From`, add `#[from_variants(no_std)]` at the struct level.