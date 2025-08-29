mod i18n_enum;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// This derive macro defines three attributes:
/// - `i18n_root_key`: Placed on the enum itself to specify a root translation key
/// - `error_id`: Placed on each enum element to define a string ID for errors. This will be used
///   for translation keys
/// - `translate_fields`: Optionally placed on each enum element to pass field names to the
///   translation, interpolated with `%(field_name)` in the translations. The member name can be
///   surrounded with `translate` to recursively translate it
///
/// Example:
/// ```
/// #[derive(I18nEnum)]
/// #[i18n_root_key("error.example")]
/// enum ExampleEnum {
///     #[error_id("example")]
///     #[field_names(cause = 0)]
///     Example(SomeEnum),
///
///     #[error_id("translated_example")]
///     #[field_names(cause = translate(0))]
///     TranslatedExample(SomeTranslatableEnum),
/// }
/// ```
#[proc_macro_derive(
    I18nEnum,
    attributes(i18n_root_key, translation_id, translate_fields)
)]
pub fn i18n_enum(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    i18n_enum::generate_impls(input)
        .unwrap_or_else(|err| err.to_compile_error().into())
}

// This exists purely to work around https://github.com/actix/actix-web/issues/2925
#[cfg(feature = "labrinth")]
#[proc_macro_attribute]
pub fn localized_labrinth_error(
    _args: TokenStream,
    input: TokenStream,
) -> TokenStream {
    use quote::quote;
    use syn::spanned::Spanned;
    use syn::{Error, FnArg, ItemFn, PatType, parse_quote};

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
