pub struct Example {
    dut: *mut (),
}
impl Example {
    fn new() -> Self {
        let dut = unsafe { ex_module_new() };
        Self { dut }
    }
    fn eval(&mut self) {
        unsafe { ex_module_eval(self.dut) };
    }
    fn tick(&mut self) {
        self.set_clk(true);
        self.eval();
        self.set_clk(false);
        self.eval();
    }
    fn set_clk(&mut self, value: bool) {
        unsafe { ex_module_set_clk(self.dut, value) };
    }
    fn set_a(&mut self, value: u8) {
        unsafe { ex_module_set_a(self.dut, value) };
    }
    fn get_b(&self) -> u64 {
        unsafe { ex_module_get_b(self.dut) }
    }
}
impl Drop for Example {
    fn drop(&mut self) {
        unsafe { ex_module_del(self.dut) };
    }
}
#[link(name = "Vex_module")]
extern "C" {
    fn ex_module_new() -> *mut ();
    fn ex_module_del(dut: *mut ());
    fn ex_module_eval(dut: *mut ());
    fn ex_module_set_clk(dut: *mut (), value: bool);
    fn ex_module_set_a(dut: *mut (), value: u8);
    fn ex_module_get_b(dut: *mut ()) -> u64;
}
