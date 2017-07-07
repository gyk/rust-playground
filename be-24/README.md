## Objective

Evaluate different approaches of decoding/encoding big-endian integers in Rust, which are frequetly
used when parsing/constructing binary data.

## Run Benchmark

```bash
rustup run nightly cargo bench
```

## Results

```
running 12 tests
test tests::test_be_24 ... ignored
test tests::bench_read_u24_safe_slice             ... bench:      20,257 ns/iter (+/- 5,334)
test tests::bench_read_u24_safe_trait_once__index ... bench:      20,706 ns/iter (+/- 9,131)
test tests::bench_read_u24_safe_trait_once__rd    ... bench:      17,668 ns/iter (+/- 5,625)
test tests::bench_read_u24_safe_trait_step        ... bench:     349,629 ns/iter (+/- 89,415)
test tests::bench_read_u24_safe_trait_step_notry  ... bench:      20,330 ns/iter (+/- 6,631)
test tests::bench_read_u24_unsafe_trait           ... bench:      18,476 ns/iter (+/- 4,528)

test tests::bench_write_u24_safe_slice            ... bench:     120,087 ns/iter (+/- 56,431)
test tests::bench_write_u24_safe_trait_once       ... bench:     274,817 ns/iter (+/- 71,774)
test tests::bench_write_u24_safe_trait_step       ... bench:     928,649 ns/iter (+/- 354,580)
test tests::bench_write_u24_safe_trait_step_notry ... bench:     118,179 ns/iter (+/- 28,257)
test tests::bench_write_u24_unsafe_trait          ... bench:     275,280 ns/iter (+/- 93,090)
```

This benchmark produces mysterious results. `bench_read_u24_safe_trait_step` runs significantly
slower than others. It turns out the tests, even though marked as "ignored", have some conflict with
benchmarks (<https://github.com/rust-lang/rust/issues/25293>). Just commenting out the call to
`read_be_u24_step` in `test_be_24` makes the performances of read functions comparable.
