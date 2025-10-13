pub mod attr {
    pub use ferrilator_macros::ferrilate;
}

use ferrilator_core::Module;
use ferrilator_core::err;
use std::io::Read;
use std::io::Write;

/// Call from `build.rs`. Currently, the struct `name` must appear at the
/// top level of `rust_file`. Include any `verilog_files` required to build
/// the module specified in the `ferrilate` attribute applied to `name`.
/// All file paths are relative to the crate root.
/// Verilator is assumed to be installed at `/usr/share/verilator` but this
/// can be overriden by setting VERILATOR_ROOT to the install location.
pub fn build(name: &str, rust_file: &str, verilog_files: &[&str]) -> err::Result<()> {
    for fname in verilog_files {
        if !std::fs::exists(fname)? {
            return err::input!("file {fname} does not exist");
        }
    }

    let item = load_struct(name, rust_file)?;
    let module_name = read_module_name(&item)?;
    let module = Module::from_struct(module_name.clone(), item)?;

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let verilated_dir = format!("{out_dir}/{module_name}_verilated");
    let binding_src = format!("{verilated_dir}/{module_name}_binding.cc");
    write_binding_file(&module_name, &binding_src, &module)?;

    check_process_output(
        "verilator",
        std::process::Command::new("verilator")
            .arg("--cc")
            .arg("--build")
            .args(["--top-module", &module_name])
            .args(["--Mdir", &verilated_dir])
            .args(verilog_files)
            .arg(&binding_src)
            .output()
            .unwrap(),
    );

    let verilator_root = std::env::var("VERILATOR_ROOT").unwrap_or("/usr/share/verilator".into());
    let verilator_include = format!("{verilator_root}/include");
    let binding_obj = format!("{verilated_dir}/{module_name}_binding.o");
    check_process_output(
        "build binding file",
        std::process::Command::new("g++")
            .arg(&format!("-I{verilator_include}"))
            .arg(&format!("-I{verilated_dir}"))
            .args(&["-c", &binding_src])
            .args(&["-o", &binding_obj])
            .output()
            .unwrap(),
    );

    let all_path = format!("{verilated_dir}/V{module_name}__ALL.a");
    let module_path = format!("{verilated_dir}/libV{module_name}.a");
    std::fs::copy(&all_path, &module_path).unwrap();
    check_process_output(
        "archive verilator runtime",
        std::process::Command::new("ar")
            .arg("rcs")
            .arg(&module_path)
            .arg(&format!("{verilated_dir}/{module_name}_binding.o"))
            .arg(&binding_obj)
            .output()
            .unwrap(),
    );

    let verilated_src = format!("{verilator_include}/verilated.cpp");
    let verilated_obj = format!("{verilated_dir}/verilated.o");
    let runtime_path = format!("{verilated_dir}/libverilated.a");
    if is_older(&runtime_path, &verilated_src) {
        check_process_output(
            "build verilator runtime",
            std::process::Command::new("g++")
                .arg(&format!("-I{verilator_include}"))
                .args(&["-c", &verilated_src])
                .args(&["-o", &verilated_obj])
                .output()
                .unwrap(),
        );
        check_process_output(
            "archive verilator runtime",
            std::process::Command::new("ar")
                .arg("rcs")
                .arg(&runtime_path)
                .arg(&verilated_obj)
                .output()
                .unwrap(),
        );
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

fn is_older(lhs: &str, rhs: &str) -> bool {
    if let Ok(lhs) = std::fs::metadata(lhs)
        && let Ok(rhs) = std::fs::metadata(rhs)
        && let Ok(lhs) = lhs.modified()
        && let Ok(rhs) = rhs.modified()
    {
        lhs < rhs
    } else {
        true
    }
}

fn check_process_output(task: &str, out: std::process::Output) {
    if !out.status.success() {
        println!("{task} failed");
        println!("--- stdout ---");
        std::io::copy(&mut &out.stdout[..], &mut std::io::stdout()).unwrap();
        println!("--- stderr ---");
        std::io::copy(&mut &out.stderr[..], &mut std::io::stderr()).unwrap();
        panic!("cannot proceed");
    }
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

    err::input!("failed to find struct defn for {name}")
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
    err::input!(
        "struct {} has no 'ferrilate' attribute",
        item.ident.to_string()
    )
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
