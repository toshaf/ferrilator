fn main() {
    ferrilator::build("Counter", "src/counter.rs", &["src/hdl/counter.sv"]).unwrap();
    ferrilator::build("Wide", "src/wide.rs", &["src/hdl/wide.sv"]).unwrap();
}
