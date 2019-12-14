//! # Unboxed Closures and FFI Callbacks
//!
//! Ref: <https://aatch.github.io/blog/2015/01/17/unboxed-closures-and-ffi-callbacks/>.
//!

use std::ffi::*;
use std::os::raw::c_int;

pub mod ffi {
    use super::*;
    extern "C" {
        // Declare the prototype for the external function
        pub fn do_thing(cb: extern "C" fn(*mut c_void, c_int) -> c_int, context: *mut c_void);
    }
}

// Exposed function to the user of the bindings
pub fn do_thing<F>(f: &F)
where
    F: FnMut(i32) -> i32,
{
    let context = f as *const _ as *mut c_void;
    unsafe {
        ffi::do_thing(do_thing_wrapper::<F>, context);
    }

    // Shim interface function
    extern "C" fn do_thing_wrapper<F>(closure: *mut c_void, x: c_int) -> c_int
    where
        F: FnMut(i32) -> i32,
    {
        // Due to Rust's null pointer optimization, it's also a `*mut Option<F>`.
        let closure = closure as *mut F;
        unsafe {
            let res = (*closure)(x as i32);
            return res as c_int;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Context {
        state: i32,
    }

    #[test]
    fn smoke() {
        let mut ctx = Context { state: 42 };

        // `callback` is an "unboxed" closure (in the early days of Rust, closures are boxed, see
        // https://www.reddit.com/r/rust/comments/2lo6yt/closures_vs_unboxed_closures/clwlrfa/),
        // which is effectively an instance of an anonymous struct that contains the captured
        // variables (`ctx` in this case).
        let callback = |x: i32| -> i32 {
            let result = ctx.state + x;
            println!("context = {:?}, result = {}", ctx, result);
            ctx.state += 1;
            result
        };

        do_thing(&callback);
        do_thing(&callback);
        do_thing(&callback);
    }
}
