# bdwgc-alloc-sys

[![GitHub Action](https://img.shields.io/github/actions/workflow/status/bdwgc/bdwgc-rust/test.yaml?branch=main&style=flat-square)](https://github.com/bdwgc/bdwgc-rust/actions)
[![License](https://img.shields.io/github/license/bdwgc/bdwgc-rust.svg?style=flat-square)](https://github.com/bdwgc/bdwgc-rust/blob/main/LICENSE)

Rust bindings to the C API of [`bdwgc`][bdwgc], the conservative garbage collector.

This crate builds [`bdwgc`][bdwgc] from its vendored source and links it statically. The bindings are generated from its headers at build time by [`bindgen`](https://github.com/rust-lang/rust-bindgen), which requires `libclang`.

See the [`bdwgc-alloc`](https://crates.io/crates/bdwgc-alloc) crate for a [`GlobalAlloc`](https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html) implementation on top of these bindings.

## Install

```sh
cargo add bdwgc-alloc-sys
```

By default [`bdwgc`][bdwgc] is built with autotools. To build with cmake, enable the `cmake` feature:

```sh
cargo add bdwgc-alloc-sys --no-default-features --features cmake
```

## License

[MIT](https://github.com/bdwgc/bdwgc-rust/blob/main/LICENSE)

[bdwgc]: https://github.com/bdwgc/bdwgc
