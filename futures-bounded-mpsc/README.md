# futures-bounded-mpsc

This project demonstrates a pitfall in using `futures::sync::mpsc::channel`'s API: When calling
`sender.clone().send(m)`, the returned `Sender` always has a false `maybe_parked` field, which makes
the control flow in `poll_unparked` fall outside of the "maybe parked" fast path. As a result, the
sender will never block and no backpressure takes place. This is mentioned in the doc as "The
channel capacity is equal to `buffer + num-senders`".

## Possible workaround

Just make `Clone::clone(&Sender)` return

```rust
    Sender {
        inner: self.inner.clone(),
        sender_task: self.sender_task.clone(),
        maybe_parked: true,
    };
```

This is implemented in the code as `fn custom_clone<T>(tx: &Sender<T>) -> Sender<T>`.

## Sample output

Run the original version by `cargo run`:

```
...
[CLONE] sender.state = State { is_open: true, num_messages: 7 }, maybe_parked = false
[SENT]  sender.state = State { is_open: true, num_messages: 8 }, maybe_parked = false
tx -> 15
[CLONE] sender.state = State { is_open: true, num_messages: 8 }, maybe_parked = false
[SENT]  sender.state = State { is_open: true, num_messages: 9 }, maybe_parked = true
tx -> 16
    rx <- 7

[CLONE] sender.state = State { is_open: true, num_messages: 8 }, maybe_parked = false
[SENT]  sender.state = State { is_open: true, num_messages: 9 }, maybe_parked = true
tx -> 17
[CLONE] sender.state = State { is_open: true, num_messages: 9 }, maybe_parked = false
[SENT]  sender.state = State { is_open: true, num_messages: 10 }, maybe_parked = true
tx -> 18
    rx <- 8
...
```

Run the code with custom cloning by `CUSTOM_CLONE=1 cargo run`:

```
...
[!CLONED] sender.state = State { is_open: true, num_messages: 7 }, maybe_parked = true
[SENT]    sender.state = State { is_open: true, num_messages: 8 }, maybe_parked = false
tx -> 15
[!CLONED] sender.state = State { is_open: true, num_messages: 8 }, maybe_parked = true
[SENT]    sender.state = State { is_open: true, num_messages: 9 }, maybe_parked = true
tx -> 16
    rx <- 7

[!CLONED] sender.state = State { is_open: true, num_messages: 8 }, maybe_parked = true
[SENT]    sender.state = State { is_open: true, num_messages: 9 }, maybe_parked = true
tx -> 17
[!CLONED] sender.state = State { is_open: true, num_messages: 9 }, maybe_parked = true
    rx <- 8

[SENT]    sender.state = State { is_open: true, num_messages: 9 }, maybe_parked = true
tx -> 18
[!CLONED] sender.state = State { is_open: true, num_messages: 9 }, maybe_parked = true
    rx <- 9

[SENT]    sender.state = State { is_open: true, num_messages: 9 }, maybe_parked = true
tx -> 19
[!CLONED] sender.state = State { is_open: true, num_messages: 9 }, maybe_parked = true
    rx <- 10
...
```

## Better solution

Use interior mutability for the `Sender`.
