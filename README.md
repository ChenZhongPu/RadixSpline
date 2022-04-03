## RadixSpline In Rust

This is the Rust implementation for [RadixSpline: A Single-Pass Learned Index](https://github.com/learnedsystems/RadixSpline).

For simplicity, only `u64` is allowed for `key`'s type.

### Overview

In this repository, two methods are implemented respectively.

The first one is _Greedy Spline Corridor_, and its main idea is to interpolating the sorted data. Its search process has three steps given a `key`:

- A binary search to get the bound. Note that if the `key` is found now, it can return the position directly.
- A predication within an error bound.
- A second binary search in a narrower range.

The [src/spline_corridor.rs](src/spline_corridor.rs) is self-contained source code.

```rust
let spline = GreedySplineCorridor::default(&data);
if let Some(idx) = spline.search(value) {
    assert_eq!(data[idx], value);
}
```

The second one is the _RadixSpline_, and its search process has two steps:

- A predication within an error bound. Note that if the `key` is found now, it can return the position directly.
- A second binary search in a narrower range.

```rust
let radix_spline = RadixSpline::default(&data);
if let Some(idx) = radix_spline.search(value) {
    assert_eq!(data[idx], value);
}
```

### Performance

Both `GreedySplineCorridor` and `RadixSpline` are faster than a full range _binary search_, as those two conduct the searching in a much smaller range.

```bash
cargo bench 
```

There are 10 million records, and we randomly conduct the searching using three methods. The average running time is reported as following:

| Binary Search | Spline Search | SplineRadix Search |
| ------------- | ------------- | ------------------ |
| 87 ns         | 68 ns         | 50 ns              |
