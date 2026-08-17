use darling::{FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Attribute, DeriveInput, Error, Ident, Result, Type, Visibility};

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_named))]
struct Component {
    ident: Ident,
    vis: Visibility,
    data: darling::ast::Data<(), ComponentField>,
}

#[derive(Debug, FromField)]
#[darling(attributes(component), forward_attrs)]
struct ComponentField {
    ident: Option<Ident>,
    vis: Visibility,
    ty: Type,
    attrs: Vec<Attribute>,
    #[darling(default)]
    synthetic: bool,
    #[darling(default)]
    nested: bool,
}

pub fn derive(input: &DeriveInput) -> Result<TokenStream> {
    let Component { ident, vis, data } = Component::from_derive_input(input)?;
    let fields = data
        .take_struct()
        .expect("macro only works on structs with named fields");

    let fields = &fields.fields;
    let struct_serial = struct_serial(&vis, &ident, fields)?;
    let struct_partial = struct_partial(&vis, &ident, fields)?;
    let impl_apply_to = impl_apply_to(&ident, fields);
    let impl_into_diff_from = impl_into_diff_from(&ident, fields);

    // `#[validate(nested)]` needs `Validate` in scope; `as _` avoids a name clash
    let validate_import = if fields.iter().any(|field| field.nested) {
        quote! { use validator::Validate as _; }
    } else {
        quote! {}
    };

    Ok(quote! {
        #validate_import

        #struct_serial
        #struct_partial

        const _: () = {
            #impl_apply_to
            #impl_into_diff_from
        };
    })
}

fn struct_serial(
    vis: &Visibility,
    ident: &Ident,
    fields: &[ComponentField],
) -> Result<TokenStream> {
    let ident_serial = format_ident!("Serial{ident}");

    let fields = fields
        .iter()
        .filter_map(|field| {
            if field.synthetic {
                return None;
            }

            let ident = &field
                .ident
                .as_ref()
                .expect("macro only works on structs with named fields");
            let vis = &field.vis;
            let ty = &field.ty;
            let attrs = &field.attrs;

            let (field_ty, validate_attr) = if field.nested {
                let field_ty = match nested_type(ty, "Serial") {
                    Ok(field_ty) => field_ty,
                    Err(err) => return Some(Err(err)),
                };
                (field_ty, quote! { #[validate(nested)] })
            } else {
                (quote! { #ty }, quote! {})
            };

            Some(Ok(quote! {
                #(#attrs)*
                #validate_attr
                #vis #ident: #field_ty
            }))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        #[derive(
            Debug,
            Clone,
            ::serde::Serialize,
            ::serde::Deserialize,
            ::validator::Validate,
            ::utoipa::ToSchema,
        )]
        #vis struct #ident_serial {
            #(#fields),*
        }
    })
}

fn struct_partial(
    vis: &Visibility,
    ident: &Ident,
    fields: &[ComponentField],
) -> Result<TokenStream> {
    let ident_partial = format_ident!("Partial{ident}");

    let fields = fields
        .iter()
        .filter_map(|field| {
            if field.synthetic {
                return None;
            }

            let ident = &field
                .ident
                .as_ref()
                .expect("macro only works on structs with named fields");
            let vis = &field.vis;
            let ty = &field.ty;
            let attrs = &field.attrs;

            let (inner_ty, validate_attr) = if field.nested {
                let inner_ty = match nested_type(ty, "Partial") {
                    Ok(inner_ty) => inner_ty,
                    Err(err) => return Some(Err(err)),
                };
                (inner_ty, quote! { #[validate(nested)] })
            } else {
                (quote! { #ty }, quote! {})
            };

            let serde_attr = if !field.nested
                && let Type::Path(path) = ty
                && path
                    .path
                    .segments
                    .first()
                    .is_some_and(|segment| segment.ident == "Option")
            {
                quote! {
                    #[serde(
                        default,
                        skip_serializing_if = "::core::option::Option::is_none",
                        with = "::serde_with::rust::double_option"
                    )]
                }
            } else {
                quote! { #[serde(default, skip_serializing_if = "::core::option::Option::is_none")] }
            };

            Some(Ok(quote! {
                #(#attrs)*
                #validate_attr
                #serde_attr
                #vis #ident: ::core::option::Option<#inner_ty>
            }))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        #[derive(
            Debug,
            Clone,
            ::serde::Serialize,
            ::serde::Deserialize,
            ::validator::Validate,
            ::utoipa::ToSchema,
        )]
        #vis struct #ident_partial {
            #(#fields),*
        }
    })
}

fn impl_apply_to(ident: &Ident, fields: &[ComponentField]) -> TokenStream {
    let ident_partial = format_ident!("Partial{ident}");

    let apply_fields = fields
        .iter()
        .filter_map(|field| {
            if field.synthetic {
                return None;
            }

            let ident = field
                .ident
                .as_ref()
                .expect("macro only works on structs with named fields");
            let apply_value = if field.nested {
                quote! { t.apply_to(&mut component.#ident) }
            } else {
                quote! { component.#ident = t }
            };

            Some(quote! {
                if let Some(t) = self.#ident {
                    #apply_value;
                }
            })
        })
        .collect::<Vec<_>>();

    quote! {
        impl #ident_partial {
            pub fn apply_to(self, component: &mut #ident) {
                #(#apply_fields)*
            }
        }
    }
}

fn impl_into_diff_from(
    ident: &Ident,
    fields: &[ComponentField],
) -> TokenStream {
    let ident_partial = format_ident!("Partial{ident}");

    let diff_fields = fields
        .iter()
        .filter_map(|field| {
            if field.synthetic {
                return None;
            }

            let ident = field
                .ident
                .as_ref()
                .expect("macro only works on structs with named fields");
            let diff_value = if field.nested {
                quote! { self.#ident.into_diff_from(&base.#ident) }
            } else {
                quote! { self.#ident }
            };

            Some(quote! {
                #ident: (self.#ident != base.#ident).then(|| #diff_value)
            })
        })
        .collect::<Vec<_>>();

    quote! {
        impl #ident {
            pub fn into_diff_from(self, base: &Self) -> #ident_partial {
                #ident_partial {
                    #(#diff_fields),*
                }
            }
        }
    }
}

fn nested_type(ty: &Type, prefix: &str) -> Result<TokenStream> {
    if let Type::Path(path) = ty
        && let Some(segment) = path.path.segments.last()
    {
        // FIXME: Validate that nested type also derives component, prob by checking for component impl
        let nested = format_ident!("{}{}", prefix, segment.ident);
        Ok(quote! { #nested })
    } else {
        Err(Error::new_spanned(
            ty,
            "nested component fields must be a named path type",
        ))
    }
}
