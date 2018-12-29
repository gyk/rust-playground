# delay-queue

A queue for executing delayed tasks. `tokio-timer`'s [`DelayQueue`][delay-queue] provides a solution
for Tokio-based applications, while the implementations here have a much simpler design, do not rely
on Tokio (run on a standalone thread per queue), and offer very basic functionality (only supporting
a global delay, and other limitations).

There are two implementations in this project:

[delay-queue]: https://docs.rs/tokio-timer/0.2.8/tokio_timer/delay_queue/struct.DelayQueue.html

- `simple_delay_queue`: As the name suggests, a simple delay queue.
- `complex_delay_queue`: A simple delay queue that is not as simple as `simple_delay_queue`.

## Run demos

```bash
for test in smoke_simple smoke_complex; do
    cargo test $test -- --nocapture
done
```

## Caveat

Of course one should use `tokio-timer`'s `DelayQueue` rather than this toy project.
