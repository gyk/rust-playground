use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicPtr, AtomicUsize};
use std::sync::{Arc, Mutex};
use std::usize;

#[derive(Debug)]
pub struct Dummy {}


#[derive(Debug)]
pub struct Sender<T> {
    // Channel state shared between the sender and receiver.
    pub inner: Arc<Inner<T>>,

    // Handle to the task that is blocked on this sender. This handle is sent
    // to the receiver half in order to be notified when the sender becomes
    // unblocked.
    pub sender_task: Arc<Mutex<Dummy>>,

    // True if the sender might be blocked. This is an optimization to avoid
    // having to lock the mutex most of the time.
    pub maybe_parked: bool,
}

#[derive(Debug)]
pub struct Inner<T> {
    // Max buffer size of the channel. If `None` then the channel is unbounded.
    pub buffer: Option<usize>,

    // Internal channel state. Consists of the number of messages stored in the
    // channel as well as a flag signalling that the channel is closed.
    pub state: AtomicUsize,

    // Atomic, FIFO queue used to send messages to the receiver
    pub message_queue: Queue<Option<T>>,

    // Atomic, FIFO queue used to send parked task handles to the receiver.
    pub parked_queue: Queue<Arc<Mutex<Dummy>>>,

    // Number of senders in existence
    pub num_senders: AtomicUsize,

    // Handle to the receiver's task.
    pub recv_task: Mutex<Dummy>,
}

#[derive(Debug)]
pub struct Queue<T> {
    pub head: AtomicPtr<T>,
    pub tail: UnsafeCell<*mut T>,
}

#[derive(Debug, Clone, Copy)]
pub struct State {
    // `true` when the channel is open
    is_open: bool,

    // Number of messages in the channel
    num_messages: usize,
}

// The `is_open` flag is stored in the left-most bit of `Inner::state`
const OPEN_MASK: usize = usize::MAX - (usize::MAX >> 1);

// The maximum number of messages that a channel can track is `usize::MAX >> 1`
const MAX_CAPACITY: usize = !(OPEN_MASK);

pub fn decode_state(num: usize) -> State {
    State {
        is_open: num & OPEN_MASK == OPEN_MASK,
        num_messages: num & MAX_CAPACITY,
    }
}
