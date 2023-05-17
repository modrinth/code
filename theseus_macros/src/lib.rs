extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn debug_pin(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(item as ItemFn);

    let attrs = &input.attrs;
    let vis = &input.vis; // visibility modifier
    let sig = &input.sig; // function signature
    let body = &input.block;

    // Use cfg attribute for conditional compilation
    let result = quote! {
        #[cfg(debug_assertions)]
        #(#attrs) *
        #vis #sig {
            Box::pin(async move {
                #body
            }).await
        }

        #[cfg(not(debug_assertions))]
        #(#attrs) *
        #vis #sig {
            #body
        }
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(result)
}
