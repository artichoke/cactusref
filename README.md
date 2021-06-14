# CactusRef

[![GitHub Actions](https://github.com/artichoke/cactusref/workflows/CI/badge.svg)](https://github.com/artichoke/cactusref/actions)
[![Discord](https://img.shields.io/discord/607683947496734760)](https://discord.gg/QCe2tp2)
[![Twitter](https://img.shields.io/twitter/follow/artichokeruby?label=Follow&style=social)](https://twitter.com/artichokeruby)
<br>
[![Crate](https://img.shields.io/crates/v/cactusref.svg)](https://crates.io/crates/cactusref)
[![API](https://docs.rs/cactusref/badge.svg)](https://docs.rs/cactusref)
[![API trunk](https://img.shields.io/badge/docs-trunk-blue.svg)](https://artichoke.github.io/cactusref/cactusref/)

Single-threaded, cycle-aware, reference-counting pointers. 'Rc' stands for
'Reference Counted'.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
cactusref = "0.1"
```

## License

CactusRef is licensed with the [MIT License](LICENSE) (c) Ryan Lopopolo.

CactusRef is derived from `Rc` in the Rust standard library @
[`f586d79d`][alloc-rc-snapshot]. which is dual licensed with the [MIT
License][rust-mit-license] and [Apache 2.0 License][rust-apache2-license].

[alloc-rc-snapshot]:
  https://github.com/rust-lang/rust/blob/f586d79d183d144e0cbf519e29247f36670e2076/library/alloc/src/rc.rs
[rust-mit-license]:
  https://github.com/rust-lang/rust/blob/f586d79d183d144e0cbf519e29247f36670e2076/LICENSE-MIT
[rust-apache2-license]:
  https://github.com/rust-lang/rust/blob/f586d79d183d144e0cbf519e29247f36670e2076/LICENSE-APACHE
