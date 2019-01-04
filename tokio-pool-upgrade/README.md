# tokio-pool-upgrade

Migrates [`tokio-pool`](https://github.com/Connicpu/tokio-pool) to the new Tokio platform.

- - - -

From the old README:

> [Documentation](https://docs.rs/tokio-pool)
>
> TokioPool is a library designed to help you multithread your tokio applications by providing a
> pool of threads which you can distribute your load across.

- - - -

## When to use `tokio-pool`?

Actually you don't need `tokio-pool` at all. Just use Tokio's new `runtime::Runtime` (Beware, not
`runtime::current_thread::Runtime`), which is backed by an underlying work-stealing thread pool.

Furthermore, `tokio-pool` is not really a thread pool you people commonly talk about. The word
"pool" here is better understood as in "a swimming pool with lanes".
