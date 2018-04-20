# Signal Handling and Graceful Shutdown

RFCS #1368 - [Signal Handling](https://github.com/rust-lang/rfcs/issues/1368)

## `graceful`

This example comes from [r/rust](https://www.reddit.com/r/rust) post [How to properly catch sigint in a threaded program?](https://www.reddit.com/r/rust/comments/6swidb/how_to_properly_catch_sigint_in_a_threaded_program/)
and [chan-sginal](https://github.com/BurntSushi/chan-signal)'s documentation, with some modifications.

## `graceful-stdin`

See the discussion in `rust-ctrlc` [Issues 30](https://github.com/Detegr/rust-ctrlc/issues/30).
