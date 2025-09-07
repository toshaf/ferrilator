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
