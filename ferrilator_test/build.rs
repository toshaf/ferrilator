fn main() {
    ferrilator::build("Counter", "src/counter.rs", &["src/hdl/counter.sv"]).unwrap();
}
