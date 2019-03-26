// https://stackoverflow.com/questions/37572734/how-can-i-implement-the-observer-pattern-in-rust

use std::cell::RefCell;
use std::rc::{Rc, Weak};


struct Event(String);

// An alternative implementation is make an interface of `fn register(event_hook: Weak<RefCell<dyn
// EventHook>>)`, and change the `event_hook` field of `MyEventHandler` to `Option<Weak<RefCell<dyn
// EventHook>>>`.

/// The "Observable"
trait EventHandler {
    fn new(event_hook: Weak<RefCell<dyn EventHook>>) -> Self;
    fn handle(&mut self, event: &Event);
}

/// The "Observer"
trait EventHook {
    fn notify(&mut self, event: &Event);
}

struct MyEventHandler {
    event_hook: Weak<RefCell<dyn EventHook>>,
}

impl EventHandler for MyEventHandler {
    fn new(event_hook: Weak<RefCell<dyn EventHook>>) -> Self {
        Self {
            event_hook
        }
    }

    fn handle(&mut self, event: &Event) {
        if let Some(ref event_hook) = self.event_hook.upgrade() {
            event_hook.borrow_mut().notify(event);
        }
    }
}

struct EventManager {
    inner: Rc<RefCell<Inner>>,
    event_handler: MyEventHandler,
}

#[derive(Default)]
struct Inner {
    flag: bool,
}

impl EventHook for Inner {
    fn notify(&mut self, event: &Event) {
        println!("RECV event '{}', flag = {}", event.0, self.flag);
    }
}

impl EventManager {
    pub fn new() -> Self {
        let inner = Rc::new(RefCell::new(Inner { flag: false }));
        let weak_inner = Rc::downgrade(&inner);
        let event_handler = MyEventHandler::new(weak_inner);

        Self {
            inner,
            event_handler,
        }
    }

    pub fn handle(&mut self, event: &Event) {
        self.event_handler.handle(event);
    }

    /// Switches the internal flag state.
    pub fn switch(&mut self) {
        let flag = &mut self.inner.borrow_mut().flag;
        *flag = !*flag;
    }
}

fn main() {
    let mut mgr = EventManager::new();
    mgr.handle(&Event("The first event".to_owned()));
    mgr.switch();
    mgr.handle(&Event("The second event".to_owned()));
}
