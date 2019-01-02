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

2. Run subscribers (4 threads, 1024 subscribers in total):
    ```bash
    cargo run --release --example subscriber -- 4 1024
    ```

3. Run publisher (a single publisher sending 10KB data packets at 100ms interval):
    ```bash
    cargo run --release --example publisher -- 10000 100
    ```

## Results

Here are some (very inaccurate) results when running the release version on a MacBook Pro (CPU: 2.7
GHz Intel Core i5).

For the setting of 1024 clients on 4 threads with bit rate = 800 kbps, the CPU utilizations are:

| Packet size | Sending interval | CPU (v0.1) | CPU (v0.2) |
|:-----------:|:----------------:|:----------:|:----------:|
|     20KB    |       200ms      |     14%    |     17%    |
|     10KB    |       100ms      |     17%    |     21%    |
|     5KB     |       50ms       |     28%    |     32%    |
|     2KB     |       20ms       |     61%    |     58%    |
|     1KB     |       10ms       |     93%    |     88%    |

- v0.1: old Tokio (`tokio-core` 0.1)
- v0.2: new Tokio (`tokio` 0.1)

## Notes on Implementation Details

[tokio-broadcast-example](https://github.com/arjsin/tokio-broadcast-example/)'s implementation is
incorrect for two reasons:

1. Broadcasting is done by iterating through a `Vec` of `Sender` in an ordered manner, but it is
   natually unordered. This is solved here by using `futures::stream::futures_unordered`.
2. The `Sender` is `clone`d each time so it disables the backpressure mechanism.

My first attempt to write the server also took the wrong `sender.clone().send(msg)` way. How do you
know this is wrong? When the broadcasting server is being overloaded, no lagging is reported by the
publisher, while the resident memory keeps growing so we know that the channel queue is filled with
more messages than expected. See `../futures-bounded-mpsc` for a detailed explanation.

The new approach takes the `Rc<RefCell<option<Sender>>>` way and it offers proper backpressure to
the sender. However, it is neither optimal. Conceptually, broadcasting would better be done via
(S|M)PMC rather than (S|M)PSC. [multiqueue](https://github.com/schets/multiqueue/) seems to be a
viable solution, but unfortunately it is based on the
unmaintained ([?](https://internals.rust-lang.org/t/crossbeam-request-for-help/4933))
[crossbeam](https://github.com/crossbeam-rs/crossbeam) v0.2.

Another thing to consider: What if some consumers are lagging behind? Certainly we hope other
consumers can keep going. In the situations where packet dropping is acceptable, we can provide
dropping mechanism on the receiver side of the channel. In this case, the channel's own backpressure
feature is not mandatory, that is, using an unbounded one is OK. Tokio's official
[chat](https://github.com/tokio-rs/tokio-core/blob/master/examples/chat.rs) example does use
`futures::sync::mpsc::unbounded` channel.

## Changelog

Version 0.2.0 upgrades to the new Tokio.
