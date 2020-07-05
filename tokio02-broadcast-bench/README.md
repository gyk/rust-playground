# tokio02-broadcast-bench

Upgrade [tokio-broadcast-bench](../tokio-broadcast-bench) to use Tokio 0.2. In Version 0.2 Tokio
come with its own broadcast channel (`tokio::sync::broadcast`), so this toy implementation is no
longer relevant. Nevertheless, the code can still serve the purpose of benchmarking.

## Run Benchmark

1. Run broadcasting server:
    ```bash
    cargo run --release
    ```

    Alternatively, run the server with built-in publisher based on `tokio::time::Interval` (sending
    1KB packet every 200 ms):

    ```bash
    cargo run --release -- 1000 200
    ```

2. Run subscribers (4 threads, 1024 subscribers in total):
    ```bash
    cargo run --release --example subscriber -- 4 1024
    ```

3. Run publisher (a single publisher sending 10KB data packets at 100ms interval):
    ```bash
    cargo run --release --example publisher -- 10000 100
    ```

## Results

Here are some rough benchmark for the release version on a MacBook Pro 2015 (CPU: 2.7 GHz Intel Core
i5).

For the setting of 1024 clients on 4 threads with bit rate = 800 kbps, the CPU utilizations are:

| Packet size | Sending interval | CPU (v0.1) | CPU (v0.2) | CPU (v0.3) |
|:-----------:|:----------------:|:----------:|:----------:|:----------:|
|     20KB    |       200ms      |     14%    |     17%    |     15%    |
|     10KB    |       100ms      |     17%    |     21%    |     21%    |
|     5KB     |       50ms       |     28%    |     32%    |     33%    |
|     2KB     |       20ms       |     61%    |     58%    |     70%    |
|     1KB     |       10ms       |     93%    |     88%    |     95%    |

- v0.1: old Tokio (`tokio-core` 0.1)
- v0.2: new Tokio (`tokio` 0.1)
- v0.3: new Tokio (`tokio` 0.2)

For v0.1 and v0.2, the code can be found in <../tokio-broadcast-bench>. It is not fair play for v0.3
because it uses `Arc` instead of `Rc`.
