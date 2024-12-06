# Crible

Crible is 2 things:
- software bits to help one improve their [cribbage][cribbage] game;
- a pretext for the author to doodle around programming concepts.

## Usage

`cargo run card{4,6}` where card is expressed as 2 characters `Ns`:
- `N` is the card number, one of: A (or 1), 2..9, T, J, Q, K.
- `s` is the card suit, one of: h, s, c, d.

Example: `cargo run 5d 6h Ac 8d Kd Qc`

## Benchmark

Part of the fun is to make hand scoring faster than it needs to be. Run the benchmark from `crible-core`:

```
cargo bench
```

[cribbage]: https://bicyclecards.com/how-to-play/cribbage
