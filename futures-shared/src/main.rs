extern crate futures;

use std::cell::RefCell;
use std::mem;
use std::thread;

use futures::{future, Future};

fn main() {
    let future_currency = future::ok::<_, ()>(RefCell::new("Bitcoin".to_owned()));
    let shared_money = future_currency.shared();
    let duplicated_money1 = shared_money.clone();
    let duplicated_money2 = shared_money.clone();

    let join_cypherpunk = thread::spawn(move || {
        duplicated_money1.map(|money| {
            for _ in 1..1001 {
                thread::yield_now();
                *money.borrow_mut() = "shit".to_owned();
            }
        }).wait().expect("money");
    });

    let join_crypto_anarchism = thread::spawn(move || {
        duplicated_money2.map(|money| {
            for _ in 1..1001 {
                thread::yield_now();
                let money_taken = mem::replace(&mut *money.borrow_mut(), "nothing".to_owned());
                drop(money_taken); // unnecessary
            }
        }).wait().expect("money, too");
    });

    join_cypherpunk.join().unwrap();
    join_crypto_anarchism.join().unwrap();

    shared_money.map(|money| {
        println!("There is {} in your wallet.", money.borrow());
    }).wait().unwrap();

    // If you are extremely lucky, this program will tell you that there is shit in your Bitcoin
    // wallet (At least you've got something in your wallet, although it might not be what you
    // expect). But in the majority of cases, it just panics. So it is basically an **Undefined
    // Behavior** without using `unsafe`.
}
