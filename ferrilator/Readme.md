# Ferrilator

For writing Verilator tests in Rust.

There are two main parts; binding generation and build orchestration.

Bindings for your top module can be generated like this:

```rust
use ferrilator::attr::ferrilate;

#[ferrilate(counter)]
struct Counter {
    #[clock]
    #[input]
    clk: bool,

    #[input]
    reset: bool,

    #[input]
    enable: bool,

    #[output]
    value: u8,

    #[output]
    overflow: bool,
}
```

and tests can then be written like this:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let mut dut = Counter::new();
        dut.set_enable(true);

        dut.tick();
        assert_eq!(0, dut.get_value());
        assert_eq!(false, dut.get_overflow());

        dut.tick();
        assert_eq!(1, dut.get_value());
        assert_eq!(false, dut.get_overflow());

        dut.tick();
        assert_eq!(2, dut.get_value());
        assert_eq!(false, dut.get_overflow());

        dut.set_reset(true);
        dut.tick();
        assert_eq!(0, dut.get_value());
        assert_eq!(false, dut.get_overflow());

        dut.set_reset(false);
        dut.set_enable(false);

        for _ in 0..10 {
            dut.tick();
            assert_eq!(0, dut.get_value());
            assert_eq!(false, dut.get_overflow());
        }

        dut.set_enable(true);

        for i in 0..255 {
            assert_eq!(i, dut.get_value());
            assert_eq!(false, dut.get_overflow());
            dut.tick();
        }

        dut.tick();
        assert_eq!(0, dut.get_value());
        assert_eq!(true, dut.get_overflow());

        dut.tick();
        assert_eq!(1, dut.get_value());
        assert_eq!(false, dut.get_overflow());
    }
}
```

The full Verilator build can be run from your `build.rs` like this:

```rust

fn main() {
    ferrilator::build("Counter", "src/counter.rs", &["src/hdl/counter.sv"]).unwrap();
}

```
