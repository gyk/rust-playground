# futures-shared

Reproduces the bug reported in `futures-rs` [#952][952] "future::shared is unsafe (Send/Sync
issue)". This bug makes our server crash now and then.

The code can only be compiled with past versions of futures, so an exact version of futures is
specified in `Cargo.toml`.

[952]: https://github.com/rust-lang-nursery/futures-rs/issues/952

## Output

```plain
thread '<unnamed>' panicked at 'already borrowed: BorrowMutError', src/libcore/result.rs:916:5
note: Run with `RUST_BACKTRACE=1` for a backtrace.
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: Any', src/libcore/result.rs:916:5
```
