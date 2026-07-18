use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Field, Fields,
    GenericParam, Generics, Lifetime, LifetimeParam, Meta, Token, Type,
    parse_macro_input, parse_quote,
};

#[proc_macro_derive(CachedProjection, attributes(cached_projection))]
pub fn cached_projection(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_cached_projection(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[derive(Clone, Copy)]
enum Projection {
    Ordinary,
    Nested,
    Wrap,
}

fn derive_cached_projection(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let projected_name = format_ident!("{name}ProjectedType");
    let projected_ref_name = format_ident!("{name}ProjectedTypeRef");
    let visibility = &input.vis;
    let lifetime = Lifetime::new("'__cached_projection", name.span());

    let fields = all_fields(&input.data);
    let ref_lifetime = (!fields.is_empty()).then_some(&lifetime);
    let projections = fields
        .iter()
        .map(|field| field_projection(field))
        .collect::<syn::Result<Vec<_>>>()?;

    let mut implementation_generics = input.generics.clone();
    add_projection_bounds(
        &mut implementation_generics,
        fields.iter().zip(&projections),
    );
    let (impl_generics, _, where_clause) =
        implementation_generics.split_for_impl();
    let (_, type_generics, _) = input.generics.split_for_impl();

    let mut ref_generics = implementation_generics.clone();
    if ref_lifetime.is_some() {
        ref_generics.params.insert(
            0,
            GenericParam::Lifetime(LifetimeParam::new(lifetime.clone())),
        );
        add_outlives_bounds(&mut ref_generics, fields.iter(), &lifetime);
    }
    let ref_type_arguments = generic_arguments(&input.generics, ref_lifetime);

    let owned_definition = projected_definition(
        &input.data,
        &projected_name,
        visibility,
        &implementation_generics,
        None,
    )?;
    let ref_definition = projected_definition(
        &input.data,
        &projected_ref_name,
        visibility,
        &ref_generics,
        ref_lifetime,
    )?;

    let project_ref_body =
        project_ref_body(&input.data, &projected_ref_name, &projections)?;
    let into_projected_body =
        into_projected_body(&input.data, &projected_name, &projections)?;
    let from_projected_body =
        from_projected_body(&input.data, &projected_name, &projections)?;

    Ok(quote! {
        #owned_definition
        #ref_definition

        impl #impl_generics ::cached_projection::CachedProjection for #name #type_generics #where_clause {
            type ProjectedType = #projected_name #type_generics;
            type ProjectedTypeRef<#lifetime> = #projected_ref_name #ref_type_arguments
            where
                Self: #lifetime;

            fn project_ref(&self) -> Self::ProjectedTypeRef<'_> {
                #project_ref_body
            }

            fn into_projected(self) -> Self::ProjectedType {
                #into_projected_body
            }

            fn from_projected(projected: Self::ProjectedType) -> Self {
                #from_projected_body
            }
        }
    })
}

fn all_fields(data: &Data) -> Vec<&Field> {
    match data {
        Data::Struct(data) => data.fields.iter().collect(),
        Data::Enum(data) => data
            .variants
            .iter()
            .flat_map(|variant| variant.fields.iter())
            .collect(),
        Data::Union(data) => data.fields.named.iter().collect(),
    }
}

fn field_projection(field: &Field) -> syn::Result<Projection> {
    let mut nested = false;
    let mut wrap = false;

    for attribute in &field.attrs {
        if !attribute.path().is_ident("cached_projection") {
            continue;
        }

        attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("nested") {
                nested = true;
                Ok(())
            } else if meta.path.is_ident("wrap") {
                wrap = true;
                Ok(())
            } else {
                Err(meta.error("unsupported cached projection option"))
            }
        })?;
    }

    if nested && wrap {
        return Err(syn::Error::new_spanned(
            field,
            "`nested` cannot be combined with `wrap`",
        ));
    }

    Ok(if nested {
        Projection::Nested
    } else if wrap {
        Projection::Wrap
    } else {
        Projection::Ordinary
    })
}

fn add_projection_bounds<'a>(
    generics: &mut Generics,
    fields: impl Iterator<Item = (&'a &'a Field, &'a Projection)>,
) {
    let where_clause = generics.make_where_clause();
    for (field, projection) in fields {
        let ty = &field.ty;
        let predicate = match projection {
            Projection::Nested => {
                parse_quote!(#ty: ::cached_projection::CachedProjection)
            }
            Projection::Ordinary | Projection::Wrap => parse_quote!(
                #ty: ::serde::Serialize + ::serde::de::DeserializeOwned
            ),
        };
        where_clause.predicates.push(predicate);
    }
}

fn add_outlives_bounds<'a>(
    generics: &mut Generics,
    fields: impl Iterator<Item = &'a &'a Field>,
    lifetime: &Lifetime,
) {
    let where_clause = generics.make_where_clause();
    for field in fields {
        let ty = &field.ty;
        where_clause.predicates.push(parse_quote!(#ty: #lifetime));
    }
}

fn generic_arguments(
    generics: &Generics,
    lifetime: Option<&Lifetime>,
) -> TokenStream2 {
    let mut arguments = Vec::new();
    if let Some(lifetime) = lifetime {
        arguments.push(quote!(#lifetime));
    }
    arguments.extend(generics.params.iter().map(|parameter| match parameter {
        GenericParam::Lifetime(parameter) => {
            let lifetime = &parameter.lifetime;
            quote!(#lifetime)
        }
        GenericParam::Type(parameter) => {
            let ident = &parameter.ident;
            quote!(#ident)
        }
        GenericParam::Const(parameter) => {
            let ident = &parameter.ident;
            quote!(#ident)
        }
    }));

    if arguments.is_empty() {
        quote!()
    } else {
        quote!(<#(#arguments),*>)
    }
}

fn projected_definition(
    data: &Data,
    name: &syn::Ident,
    visibility: &syn::Visibility,
    generics: &Generics,
    lifetime: Option<&Lifetime>,
) -> syn::Result<TokenStream2> {
    let derive = if lifetime.is_some() {
        quote!(#[derive(::serde::Serialize)])
    } else {
        quote!(#[derive(::serde::Serialize, ::serde::Deserialize)])
    };
    let serde_bound = serde_bound_attribute(data, lifetime);

    match data {
        Data::Struct(data) => {
            let fields = projected_fields(&data.fields, lifetime)?;
            let where_clause = &generics.where_clause;
            let declaration = match &data.fields {
                Fields::Named(_) => quote!({ #(#fields),* }),
                Fields::Unnamed(_) => {
                    quote!(( #(#fields),* ) #where_clause;)
                }
                Fields::Unit => quote!(#where_clause;),
            };
            let leading_where_clause = match &data.fields {
                Fields::Named(_) => quote!(#where_clause),
                Fields::Unnamed(_) | Fields::Unit => quote!(),
            };
            Ok(quote! {
                #[doc(hidden)]
                #derive
                #serde_bound
                #visibility struct #name #generics #leading_where_clause #declaration
            })
        }
        Data::Enum(data) => {
            let where_clause = &generics.where_clause;
            let variants = data
                .variants
                .iter()
                .map(|variant| {
                    let attributes = definition_attributes(&variant.attrs)?;
                    let ident = &variant.ident;
                    let fields = projected_fields(&variant.fields, lifetime)?;
                    let declaration = match &variant.fields {
                        Fields::Named(_) => quote!({ #(#fields),* }),
                        Fields::Unnamed(_) => quote!(( #(#fields),* )),
                        Fields::Unit => quote!(),
                    };
                    Ok(quote!(#(#attributes)* #ident #declaration))
                })
                .collect::<syn::Result<Vec<_>>>()?;
            Ok(quote! {
                #[doc(hidden)]
                #derive
                #serde_bound
                #visibility enum #name #generics #where_clause {
                    #(#variants),*
                }
            })
        }
        Data::Union(data) => Err(syn::Error::new_spanned(
            &data.union_token,
            "CachedProjection cannot be derived for unions",
        )),
    }
}

fn serde_bound_attribute(
    data: &Data,
    lifetime: Option<&Lifetime>,
) -> TokenStream2 {
    let projected_types = all_fields(data)
        .into_iter()
        .map(|field| {
            let projection = field_projection(field)
                .expect("field projection was already validated");
            projected_field_type(&field.ty, projection, lifetime)
        })
        .collect::<Vec<_>>();
    if projected_types.is_empty() {
        return quote!();
    }
    let serialize_bounds = projected_types
        .iter()
        .map(|ty| quote!(#ty: ::serde::Serialize))
        .collect::<Vec<_>>();
    let serialize_bounds = syn::LitStr::new(
        &quote!(#(#serialize_bounds),*).to_string(),
        proc_macro2::Span::call_site(),
    );

    if lifetime.is_some() {
        quote!(#[serde(bound(serialize = #serialize_bounds))])
    } else {
        let deserialize_bounds = projected_types
            .iter()
            .map(|ty| quote!(#ty: ::serde::de::DeserializeOwned))
            .collect::<Vec<_>>();
        let deserialize_bounds = syn::LitStr::new(
            &quote!(#(#deserialize_bounds),*).to_string(),
            proc_macro2::Span::call_site(),
        );
        quote!(#[serde(bound(
            serialize = #serialize_bounds,
            deserialize = #deserialize_bounds,
        ))])
    }
}

fn projected_fields(
    fields: &Fields,
    lifetime: Option<&Lifetime>,
) -> syn::Result<Vec<TokenStream2>> {
    let projected = fields
        .iter()
        .map(|field| {
            let attributes = definition_attributes(&field.attrs)?;
            let visibility = &field.vis;
            let ident = &field.ident;
            let projection = field_projection(field)?;
            let ty = projected_field_type(&field.ty, projection, lifetime);
            Ok(if let Some(ident) = ident {
                quote!(#(#attributes)* #visibility #ident: #ty)
            } else {
                quote!(#(#attributes)* #visibility #ty)
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;

    Ok(projected)
}

fn projected_field_type(
    ty: &Type,
    projection: Projection,
    lifetime: Option<&Lifetime>,
) -> TokenStream2 {
    match (projection, lifetime) {
        (Projection::Ordinary, None) => quote!(#ty),
        (Projection::Ordinary, Some(lifetime)) => quote!(&#lifetime #ty),
        (Projection::Nested, None) => quote!(
            <#ty as ::cached_projection::CachedProjection>::ProjectedType
        ),
        (Projection::Nested, Some(lifetime)) => quote!(
            <#ty as ::cached_projection::CachedProjection>::ProjectedTypeRef<#lifetime>
        ),
        (Projection::Wrap, None) => quote!(
            ::cached_projection::DeserializeAnyJsonWrapper<#ty>
        ),
        (Projection::Wrap, Some(lifetime)) => quote!(
            ::cached_projection::DeserializeAnyJsonWrapperRef<#lifetime, #ty>
        ),
    }
}

fn definition_attributes(
    attributes: &[Attribute],
) -> syn::Result<Vec<Attribute>> {
    let mut filtered = Vec::new();
    for attribute in attributes {
        if attribute.path().is_ident("cfg") {
            filtered.push(attribute.clone());
        } else if attribute.path().is_ident("serde") {
            if serde_has_skip(attribute)? {
                filtered.push(parse_quote!(#[serde(skip)]));
            }
        } else if attribute.path().is_ident("cfg_attr") {
            if let Some(attribute) = filter_cfg_attr(attribute, true)? {
                filtered.push(attribute);
            }
        }
    }
    Ok(filtered)
}

fn structural_attributes(
    attributes: &[Attribute],
) -> syn::Result<Vec<Attribute>> {
    let mut filtered = Vec::new();
    for attribute in attributes {
        if attribute.path().is_ident("cfg") {
            filtered.push(attribute.clone());
        } else if attribute.path().is_ident("cfg_attr") {
            if let Some(attribute) = filter_cfg_attr(attribute, false)? {
                filtered.push(attribute);
            }
        }
    }
    Ok(filtered)
}

fn serde_has_skip(attribute: &Attribute) -> syn::Result<bool> {
    let Meta::List(list) = &attribute.meta else {
        return Ok(false);
    };
    let items =
        list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
    Ok(items
        .iter()
        .any(|item| matches!(item, Meta::Path(path) if path.is_ident("skip"))))
}

fn filter_cfg_attr(
    attribute: &Attribute,
    include_serde_skip: bool,
) -> syn::Result<Option<Attribute>> {
    let Meta::List(list) = &attribute.meta else {
        return Ok(None);
    };
    let mut items = list
        .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?
        .into_iter();
    let Some(condition) = items.next() else {
        return Ok(None);
    };
    let mut retained = Vec::new();
    for item in items {
        if item.path().is_ident("cfg") {
            retained.push(item);
        } else if include_serde_skip && item.path().is_ident("serde") {
            let attribute: Attribute = parse_quote!(#[#item]);
            if serde_has_skip(&attribute)? {
                retained.push(parse_quote!(serde(skip)));
            }
        }
    }
    if retained.is_empty() {
        Ok(None)
    } else {
        Ok(Some(parse_quote!(#[cfg_attr(#condition, #(#retained),*)])))
    }
}

fn project_ref_body(
    data: &Data,
    projected_name: &syn::Ident,
    projections: &[Projection],
) -> syn::Result<TokenStream2> {
    match data {
        Data::Struct(data) => struct_projection_body(
            data,
            projected_name,
            projections,
            Conversion::ProjectRef,
        ),
        Data::Enum(data) => enum_projection_body(
            data,
            projected_name,
            projections,
            Conversion::ProjectRef,
        ),
        Data::Union(_) => unreachable!(),
    }
}

fn into_projected_body(
    data: &Data,
    projected_name: &syn::Ident,
    projections: &[Projection],
) -> syn::Result<TokenStream2> {
    match data {
        Data::Struct(data) => struct_projection_body(
            data,
            projected_name,
            projections,
            Conversion::IntoProjected,
        ),
        Data::Enum(data) => enum_projection_body(
            data,
            projected_name,
            projections,
            Conversion::IntoProjected,
        ),
        Data::Union(_) => unreachable!(),
    }
}

fn from_projected_body(
    data: &Data,
    projected_name: &syn::Ident,
    projections: &[Projection],
) -> syn::Result<TokenStream2> {
    match data {
        Data::Struct(data) => {
            struct_from_projected_body(data, projected_name, projections)
        }
        Data::Enum(data) => {
            enum_from_projected_body(data, projected_name, projections)
        }
        Data::Union(_) => unreachable!(),
    }
}

#[derive(Clone, Copy)]
enum Conversion {
    ProjectRef,
    IntoProjected,
}

fn struct_projection_body(
    data: &DataStruct,
    projected_name: &syn::Ident,
    projections: &[Projection],
    conversion: Conversion,
) -> syn::Result<TokenStream2> {
    let bindings = field_bindings(&data.fields);
    let pattern = fields_pattern(&data.fields, &bindings, None)?;
    let target = parse_quote!(#projected_name);
    let values = data
        .fields
        .iter()
        .zip(bindings.iter())
        .zip(projections)
        .map(|((field, binding), projection)| {
            let attributes = structural_attributes(&field.attrs)?;
            let value = conversion_expression(binding, *projection, conversion);
            let ident = &field.ident;
            Ok(if let Some(ident) = ident {
                quote!(#(#attributes)* #ident: #value)
            } else {
                quote!(#(#attributes)* #value)
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;
    let construction = fields_construction(&data.fields, &target, &values);

    Ok(quote! {
        let #pattern = self;
        #construction
    })
}

fn struct_from_projected_body(
    data: &DataStruct,
    projected_name: &syn::Ident,
    projections: &[Projection],
) -> syn::Result<TokenStream2> {
    let bindings = field_bindings(&data.fields);
    let source = parse_quote!(#projected_name);
    let pattern = fields_pattern(&data.fields, &bindings, Some(&source))?;
    let values = data
        .fields
        .iter()
        .zip(bindings.iter())
        .zip(projections)
        .map(|((field, binding), projection)| {
            let attributes = structural_attributes(&field.attrs)?;
            let value = from_expression(binding, &field.ty, *projection);
            let ident = &field.ident;
            Ok(if let Some(ident) = ident {
                quote!(#(#attributes)* #ident: #value)
            } else {
                quote!(#(#attributes)* #value)
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;
    let construction =
        fields_construction(&data.fields, &parse_quote!(Self), &values);

    Ok(quote! {
        let #pattern = projected;
        #construction
    })
}

fn enum_projection_body(
    data: &DataEnum,
    projected_name: &syn::Ident,
    projections: &[Projection],
    conversion: Conversion,
) -> syn::Result<TokenStream2> {
    let mut offset = 0;
    let arms = data
        .variants
        .iter()
        .map(|variant| {
            let count = variant.fields.len();
            let variant_projections = &projections[offset..offset + count];
            offset += count;
            let attributes = structural_attributes(&variant.attrs)?;
            let bindings = field_bindings(&variant.fields);
            let variant_ident = &variant.ident;
            let pattern = fields_pattern(
                &variant.fields,
                &bindings,
                Some(&parse_quote!(Self::#variant_ident)),
            )?;
            let values = variant
                .fields
                .iter()
                .zip(bindings.iter())
                .zip(variant_projections)
                .map(|((field, binding), projection)| {
                    let attributes = structural_attributes(&field.attrs)?;
                    let value =
                        conversion_expression(binding, *projection, conversion);
                    let ident = &field.ident;
                    Ok(if let Some(ident) = ident {
                        quote!(#(#attributes)* #ident: #value)
                    } else {
                        quote!(#(#attributes)* #value)
                    })
                })
                .collect::<syn::Result<Vec<_>>>()?;
            let target = parse_quote!(#projected_name::#variant_ident);
            let construction =
                fields_construction(&variant.fields, &target, &values);
            Ok(quote!(#(#attributes)* #pattern => #construction))
        })
        .collect::<syn::Result<Vec<_>>>()?;

    Ok(quote!(match self { #(#arms),* }))
}

fn enum_from_projected_body(
    data: &DataEnum,
    projected_name: &syn::Ident,
    projections: &[Projection],
) -> syn::Result<TokenStream2> {
    let mut offset = 0;
    let arms = data
        .variants
        .iter()
        .map(|variant| {
            let count = variant.fields.len();
            let variant_projections = &projections[offset..offset + count];
            offset += count;
            let attributes = structural_attributes(&variant.attrs)?;
            let bindings = field_bindings(&variant.fields);
            let variant_ident = &variant.ident;
            let source = parse_quote!(#projected_name::#variant_ident);
            let pattern =
                fields_pattern(&variant.fields, &bindings, Some(&source))?;
            let values = variant
                .fields
                .iter()
                .zip(bindings.iter())
                .zip(variant_projections)
                .map(|((field, binding), projection)| {
                    let attributes = structural_attributes(&field.attrs)?;
                    let value =
                        from_expression(binding, &field.ty, *projection);
                    let ident = &field.ident;
                    Ok(if let Some(ident) = ident {
                        quote!(#(#attributes)* #ident: #value)
                    } else {
                        quote!(#(#attributes)* #value)
                    })
                })
                .collect::<syn::Result<Vec<_>>>()?;
            let target = parse_quote!(Self::#variant_ident);
            let construction =
                fields_construction(&variant.fields, &target, &values);
            Ok(quote!(#(#attributes)* #pattern => #construction))
        })
        .collect::<syn::Result<Vec<_>>>()?;

    Ok(quote!(match projected { #(#arms),* }))
}

fn field_bindings(fields: &Fields) -> Vec<syn::Ident> {
    fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            field
                .ident
                .clone()
                .unwrap_or_else(|| format_ident!("field_{index}"))
        })
        .collect()
}

fn fields_pattern(
    fields: &Fields,
    bindings: &[syn::Ident],
    prefix: Option<&syn::Path>,
) -> syn::Result<TokenStream2> {
    let prefix = prefix
        .map(|prefix| quote!(#prefix))
        .unwrap_or_else(|| quote!(Self));
    let entries = fields
        .iter()
        .zip(bindings)
        .map(|(field, binding)| {
            let attributes = structural_attributes(&field.attrs)?;
            let ident = &field.ident;
            Ok(if let Some(ident) = ident {
                quote!(#(#attributes)* #ident)
            } else {
                quote!(#(#attributes)* #binding)
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;

    Ok(match fields {
        Fields::Named(_) => quote!(#prefix { #(#entries),* }),
        Fields::Unnamed(_) => quote!(#prefix( #(#entries),* )),
        Fields::Unit => quote!(#prefix),
    })
}

fn fields_construction(
    fields: &Fields,
    target: &syn::Path,
    values: &[TokenStream2],
) -> TokenStream2 {
    match fields {
        Fields::Named(_) => quote!(#target { #(#values),* }),
        Fields::Unnamed(_) => quote!(#target( #(#values),* )),
        Fields::Unit => quote!(#target),
    }
}

fn conversion_expression(
    binding: &syn::Ident,
    projection: Projection,
    conversion: Conversion,
) -> TokenStream2 {
    match (projection, conversion) {
        (Projection::Ordinary, _) => quote!(#binding),
        (Projection::Nested, Conversion::ProjectRef) => quote!(
            ::cached_projection::CachedProjection::project_ref(#binding)
        ),
        (Projection::Nested, Conversion::IntoProjected) => quote!(
            ::cached_projection::CachedProjection::into_projected(#binding)
        ),
        (Projection::Wrap, Conversion::ProjectRef) => quote!(
            ::cached_projection::DeserializeAnyJsonWrapperRef(#binding)
        ),
        (Projection::Wrap, Conversion::IntoProjected) => quote!(
            ::cached_projection::DeserializeAnyJsonWrapper(#binding)
        ),
    }
}

fn from_expression(
    binding: &syn::Ident,
    ty: &Type,
    projection: Projection,
) -> TokenStream2 {
    match projection {
        Projection::Ordinary => quote!(#binding),
        Projection::Nested => quote!(
            <#ty as ::cached_projection::CachedProjection>::from_projected(#binding)
        ),
        Projection::Wrap => quote!(#binding.0),
    }
}
