pub mod attr {
    pub use ferrilator_macros::ferrilate;
}

use ferrilator_core::err;
use ferrilator_core::Module;
use std::io::Read;
use std::io::Write;

/// Call from `build.rs`. Currently, the struct `name` must appear at the
/// top level of `rust_file`. Include any `verilog_files` required to build
/// the module specified in the `ferrilate` attribute applied to `name`.
/// All file paths are relative to the crate root.
pub fn build(name: &str, rust_file: &str, verilog_files: &[&str]) -> err::Result<()> {
    for fname in verilog_files {
        if !std::fs::exists(fname)? {
            return err::input(format!("file {fname} does not exist"));
        }
    }

    let item = load_struct(name, rust_file)?;
    let module_name = read_module_name(&item)?;
    let module = Module::from_struct(module_name.clone(), item)?;

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let verilated_dir = format!("{out_dir}/{module_name}_verilated");
    let binding_file = format!("{module_name}_binding.cc");
    write_binding_file(
        &module_name,
        &format!("{verilated_dir}/{binding_file}"),
        &module,
    )?;

    let result = std::process::Command::new("verilator")
        .arg("--cc")
        .arg("--build")
        .args(["--top", &module_name])
        .args(["--Mdir", &verilated_dir])
        .args(verilog_files)
        .arg(&binding_file)
        .output()
        .unwrap();

    if !result.status.success() {
        println!("verilator failed");
        println!("stdout:");
        println!("{}", std::str::from_utf8(&result.stdout).unwrap());
        println!("stderr:");
        println!("{}", std::str::from_utf8(&result.stderr).unwrap());
        panic!("cannot proceed");
    }

    println!("cargo:rustc-link-search=native={verilated_dir}");
    println!("cargo:rustc-link-lib=static=V{module_name}");
    println!("cargo:rustc-link-lib=static=verilated");
    println!("cargo:rustc-link-lib=dylib=stdc++");

    for fname in verilog_files {
        println!("cargo:rerun-if-changed={fname}");
    }

    Ok(())
}

// TODO: module path?
fn load_struct(name: &str, rust_file: &str) -> err::Result<syn::ItemStruct> {
    let mut content = String::new();
    let mut file = std::fs::File::open(rust_file)?;
    file.read_to_string(&mut content)?;
    let file = syn::parse_file(&content)?;

    for item in file.items {
        if let syn::Item::Struct(item) = item {
            if item.ident.to_string() == name {
                return Ok(item);
            }
        }
    }

    err::input(format!("failed to find struct defn for {name}"))
}

fn read_module_name(item: &syn::ItemStruct) -> err::Result<String> {
    for attr in &item.attrs {
        match &attr.meta {
            syn::Meta::List(attr) => match attr.path.segments.last() {
                Some(seg) => {
                    if seg.ident.to_string() == "ferrilate" {
                        return Ok(attr.tokens.to_string());
                    }
                }
                None => {}
            },
            syn::Meta::Path(_) => {}
            syn::Meta::NameValue(_) => {}
        }
    }
    err::input(format!(
        "struct {} has no 'ferrilate' attribute",
        item.ident.to_string()
    ))
}

fn write_binding_file(module_name: &str, fname: &str, module: &Module) -> err::Result<()> {
    if let Some(dir) = std::path::Path::new(fname).parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut file = std::fs::File::create(fname)?;
    writeln!(file, "#include <V{module_name}.h>")?;
    writeln!(file)?;
    writeln!(file, "extern \"C\" {{")?;

    writeln!(file, "V{module_name}* {module_name}_new() {{")?;
    writeln!(file, "  return new V{module_name};")?;
    writeln!(file, "}}")?;

    writeln!(file, "void {module_name}_del(V{module_name}* dut) {{")?;
    writeln!(file, "  delete dut;")?;
    writeln!(file, "}}")?;

    writeln!(file, "void {module_name}_eval(V{module_name}* dut) {{")?;
    writeln!(file, "  dut->eval();")?;
    writeln!(file, "}}")?;

    for port in module.ports() {
        let port_name = &port.name();
        let type_name = port.data_type().as_c();

        if port.input() {
            writeln!(
                file,
                "void {module_name}_set_{port_name}(V{module_name}* dut, {type_name} value) {{"
            )?;
            writeln!(file, "  dut->{port_name} = value;")?;
            writeln!(file, "}}")?;
        }

        if port.output() {
            writeln!(
                file,
                "{type_name} {module_name}_get_{port_name}(V{module_name}* dut) {{"
            )?;
            writeln!(file, "  return dut->{port_name};")?;
            writeln!(file, "}}")?;
        }
    }

    writeln!(file, "}}")?;

    Ok(())
}
