#![doc = include_str!("../README.md")]

use darling::{FromMeta, ast::NestedMeta};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::{
    Attribute, Error, Fields, Ident, Item, ItemEnum, ItemStruct, Meta, Path,
    Result, Token, parse_macro_input, parse_quote,
};

#[derive(Default, FromMeta)]
struct Args {
    #[darling(default)]
    schema: bool,
}

#[derive(Default, FromMeta)]
struct AttributeArgs {
    #[darling(default)]
    human: Option<SerdeOptions>,
    #[darling(default)]
    binary: Option<SerdeOptions>,
}

struct SerdeOptions(Vec<Meta>);

impl FromMeta for SerdeOptions {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        let options = items
            .iter()
            .map(|item| match item {
                NestedMeta::Meta(option) => Ok(option.clone()),
                NestedMeta::Lit(literal) => {
                    Err(darling::Error::custom("expected a Serde option")
                        .with_span(literal))
                }
            })
            .collect::<darling::Result<Vec<_>>>()?;
        Ok(Self(options))
    }
}

#[derive(Clone, Copy)]
enum Representation {
    Human,
    Binary,
}

#[proc_macro_attribute]
/// Implements `Serialize` and `Deserialize` through generated human and binary
/// remote proxies.
///
/// The annotated type must not derive either trait itself. Ordinary
/// `#[serde(...)]` attributes define the human-readable representation.
///
/// # Attributes
///
/// - **`#[serde_binhum]`**: Generates human-readable and binary Serde proxies
///   and delegates `Serialize` and `Deserialize` to the appropriate proxy.
/// - **`#[serde_binhum(schema)]`**: Also forwards `utoipa::PartialSchema` and
///   `utoipa::ToSchema` to the human-readable proxy.
/// - **`#[serde_binhum(human(...))]`**: Adds the enclosed Serde options only
///   to the human-readable proxy. This can be placed on the type, a variant, or
///   a field.
/// - **`#[serde_binhum(binary(...))]`**: Adds the enclosed Serde options only
///   to the binary proxy. This can be placed on the type, a variant, or a field.
pub fn serde_binhum(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = match NestedMeta::parse_meta_list(args.into())
        .map_err(darling::Error::from)
        .and_then(|args| Args::from_list(&args))
    {
        Ok(args) => args,
        Err(error) => return error.write_errors().into(),
    };
    let item = parse_macro_input!(input as Item);

    expand(args, item)
        .unwrap_or_else(|error| error.to_compile_error())
        .into()
}

fn expand(args: Args, item: Item) -> Result<proc_macro2::TokenStream> {
    match item {
        Item::Enum(item) => expand_enum(args, item),
        Item::Struct(item) => expand_struct(args, item),
        item => Err(Error::new_spanned(
            item,
            "`serde_binhum` only supports structs and enums",
        )),
    }
}

fn expand_enum(
    args: Args,
    mut item: ItemEnum,
) -> Result<proc_macro2::TokenStream> {
    validate_item(&item.ident, &item.generics.params, &item.attrs)?;

    let ident = item.ident.clone();
    let human = enum_proxy(&item, Representation::Human, args.schema)?;
    let binary = enum_proxy(&item, Representation::Binary, false)?;
    clean_attributes(&mut item.attrs);
    for variant in &mut item.variants {
        clean_attributes(&mut variant.attrs);
        clean_fields(&mut variant.fields);
    }
    let implementations = implementations(&ident, args.schema);

    Ok(quote! {
        #item

        const _: () = {
            #human
            #binary
            #implementations
        };
    })
}

fn expand_struct(
    args: Args,
    mut item: ItemStruct,
) -> Result<proc_macro2::TokenStream> {
    validate_item(&item.ident, &item.generics.params, &item.attrs)?;

    let ident = item.ident.clone();
    let human = struct_proxy(&item, Representation::Human, args.schema)?;
    let binary = struct_proxy(&item, Representation::Binary, false)?;
    clean_attributes(&mut item.attrs);
    clean_fields(&mut item.fields);
    let implementations = implementations(&ident, args.schema);

    Ok(quote! {
        #item

        const _: () = {
            #human
            #binary
            #implementations
        };
    })
}

fn validate_item(
    ident: &Ident,
    generics: &Punctuated<syn::GenericParam, Token![,]>,
    attrs: &[Attribute],
) -> Result<()> {
    if !generics.is_empty() {
        return Err(Error::new_spanned(
            generics,
            "`serde_binhum` does not yet support generic types",
        ));
    }

    for attr in attrs.iter().filter(|attr| attr.path().is_ident("derive")) {
        let derives = attr
            .parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated)?;
        for derive in derives {
            let Some(name) = derive.segments.last() else {
                continue;
            };
            if name.ident == "Serialize" || name.ident == "Deserialize" {
                return Err(Error::new_spanned(
                    derive,
                    format!(
                        "`{ident}` must not derive `Serialize` or `Deserialize`; `#[serde_binhum]` implements both"
                    ),
                ));
            }
        }
    }

    Ok(())
}

fn enum_proxy(
    item: &ItemEnum,
    representation: Representation,
    schema: bool,
) -> Result<ItemEnum> {
    let mut proxy = item.clone();
    proxy.ident = match representation {
        Representation::Human => format_ident!("HumanProxy"),
        Representation::Binary => format_ident!("BinaryProxy"),
    };
    proxy.vis = syn::Visibility::Inherited;
    proxy.attrs = proxy_item_attributes(
        &item.attrs,
        &item.ident,
        representation,
        schema,
    )?;

    for variant in &mut proxy.variants {
        variant.attrs = proxy_attributes(&variant.attrs, representation)?;
        proxy_fields(&mut variant.fields, representation)?;
    }

    Ok(proxy)
}

fn struct_proxy(
    item: &ItemStruct,
    representation: Representation,
    schema: bool,
) -> Result<ItemStruct> {
    let mut proxy = item.clone();
    proxy.ident = match representation {
        Representation::Human => format_ident!("HumanProxy"),
        Representation::Binary => format_ident!("BinaryProxy"),
    };
    proxy.vis = syn::Visibility::Inherited;
    proxy.attrs = proxy_item_attributes(
        &item.attrs,
        &item.ident,
        representation,
        schema,
    )?;
    proxy_fields(&mut proxy.fields, representation)?;

    Ok(proxy)
}

fn proxy_item_attributes(
    attrs: &[Attribute],
    remote: &Ident,
    representation: Representation,
    schema: bool,
) -> Result<Vec<Attribute>> {
    let mut attrs = proxy_attributes(attrs, representation)?;
    let derive = if schema {
        parse_quote!(#[derive(serde::Deserialize, serde::Serialize, utoipa::ToSchema)])
    } else {
        parse_quote!(#[derive(serde::Deserialize, serde::Serialize)])
    };
    let remote = remote.to_string();
    let remote: Attribute = parse_quote!(#[serde(remote = #remote)]);
    attrs.insert(0, remote);
    attrs.insert(0, derive);
    Ok(attrs)
}

fn proxy_fields(
    fields: &mut Fields,
    representation: Representation,
) -> Result<()> {
    for field in fields {
        field.attrs = proxy_attributes(&field.attrs, representation)?;
    }
    Ok(())
}

fn proxy_attributes(
    attrs: &[Attribute],
    representation: Representation,
) -> Result<Vec<Attribute>> {
    let mut output = Vec::new();
    let mut representation_options = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("serde") {
            let mut options = attr
                .parse_args_with(
                    Punctuated::<Meta, Token![,]>::parse_terminated,
                )?
                .into_iter()
                .collect::<Vec<_>>();
            if matches!(representation, Representation::Binary) {
                options.retain(|option| !binary_incompatible(option));
            }
            if !options.is_empty() {
                output.push(parse_quote!(#[serde(#(#options),*)]));
            }
        } else if attr.path().is_ident("serde_binhum") {
            representation_options
                .extend(parse_representation_options(attr, representation)?);
        } else if attr.path().is_ident("doc") || attr.path().is_ident("schema")
        {
            output.push(attr.clone());
        }
    }

    if !representation_options.is_empty() {
        output.push(parse_quote!(#[serde(#(#representation_options),*)]));
    }

    Ok(output)
}

fn parse_representation_options(
    attr: &Attribute,
    representation: Representation,
) -> Result<Vec<Meta>> {
    let args = AttributeArgs::from_meta(&attr.meta)
        .map_err(|error| Error::new(error.span(), error.to_string()))?;
    if args.human.is_none() && args.binary.is_none() {
        return Err(Error::new_spanned(
            attr,
            "expected `human(...)` or `binary(...)`",
        ));
    }

    Ok(match representation {
        Representation::Human => args.human,
        Representation::Binary => args.binary,
    }
    .map_or_else(Vec::new, |options| options.0))
}

fn binary_incompatible(option: &Meta) -> bool {
    let path = option.path();
    path.is_ident("flatten")
        || path.is_ident("tag")
        || path.is_ident("content")
        || path.is_ident("untagged")
        || path.is_ident("skip_serializing_if")
}

fn clean_fields(fields: &mut Fields) {
    for field in fields {
        clean_attributes(&mut field.attrs);
    }
}

fn clean_attributes(attrs: &mut Vec<Attribute>) {
    attrs.retain(|attr| {
        !attr.path().is_ident("serde") && !attr.path().is_ident("serde_binhum")
    });
}

fn implementations(ident: &Ident, schema: bool) -> proc_macro2::TokenStream {
    let schema = schema.then(|| {
        quote! {
            impl ::utoipa::PartialSchema for #ident {
                fn schema()
                -> ::utoipa::openapi::RefOr<::utoipa::openapi::schema::Schema> {
                    <HumanProxy as ::utoipa::PartialSchema>::schema()
                }
            }

            impl ::utoipa::ToSchema for #ident {
                fn schemas(
                    schemas: &mut ::std::vec::Vec<(
                        ::std::string::String,
                        ::utoipa::openapi::RefOr<::utoipa::openapi::schema::Schema>,
                    )>,
                ) {
                    <HumanProxy as ::utoipa::ToSchema>::schemas(schemas);
                }
            }
        }
    });

    quote! {
        impl ::serde::Serialize for #ident {
            fn serialize<S>(
                &self,
                serializer: S,
            ) -> ::std::result::Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                if serializer.is_human_readable() {
                    HumanProxy::serialize(self, serializer)
                } else {
                    BinaryProxy::serialize(self, serializer)
                }
            }
        }

        impl<'de> ::serde::Deserialize<'de> for #ident {
            fn deserialize<D>(
                deserializer: D,
            ) -> ::std::result::Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                if deserializer.is_human_readable() {
                    HumanProxy::deserialize(deserializer)
                } else {
                    BinaryProxy::deserialize(deserializer)
                }
            }
        }

        #schema
    }
}
