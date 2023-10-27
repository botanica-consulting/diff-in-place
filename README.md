# diff-in-place
A lightweight, Rust idiomatic trait for comparing two constant size arrays in-place and with no copies.
Each non-equal run inside the two arrays results in the callback being invoked with the starting index and the different bytes.

This is useful for embedded environments, for instance when updating the state of integrated chip over a peripheral bus such as I2c/SMBus.

This crate is suitable for usage in `no_std` targets.

# Example
```rust
use diff_in_place::DiffInPlace;

// In this scenario you want to update the state on the chip without sending all 7 bytes
let state_on_chip = [0, 0, 1, 1, 1, 1, 1];
let mut new_state = state.clone()

new_state[6] = 0;

state_on_chip.diff_in_place(new_state, |i, data| {
    // i = 6
    // data = [0,]
    peripheral.write_at(i, data);
});
```

Disclaimer: This library is not an official product, use freely at your own risk.

