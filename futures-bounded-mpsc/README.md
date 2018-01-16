# futures-bounded-mpsc

This project demonstrates a pitfall of `futures::sync::mpsc::channel`'s API usage: When calling
`sender.clone().send(m)`, the returned `Sender` always has a false `maybe_parked` field, which makes
the control flow in `poll_unparked` fall outside of the "maybe parked" fast path. As a result, the
sender will never block and no backpressure takes place. This is mentioned in the doc as "The
channel capacity is equal to `buffer + num-senders`".

## Sample output

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
