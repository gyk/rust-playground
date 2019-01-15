# tokio-cleanup-queue-1

Cleans up expired files using `tokio_timer::DelayQueue`, spawned on
`tokio::runtime::current_thread`.

```bash
cargo run --example purge_file
```

See [tokio-cleanup-queue-2](../tokio-cleanup-queue-2) for a better solution.
