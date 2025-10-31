use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Reloadable)]
pub fn derive_reloadable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        impl Reloadable for #name {
            fn reload_from_str(&mut self, data: &str) -> Result<(), Box<dyn std::error::Error>> {
                let new: Self = serde_json::from_str(data)?;
                *self = new;
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}