use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned, IdentFragment, ToTokens, TokenStreamExt};
use std::collections::HashMap;
use std::mem;
use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::{parenthesized, Attribute, Data, DeriveInput, Error, Fields, Ident, Index, LitStr, Member, Result, Token, Variant};

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
        let pattern_format = &variant.pattern_format;
        let id = variant.translation_id.as_ref().unwrap();
        quote! {
            Self::#name #pattern_format => #id,
        }
    });

    let message_cases = variants.iter().map(|variant| {
        let name = &variant.name;
        let pattern_format = &variant.pattern_format;
        let message_key = format!("{i18n_root_key}.{}", variant.translation_id.as_ref().unwrap().value());
        let params = variant.translate_fields
            .as_ref()
            .unwrap()
            .0
            .iter()
            .map(|(param_name, field)| quote! { #param_name = #field });
        quote! {
            Self::#name #pattern_format =>
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
                f.write_str(&self.translated_message("en"))
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
            pattern_format: PatternFormat::from_fields(variant.fields),
        }
    }
}

enum PatternFormat {
    Named(Vec<Ident>),
    Tuple(usize),
    Unit,
}

impl PatternFormat {
    fn from_fields(fields: Fields) -> Self {
        match fields {
            Fields::Named(named) => Self::Named(
                named.named.into_iter().map(|x| x.ident.unwrap()).collect()
            ),
            Fields::Unnamed(unnamed) => Self::Tuple(unnamed.unnamed.len()),
            Fields::Unit => Self::Unit,
        }
    }
}

impl ToTokens for PatternFormat {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            PatternFormat::Named(names) => tokens.append_all(quote! {
                { #(#names),* }
            }),
            PatternFormat::Tuple(length) => {
                let names = (0..*length).map(|x| format_ident!("_{x}"));
                tokens.append_all(quote! {
                    ( #(#names),* )
                });
            },
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
    span: Span,
}

impl ToTokens for TranslateField {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let member = match &self.member {
            Member::Named(ident) => ident.to_token_stream(),
            Member::Unnamed(Index { index, span }) => format_ident!("_{index}", span = *span).to_token_stream(),
        };
        let span = self.span;
        if self.translated {
            tokens.append_all(quote_spanned! {span=>
                ::ariadne::i18n::I18nEnum::translated_message(&#member, locale)
            });
        } else {
            member.to_tokens(tokens)
        }
    }
}

impl Parse for TranslateField {
    fn parse(input: ParseStream) -> Result<Self> {
        mod kw {
            syn::custom_keyword!(translate);
        }
        let translated = input.peek(kw::translate);
        let member: Member;
        let span: Span;
        if translated {
            let keyword = input.parse::<kw::translate>()?;
            let content;
            let parens = parenthesized!(content in input);
            member = content.parse()?;
            span = keyword.span.join(parens.span.join())
                .or_else(|| member.span())
                .unwrap();
        } else {
            member = input.parse()?;
            span = member.span().unwrap();
        }
        Ok(Self {
            member,
            translated,
            span,
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
