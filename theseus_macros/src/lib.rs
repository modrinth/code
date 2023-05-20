extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn debug_pin(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let attrs = &input.attrs;
    let vis = &input.vis;
    let sig = &input.sig;
    let body = &input.block;

    // Generate tokens for the common part
    let common_tokens = quote! {
        #(#attrs)*
        #vis #sig
    };

    let result = quote! {
        #[cfg(debug_assertions)]
        #common_tokens {
            Box::pin(async move {
                #body
            }).await
        }

        #[cfg(not(debug_assertions))]
        #common_tokens {
            #body
        }
    };

    TokenStream::from(result)
}
