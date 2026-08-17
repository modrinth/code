//! This crate is currently unused, but will replace the `macro_rules!` component
//! logic in Labrinth experimental API.

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod component;

#[proc_macro_derive(Component, attributes(component))]
pub fn component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    component::derive(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
