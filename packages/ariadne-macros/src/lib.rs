#[cfg(feature = "labrinth")]
use proc_macro::TokenStream;

// This exists purely to work around https://github.com/actix/actix-web/issues/2925
#[cfg(feature = "labrinth")]
#[proc_macro_attribute]
pub fn localized_labrinth_error(
    _args: TokenStream,
    input: TokenStream,
) -> TokenStream {
    use quote::quote;
    use syn::spanned::Spanned;
    use syn::{Error, FnArg, ItemFn, PatType, parse_macro_input, parse_quote};

    let function = parse_macro_input!(input as ItemFn);

    let mut adjusted_sig = function.sig.clone();
    adjusted_sig.output = parse_quote! {
        -> ::actix_web::HttpResponse
    };

    let vis = function.vis;
    let return_type = function.sig.output;
    let body = function.block;
    let Some(FnArg::Typed(PatType { pat: req, .. })) =
        function.sig.inputs.first()
    else {
        return Error::new(
            function.sig.inputs.span(),
            "Expected first parameter to be HttpRequest",
        )
        .to_compile_error()
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
