use proc_macro::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use std::collections::HashMap;
use std::mem;
use syn::parse::{Parse, ParseStream};
use syn::{parenthesized, Attribute, Data, DeriveInput, Error, Fields, Ident, LitStr, Member, Result, Token, Variant};

pub fn generate_impls(input: DeriveInput) -> Result<TokenStream> {
    let enum_name = input.ident;
    let i18n_root_key = find_i18n_root_key(&enum_name, input.attrs)?;
    let Data::Enum(enum_data) = input.data else {
        return Err(Error::new(
            enum_name.span(),
            "I18nEnum only supports enums. Please place the macro on the underlying error type.",
        ));
    };

    let variants = parse_variants(enum_data.variants)?;

    let translation_id_cases = variants.iter().map(|variant| {
        let name = &variant.name;
        let pattern_format = variant.pattern_format;
        let id = variant.translation_id.as_ref().unwrap();
        quote! {
            Self::#name #pattern_format => #id,
        }
    });

    let message_cases = variants.iter().map(|variant| {
        let name = &variant.name;
        let pattern_format = variant.pattern_format;
        let message_key = format!("{i18n_root_key}.{}", variant.translation_id.as_ref().unwrap().value());
        let params = variant.translate_fields
            .as_ref()
            .unwrap()
            .0
            .iter()
            .map(|(param_name, field)| quote! { #param_name = #field });
        quote! {
            x @ Self::#name #pattern_format =>
                ::rust_i18n::t!(#message_key, locale = locale, #(#params),*),
        }
    });

    let result = quote! {
        impl ::ariadne::i18n::I18nEnum for #enum_name {
            fn translation_id(&self) -> &'static str {
                match self {
                    #(#translation_id_cases)*
                }
            }

            fn translated_message(&self, locale: &str) -> ::std::borrow::Cow<'_, str> {
                match self {
                    #(#message_cases)*
                }
            }
        }

        impl ::std::fmt::Display for #enum_name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}", self.translated_message("en"))
            }
        }
    }
    .into();

    Ok(result)
}

fn find_i18n_root_key(enum_name: &Ident, attrs: Vec<Attribute>) -> Result<String> {
    for attr in attrs {
        if attr.path().is_ident("i18n_root_key") {
            let key: LitStr = attr.parse_args()?;
            return Ok(key.value());
        }
    }
    Err(Error::new(enum_name.span(), "Missing #[i18n_root_key] attribute"))
}

fn parse_variants(
    variants: impl IntoIterator<Item = Variant>,
) -> Result<Vec<VariantData>> {
    let mut result = variants.into_iter()
        .map(VariantData::parse)
        .collect();
    validate_variants(&mut result)?;
    Ok(result)
}

struct VariantData {
    name: Ident,
    pattern_format: PatternFormat,
    translation_id: Result<LitStr>,
    translate_fields: Result<TranslateFields>,
}

impl VariantData {
    fn parse(variant: Variant) -> Self {
        let name = variant.ident;
        let mut translation_id = None;
        let mut translate_fields = None;
        for attr in variant.attrs {
            if attr.path().is_ident("translation_id") {
                translation_id = Some(attr.parse_args());
            } else if attr.path().is_ident("translate_fields") {
                translate_fields = Some(attr.parse_args());

            }
        }
        Self {
            translation_id: translation_id.unwrap_or_else(|| Err(Error::new(
                name.span(),
                format!("Missing #[translation_id] for variant {}", name),
            ))),
            translate_fields: translate_fields.unwrap_or_else(|| Ok(Default::default())),

            name,
            pattern_format: PatternFormat::from_variant(&variant.fields),
        }
    }
}

#[derive(Copy, Clone)]
enum PatternFormat {
    Named,
    Tuple,
    Unit,
}

impl PatternFormat {
    fn from_variant(fields: &Fields) -> Self {
        match fields {
            Fields::Named(_) => Self::Named,
            Fields::Unnamed(_) => Self::Tuple,
            Fields::Unit => Self::Unit,
        }
    }
}

impl ToTokens for PatternFormat {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            PatternFormat::Named => tokens.append_all(quote! { { .. } }),
            PatternFormat::Tuple => tokens.append_all(quote! { (..) }),
            PatternFormat::Unit => {}
        }
    }
}

#[derive(Default)]
struct TranslateFields(HashMap<Ident, TranslateField>);

impl Parse for TranslateFields {
    fn parse(input: ParseStream) -> Result<Self> {
        let result = input.parse_terminated(|input| {
            let key = input.parse::<Ident>()?;
            input.parse::<Token![=]>()?;
            let value = input.parse::<TranslateField>()?;
            Ok((key, value))
        }, Token![,])?;
        Ok(Self(result.into_iter().collect()))
    }
}

struct TranslateField {
    member: Member,
    translated: bool,
}

impl ToTokens for TranslateField {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let member = &self.member;
        let new_tokens = if self.translated {
            quote! {
                ::ariadne::i18n::I18nEnum::translated_message(&self.#member, locale)
            }
        } else {
            quote! {
                self.#member
            }
        };
        tokens.append_all(new_tokens);
    }
}

impl Parse for TranslateField {
    fn parse(input: ParseStream) -> Result<Self> {
        mod kw {
            syn::custom_keyword!(translate);
        }
        let translated = input.peek(kw::translate);
        let member = if translated {
            input.parse::<kw::translate>()?;
            let content;
            parenthesized!(content in input);
            content.parse()
        } else {
            input.parse()
        }?;
        Ok(Self {
            member,
            translated,
        })
    }
}

fn validate_variants(variants: &mut Vec<VariantData>) -> Result<()> {
    fn take_err<T>(variant_name: &Ident, value: &mut Result<T>) -> Option<Error> {
        match value {
            Ok(_) => None,
            Err(e) => Some(mem::replace(e, Error::new(variant_name.span(), ""))),
        }
    }

    variants
        .iter_mut()
        .flat_map(|x| vec![
            take_err(&x.name, &mut x.translation_id),
            take_err(&x.name, &mut x.translate_fields),
        ])
        .filter_map(|x| x)
        .reduce(|mut a, b| {
            a.extend(b);
            a
        })
        .map_or(Ok(()), Err)
}
