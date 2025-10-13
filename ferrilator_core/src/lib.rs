pub mod err;

use proc_macro2::Ident;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use proc_macro2::TokenTree;
use quote::ToTokens;
use quote::TokenStreamExt;
use quote::quote;
use syn::ItemStruct;
use syn::Meta;
use syn::Visibility;
use syn::parse2;

pub fn ferrilate_attribute(attr: TokenStream, item: TokenStream) -> err::Result<TokenStream> {
    let module = Module::from_attribute(attr, item)?;

    let vis: TokenStream = module.vis.parse()?;
    let ident = &module.ident;
    let link_name = format!("V{}", module.name);
    let mod_new = Ident::new(&format!("{}_new", module.name), Span::call_site());
    let mod_del = Ident::new(&format!("{}_del", module.name), Span::call_site());
    let mod_eval = Ident::new(&format!("{}_eval", module.name), Span::call_site());

    let clocked_fns = match &module.clock {
        Some((name, data_type)) => {
            let set_fn = Ident::new(&format!("set_{}", name), Span::call_site());
            let (tru, fls) = data_type.true_false();
            let tru = Ident::new(tru, Span::call_site());
            let fls = Ident::new(fls, Span::call_site());
            quote! {
                fn tick(&mut self) {
                    self.#set_fn(#tru);
                    self.eval();
                    self.#set_fn(#fls);
                    self.eval();
                }
            }
        }
        None => {
            quote! {}
        }
    };

    let mut rs_fns = vec![];

    let mut cc_fns = vec![];

    for port in &module.ports {
        let data_type = &port.data_type;

        if port.input {
            let ext_name = Ident::new(
                &format!("{}_set_{}", module.name, port.name),
                Span::call_site(),
            );
            cc_fns.push(quote! {
                fn #ext_name(dut: *mut(), value: #data_type);
            });
            let fn_name = Ident::new(&format!("set_{}", port.name), Span::call_site());
            rs_fns.push(quote! {
                fn #fn_name(&mut self, value: #data_type) {
                    unsafe { #ext_name(self.dut, value) };
                }
            });
        }

        if port.output {
            let ext_name = Ident::new(
                &format!("{}_get_{}", module.name, port.name),
                Span::call_site(),
            );
            cc_fns.push(quote! {
                fn #ext_name(dut: *mut()) -> #data_type;
            });
            let fn_name = Ident::new(&format!("get_{}", port.name), Span::call_site());
            rs_fns.push(quote! {
                fn #fn_name(&self) -> #data_type {
                    unsafe { #ext_name(self.dut) }
                }
            });
        }
    }

    Ok(quote! {
        #vis struct #ident {
            dut: *mut (),
        }

        impl #ident {
            fn new() -> Self {
                let dut = unsafe { #mod_new() };
                Self { dut }
            }

            fn eval(&mut self) {
                unsafe { #mod_eval(self.dut) };
            }

            #clocked_fns

            #(#rs_fns)*
        }

        impl Drop for #ident {
            fn drop(&mut self) {
                unsafe { #mod_del(self.dut) };
            }
        }

        #[link(name = #link_name)]
        unsafe extern "C" {
            fn #mod_new() -> *mut ();
            fn #mod_del(dut: *mut ());
            fn #mod_eval(dut: *mut ());

            #(#cc_fns)*
        }
    })
}

#[derive(Clone, Debug, PartialEq)]
pub enum DataType {
    Bool,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
}

impl DataType {
    fn parse(s: &str) -> err::Result<DataType> {
        Ok(match s {
            "bool" => DataType::Bool,
            "u8" => DataType::U8,
            "u16" => DataType::U16,
            "u32" => DataType::U32,
            "u64" => DataType::U64,
            "i8" => DataType::I8,
            "i16" => DataType::I16,
            "i32" => DataType::I32,
            "i64" => DataType::I64,
            other => return err::input(format!("cannot convert '{other}' to DataType")),
        })
    }

    pub fn as_c(&self) -> String {
        String::from(match self {
            DataType::Bool => "uint8_t",
            DataType::U8 => "uint8_t",
            DataType::U16 => "uint16_t",
            DataType::U32 => "uint32_t",
            DataType::U64 => "uint64_t",
            DataType::I8 => "int8_t",
            DataType::I16 => "int16_t",
            DataType::I32 => "int32_t",
            DataType::I64 => "int64_t",
        })
    }

    fn true_false(&self) -> (&str, &str) {
        match self {
            DataType::Bool => ("true", "false"),
            DataType::U8 => ("1", "0"),
            DataType::U16 => ("1", "0"),
            DataType::U32 => ("1", "0"),
            DataType::U64 => ("1", "0"),
            DataType::I8 => ("1", "0"),
            DataType::I16 => ("1", "0"),
            DataType::I32 => ("1", "0"),
            DataType::I64 => ("1", "0"),
        }
    }
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DataType::Bool => write!(f, "bool"),
            DataType::U8 => write!(f, "u8"),
            DataType::U16 => write!(f, "u16"),
            DataType::U32 => write!(f, "u32"),
            DataType::U64 => write!(f, "u64"),
            DataType::I8 => write!(f, "i8"),
            DataType::I16 => write!(f, "i16"),
            DataType::I32 => write!(f, "i32"),
            DataType::I64 => write!(f, "i64"),
        }
    }
}

impl ToTokens for DataType {
    fn to_tokens(&self, stream: &mut TokenStream) {
        stream.append(Ident::new(&self.to_string(), Span::call_site()));
    }
}

#[derive(Debug, PartialEq)]
pub struct Port {
    name: String,
    data_type: DataType,
    input: bool,
    output: bool,
}

impl Port {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn data_type(&self) -> &DataType {
        &self.data_type
    }

    pub fn input(&self) -> bool {
        self.input
    }

    pub fn output(&self) -> bool {
        self.output
    }
}

#[derive(Debug, PartialEq)]
pub struct Module {
    name: String,
    vis: String,
    ident: syn::Ident,
    clock: Option<(String, DataType)>,
    ports: Vec<Port>,
}

impl Module {
    pub fn ports(&self) -> &Vec<Port> {
        &self.ports
    }

    fn from_attribute(attr: TokenStream, item: TokenStream) -> err::Result<Module> {
        let mut attr = attr.into_iter();
        let name = match attr.next() {
            Some(TokenTree::Ident(id)) => id.to_string(),
            Some(other) => return err::input(format!("expected module name, found {other}")),
            None => return err::input("expected module name, found nothing"),
        };

        match attr.next() {
            None => {}
            Some(token) => return err::input(format!("unexpected attr value: {token}")),
        }

        Self::from_struct(name, parse2(item)?)
    }

    pub fn from_struct(name: String, defn: ItemStruct) -> err::Result<Module> {
        let mut clock = None;
        let mut ports = vec![];
        for field in &defn.fields {
            if field.vis != Visibility::Inherited {
                return err::input("fields must be private");
            }
            let name = match &field.ident {
                Some(ident) => ident.to_string(),
                None => return err::input("fields must be named"),
            };

            let ty = as_tokens(&field.ty);
            let data_type = DataType::parse(&ty.to_string())?;
            let mut input = false;
            let mut output = false;
            for attr in &field.attrs {
                match &attr.meta {
                    Meta::Path(path) => match path.get_ident() {
                        Some(v) => match v.to_string().as_str() {
                            "input" => input = true,
                            "output" => output = true,
                            "clock" => {
                                match &clock {
                                    Some((previous, _)) => {
                                        return err::input(format!(
                                            "fields {previous} and {name} cannot both be declared clock"
                                        ));
                                    }
                                    None => {}
                                }
                                clock = Some((name.clone(), data_type.clone()));
                            }
                            _ => {}
                        },
                        None => {}
                    },
                    _ => {}
                }
            }

            ports.push(Port {
                name,
                data_type,
                input,
                output,
            });
        }

        let vis = as_tokens(&defn.vis).to_string();
        let ident = defn.ident.clone();

        Ok(Module {
            name,
            vis,
            ident,
            clock,
            ports,
        })
    }
}

fn as_tokens<T: ToTokens>(v: &T) -> TokenStream {
    let mut ts = TokenStream::new();
    v.to_tokens(&mut ts);
    ts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ferrilate() -> err::Result<()> {
        let attr = quote! { ex_module };
        let item = quote! {
            pub struct Example {
                #[clock]
                #[input]
                clk: bool,

                #[input]
                a: u8,

                #[output]
                b: u64,
            }
        };

        let output = ferrilate_attribute(attr, item)?;

        snapshot("example.rs", output);
        Ok(())
    }

    #[test]
    fn module_from_attribute() -> err::Result<()> {
        let attr = quote! { ex_module };
        let item = quote! {
            pub struct Example {
                #[clock]
                #[input]
                clk: bool,

                #[input]
                a: u8,

                #[output]
                b: u64,
            }
        };

        let module = Module::from_attribute(attr, item)?;

        assert_eq!(
            module,
            Module {
                name: "ex_module".into(),
                clock: Some(("clk".into(), DataType::Bool)),
                vis: String::from("pub"),
                ident: syn::Ident::new("Example", Span::call_site()),
                ports: vec![
                    Port {
                        name: "clk".into(),
                        data_type: DataType::Bool,
                        input: true,
                        output: false,
                    },
                    Port {
                        name: "a".into(),
                        data_type: DataType::U8,
                        input: true,
                        output: false,
                    },
                    Port {
                        name: "b".into(),
                        data_type: DataType::U64,
                        input: false,
                        output: true,
                    }
                ],
            }
        );

        Ok(())
    }

    fn snapshot(name: &str, stream: TokenStream) {
        let mut path = std::path::PathBuf::from("snapshots");
        path.push(name);

        let parsed = syn::parse2(stream).unwrap();
        let source = prettyplease::unparse(&parsed);

        if std::env::var("SNAP").is_ok() {
            use std::io::Write;
            let mut file = std::fs::File::create(path).unwrap();
            file.write(source.as_bytes()).unwrap();
        } else {
            use std::io::Read;
            let mut file = std::fs::File::open(path).unwrap();
            let mut expected = String::new();
            file.read_to_string(&mut expected).unwrap();
            assert_eq!(expected, source);
        }
    }
}
