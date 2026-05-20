use darling::{FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Attribute, DeriveInput, Ident, Result, Type, Visibility};

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
}

pub fn derive(input: &DeriveInput) -> Result<TokenStream> {
    let Component { ident, vis, data } = Component::from_derive_input(input)?;
    let fields = data
        .take_struct()
        .expect("macro only works on structs with named fields");

    let fields = &fields.fields;
    let struct_serial = struct_serial(&vis, &ident, fields)?;
    let struct_edit = struct_edit(&vis, &ident, fields)?;

    Ok(quote! {
        #struct_serial
        #struct_edit
    })
}

fn struct_serial(
    vis: &Visibility,
    ident: &Ident,
    fields: &[ComponentField],
) -> Result<TokenStream> {
    let ident_serial = format_ident!("{ident}Serial");

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

            Some(quote! {
                #(#attrs)*
                #vis #ident: #ty
            })
        })
        .collect::<Vec<_>>();

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
    let ident_edit = format_ident!("{ident}Edit");

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

            let serde_attr = if let Type::Path(path) = ty
                && let Some(root_ident) = path.path.segments.first()
                && root_ident.ident == "Option"
            {
                quote! {
                    #[serde(
                        default,
                        skip_serializing_if = "::core::option::Option::is_none",
                        with = "::serde_with::rust::double_option"
                    )]
                }
            } else {
                quote! {
                    #[serde(default)]
                }
            };

            Some((
                quote! {
                    #(#attrs)*
                    #serde_attr
                    #vis #ident: ::core::option::Option<#ty>
                },
                quote! {
                    if let Some(t) = self.#ident {
                        component.#ident = t;
                    }
                },
            ))
        })
        .unzip();

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
