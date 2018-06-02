
## How `TimeoutStream` works (in `tokio_timer` v0.1)

In `Stream::poll(&mut TimeoutStream)`, the code firstly checks whether `stream` generates any new
value. If yes, it resets `sleep` and returns the value; otherwise it polls `sleep` to find out
whether it has timed out.

Note that `TimeoutStream` returns `Async::NotReady` when both `stream` and `sleep` returns
`Async::NotReady`. As we know, whenever a `Future` returns `Async::NotReady`, the corresponding
`Task` will be parked and no longer be polled by the event loop again until being notified by some
other code. In this case, where does the `Task::nofity` call take place? The answer is it is invoked
from `tokio_timer`'s own thread. Following are some details:

After construction of a new `TimeoutStream` instance, at the first time its `sleep` field is polled,
the timeout will be registered at the worker's side
([link](https://github.com/tokio-rs/tokio-timer/blob/v0.1.2/src/timer.rs#L203)):

```rust
match self.timer.worker.set_timeout(self.when, task.clone()) {
    ...
}
```

`Worker::set_timeout` pushes a pair consisting of the timeout's `Instant` and the `Task` it is
currently running on into an MPMC queue.
[`worker::run`](https://github.com/tokio-rs/tokio-timer/blob/v0.1.2/src/worker.rs#L127) runs on a
separate thread, serving three purposes:

1. It keeps polling the `Wheel` and notifies the returned `Task` when necessary.
2. It retrieves timeout registration requests from the `Worker`'s `set_timeouts` MPMC queue and
   calls `Wheel::set_timeout` method to put it onto the `Wheel`'s slot.
3. It applies modification requests from `Wheel`'s `mod_timeouts` queue (irrelevant to our
   discussion).

So it is Step 1 that notifies that parked `TimeoutStream` future.

## See also

- <https://github.com/tokio-rs/tokio-timer/issues/10>
- <https://github.com/tokio-rs/tokio-core/issues/298>
