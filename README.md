# spin_future

Convert synchronous functions into valid but inefficient futures.

This crate needs rust 1.46 or newer to get around an
[interesting issue](https://github.com/rust-lang/rust/issues/65489).

The key trick to make this valid is that we always call the waker if we are going to return
`Pending`. That way the executor is guaranteed to continue polling us. I got this trick from
user HadrienG in [this Rust forum post](https://users.rust-lang.org/t/polling-in-new-era-futures/30531/2).
