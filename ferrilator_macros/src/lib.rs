use ferrilator_core::ferrilate_attribute;
use proc_macro::TokenStream;

#[proc_macro_attribute]
/// Mark a struct to generate bindings for Verilated C++.
pub fn ferrilate(attr: TokenStream, item: TokenStream) -> TokenStream {
    match ferrilate_attribute(attr.into(), item.into()) {
        Ok(tok) => tok.into(),
        Err(e) => {
            panic!("{e}");
        }
    }
}
