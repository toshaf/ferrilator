use ferrilator::attr::ferrilate;

#[ferrilate(wide)]
struct Wide {
    #[input]
    a: u128,

    #[output]
    a_hi: u64,

    #[output]
    a_lo: u64,

    #[input]
    b_hi: u64,

    #[input]
    b_lo: u64,

    #[output]
    b: u128,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wide_in() {
        let mut dut = Wide::new();

        assert_eq!(dut.get_a_hi(), 0);
        assert_eq!(dut.get_a_lo(), 0);
        assert_eq!(dut.get_b(), 0);

        dut.set_a(0xffff_eeee_dddd_cccc_8888_7777_6666_5555);

        dut.eval();

        assert_eq!(
            dut.get_a_hi(),
            0xffff_eeee_dddd_cccc,
            "{:#x}",
            dut.get_a_hi()
        );
        assert_eq!(
            dut.get_a_lo(),
            0x8888_7777_6666_5555,
            "{:#x}",
            dut.get_a_lo()
        );
        assert_eq!(dut.get_b(), 0, "{:#x}", dut.get_a_lo());
    }

    #[test]
    fn test_wide_out() {
        let mut dut = Wide::new();

        assert_eq!(dut.get_a_hi(), 0);
        assert_eq!(dut.get_a_lo(), 0);
        assert_eq!(dut.get_b(), 0);

        dut.set_b_lo(0x1111_2222_3333_4444);
        dut.set_b_hi(0xaaaa_bbbb_cccc_dddd);

        dut.eval();

        assert_eq!(dut.get_a_hi(), 0, "{:#x}", dut.get_a_hi());
        assert_eq!(dut.get_a_lo(), 0, "{:#x}", dut.get_a_lo());
        assert_eq!(
            dut.get_b(),
            0xaaaa_bbbb_cccc_dddd_1111_2222_3333_4444,
            "{:#x}",
            dut.get_b()
        );
    }
}
