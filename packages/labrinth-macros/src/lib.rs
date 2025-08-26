use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, ItemFn, PatType, parse_macro_input, parse_quote};

#[proc_macro_attribute]
pub fn localized_api_error(
    _args: TokenStream,
    input: TokenStream,
) -> TokenStream {
    let function = parse_macro_input!(input as ItemFn);

    let mut adjusted_sig = function.sig.clone();
    adjusted_sig.output = parse_quote! {
        -> actix_web::HttpResponse
    };

    let vis = function.vis;
    let return_type = function.sig.output;
    let body = function.block;
    let Some(FnArg::Typed(PatType { pat: req, .. })) =
        function.sig.inputs.first()
    else {
        return quote! {
            compile_error!("Expected first parameter to be HttpRequest");
        }
        .into();
    };

    quote! {
        #vis #adjusted_sig {
            let mut handler = async || #return_type #body;
            let result = handler().await;
            match result {
                Ok(resp) => resp,
                Err(e) => e.localized_error_response(&#req),
            }
        }
    }
    .into()
}
