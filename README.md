# bytes-lit
Creates byte arrays from literal values.

Currently supports only integer literals of unbounded size.

## Example

```rust
let bytes = bytes!(0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d);
assert_eq!(bytes, [
    253, 237, 63, 85, 222, 196, 114, 80, 165, 42, 140, 11, 183, 3, 142, 114,
    250, 111, 250, 174, 51, 86, 47, 119, 205, 43, 98, 158, 247, 253, 66, 77,
]);
```
