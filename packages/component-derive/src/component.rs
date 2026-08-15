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
    let struct_edit = struct_edit(&vis, &ident, fields)?;

    // `#[validate(nested)]` needs `Validate` in scope; `as _` avoids a name clash
    let validate_import = if fields.iter().any(|field| field.nested) {
        quote! { use validator::Validate as _; }
    } else {
        quote! {}
    };

    Ok(quote! {
        #validate_import

        #struct_serial
        #struct_edit
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

fn struct_edit(
    vis: &Visibility,
    ident: &Ident,
    fields: &[ComponentField],
) -> Result<TokenStream> {
    let ident_edit = format_ident!("Edit{ident}");

    let (fields, apply_fields): (Vec<_>, Vec<_>) = fields
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

            let (inner_ty, apply_value, validate_attr) = if field.nested {
                let inner_ty = match nested_type(ty, "Edit") {
                    Ok(inner_ty) => inner_ty,
                    Err(err) => return Some(Err(err)),
                };
                (
                    inner_ty,
                    quote! { t.apply_to(&mut component.#ident) },
                    quote! { #[validate(nested)] },
                )
            } else {
                (
                    quote! { #ty },
                    quote! { component.#ident = t },
                    quote! {},
                )
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
                quote! { #[serde(default)] }
            };

            Some(Ok((
                quote! {
                    #(#attrs)*
                    #validate_attr
                    #serde_attr
                    #vis #ident: ::core::option::Option<#inner_ty>
                },
                quote! {
                    if let Some(t) = self.#ident {
                        #apply_value;
                    }
                },
            )))
        })
        .collect::<Result<(Vec<_>, Vec<_>)>>()?;

    Ok(quote! {
        #[derive(
            Debug,
            Clone,
            ::serde::Serialize,
            ::serde::Deserialize,
            ::validator::Validate,
            ::utoipa::ToSchema,
        )]
        #vis struct #ident_edit {
            #(#fields),*
        }

        impl #ident_edit {
            pub fn apply_to(
                self,
                component: &mut #ident,
            ) {
                #(#apply_fields)*
            }
        }
    })
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
