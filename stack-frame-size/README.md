stack-frame-size
================

In Rust, `println` macro (or more strictly, the underlying `format_args_nl` macro) is implemented as
a compiler built-in for supporting compile-time type checking and variadic arguments:

```rust
macro_rules! format_args_nl {
    ($fmt:expr) => {{ /* compiler built-in */ }};
    ($fmt:expr, $($args:tt)*) => {{ /* compiler built-in */ }};
}
```

So unlike in C/C++, when you need to inspect the size of stack frames, it is best to avoid directly
invoking `println!` to print the frame address since it may push additional variables onto the
stack. This can be demonstrated by building the following code snippets on [Rust
Playground](https://play.rust-lang.org/) or [Compiler Explorer](https://rust.godbolt.org) in Release
mode:

- The function `factorial` in

    ```rust
    #[inline(never)]
    pub fn print_ptr(x: &usize) {
        println!("{:p}", x);
    }

    #[inline(never)]
    pub fn factorial(x: usize) -> usize {
        print_ptr(&x);
        if x <= 1 {
            1
        } else {
            x * factorial(x - 1)
        }
    }
    ```

    allocates a stack frame of 32 bytes, and generates the assembly code of

    ```asm
    playground::factorial:
        push	rbx
        sub	rsp, 16
        mov	qword ptr [rsp + 8], rdi
        lea	rdi, [rsp + 8]
        call	playground::print_ptr
        mov	rbx, qword ptr [rsp + 8]
        mov	eax, 1
        cmp	rbx, 2
        jb	.LBB6_2
        lea	rdi, [rbx - 1]
        call	playground::factorial
        imul	rax, rbx

    .LBB6_2:
        add	rsp, 16
        pop	rbx
        ret
    ```

- ...while the `factorial` here

    ```rust
    #[inline(never)]
    pub fn factorial(x: usize) -> usize {
        println!("{:p}", &x);
        if x <= 1 {
            1
        } else {
            x * factorial(x - 1)
        }
    }
    ```

    allocates a stack frame of 96 bytes, and generates the assembly code of

    ```asm
    playground::factorial:
        push	rbx
        sub	rsp, 80
        mov	qword ptr [rsp], rdi
        mov	rax, rsp
        mov	qword ptr [rsp + 8], rax
        lea	rax, [rsp + 8]
        mov	qword ptr [rsp + 16], rax
        lea	rax, [rip + <&T as core::fmt::Pointer>::fmt]
        mov	qword ptr [rsp + 24], rax
        lea	rax, [rip + .L__unnamed_2]
        mov	qword ptr [rsp + 32], rax
        mov	qword ptr [rsp + 40], 2
        mov	qword ptr [rsp + 48], 0
        lea	rax, [rsp + 16]
        mov	qword ptr [rsp + 64], rax
        mov	qword ptr [rsp + 72], 1
        lea	rdi, [rsp + 32]
        call	qword ptr [rip + std::io::stdio::_print@GOTPCREL]
        mov	rbx, qword ptr [rsp]
        mov	eax, 1
        cmp	rbx, 2
        jb	.LBB5_2
        lea	rdi, [rbx - 1]
        call	playground::factorial
        imul	rax, rbx

    .LBB5_2:
        add	rsp, 80
        pop	rbx
        ret
    ```

In general, I find Rust program takes up stack space more aggressively than C/C++. It could be a
problem when the default stack size is used and your function does recurse all the way down.
