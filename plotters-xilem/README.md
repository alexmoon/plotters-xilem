# Plotters View for Xilem

[![crates.io](https://img.shields.io/crates/v/plotters-xilem.svg?logo=rust)](https://crates.io/crates/plotters-xilem)
[![docs.rs badge](https://docs.rs/plotters-xilem/badge.svg)](https://docs.rs/plotters-xilem)

Use [Plotters](https://crates.io/crates/plotters) to draw plots in [Xilem](https://crates.io/crates/xilem).

## Examples

### [Simple](https://github.com/alexmoon/plotters-xilem/blob/main/plotters-xilem/examples/simple.rs)

This draws the [basic x² plot from the plotters example](https://docs.rs/plotters/0.3.1/plotters/#quick-start), filling out the entire window. The size of the plotting area changes when resizing the window.

```bash
cargo run --example simple
```

### [Interactive](https://github.com/alexmoon/plotters-xilem/blob/main/examples/interactive.rs)

In this example we use a value from the Xilem data to manipulate the plot.

```bash
cargo run --example interactive
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
