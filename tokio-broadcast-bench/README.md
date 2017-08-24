# tokio-broadcast-bench

## Run Benchmark

1. Run broadcasting server:
    ```bash
    cargo run --release
    ```

    Alternatively, run the server with built-in publisher based on 
    [tokio-timer](https://crates.io/crates/tokio-timer) (sending 1KB packet every 200 ms):

    ```bash
    cargo run --release -- 1000 200
    ```

    (For unknown reason, this causes high CPU usage when the interval is small.)

2. Run subscribers (4 threads, 1024 subscribers in total):
    ```bash
    cargo run --release --example subscriber -- 4 1024
    ```

3. Run publisher (a single publisher sending 10KB data packets at 100ms interval):
    ```bash
    cargo run --release --example publisher -- 10000 100
    ```
