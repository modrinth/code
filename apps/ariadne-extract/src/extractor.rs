use crate::error::Result;
use serde::{Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::collections::btree_map::Entry;
use std::fs;
use std::path::Path;
use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{
    Attribute, File, Ident, ItemEnum, ItemMod, LitStr, Macro, Meta, Token,
    Type, braced, bracketed, parenthesized, token, visit,
};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Serialize)]
pub struct TranslationEntry {
    pub message: String,
    #[serde(skip)]
    key_span: Span,
}

impl TranslationEntry {
    pub fn new(message: String, key_span: Span) -> Self {
        Self { message, key_span, }
    }
}

pub struct Extractor {
    include_tests: bool,
    output: BTreeMap<String, TranslationEntry>,
    errors: Vec<syn_miette::Error>,
}

impl Extractor {
    pub fn new(include_tests: bool) -> Self {
        Self {
            output: BTreeMap::new(),
            include_tests,
            errors: vec![],
        }
    }

    pub fn output(&self) -> &BTreeMap<String, TranslationEntry> {
        &self.output
    }

    pub fn errors(&self) -> &Vec<syn_miette::Error> {
        &self.errors
    }
}

impl Extractor {
    pub fn process_package(&mut self, package_path: &Path) -> Result<()> {
        for file in WalkDir::new(package_path.join("src")).min_depth(1) {
            let result = self.process_file(file);
            result?;
        }
        Ok(())
    }

    fn process_file(&mut self, file: walkdir::Result<DirEntry>) -> Result<()> {
        let file = file?;
        if !file.file_type().is_file()
            || file.path().extension().is_none_or(|x| x != "rs")
        {
            return Ok(());
        }
        let file_contents = fs::read_to_string(file.path())?;
        let mut file_extractor = FileExtractor {
            extractor: self,
            file: FileInfo {
                path: file.path(),
                source: &file_contents,
            },
            enum_messages: HashMap::new(),
        };
        let parsed = match syn::parse_file(&file_contents) {
            Ok(file) => file,
            Err(err) => {
                file_extractor.add_error(err);
                return Ok(());
            }
        };
        file_extractor.visit_file(&parsed);
        Ok(())
    }

    fn include_item(&self, attrs: &[Attribute]) -> bool {
        fn include_from_meta(meta: &Meta, include_tests: bool) -> bool {
            if meta.path().is_ident("test") {
                include_tests
            } else if meta.path().is_ident("not") {
                let Ok(list) = meta.require_list() else {
                    return true;
                };
                let Ok(inner) = list.parse_args() else {
                    return true;
                };
                !include_from_meta(&inner, include_tests)
            } else if meta.path().is_ident("all") || meta.path().is_ident("any")
            {
                let Ok(list) = meta.require_list() else {
                    return true;
                };
                let Ok(nested) = list.parse_args_with(
                    Punctuated::<Meta, Token![,]>::parse_terminated,
                ) else {
                    return true;
                };
                if meta.path().is_ident("all") {
                    nested.iter().all(|x| include_from_meta(x, include_tests))
                } else {
                    nested.iter().any(|x| include_from_meta(x, include_tests))
                }
            } else {
                true
            }
        }
        fn include_from_attr(attr: &Attribute, include_tests: bool) -> bool {
            if !attr.path().is_ident("cfg") {
                return true;
            }
            let Ok(inner) = attr.parse_args() else {
                return true;
            };
            include_from_meta(&inner, include_tests)
        }
        attrs
            .iter()
            .all(|x| include_from_attr(x, self.include_tests))
    }

    fn add_error(&mut self, file: &FileInfo, error: syn::Error) {
        self.errors.push(syn_miette::Error::new_named(
            error,
            file.source,
            file.path.display(),
        ));
    }
}

struct FileExtractor<'a> {
    extractor: &'a mut Extractor,
    file: FileInfo<'a>,
    enum_messages: HashMap<Ident, HashMap<Ident, EnumMessage>>,
}

struct FileInfo<'a> {
    path: &'a Path,
    source: &'a str,
}

#[derive(Clone)]
enum EnumMessage {
    Absent {
        variant_span: Span,
    },
    Present {
        message: LitStr,
        display_attribute_type: DisplayAttributeType,
    },
}

impl EnumMessage {
    fn as_option(&self) -> Option<(&LitStr, DisplayAttributeType)> {
        match self {
            Self::Absent { .. } => None,
            Self::Present { message, display_attribute_type } => Some((message, *display_attribute_type)),
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
enum DisplayAttributeType {
    ThiserrorError,
    DeriveMoreDisplay,
}

impl FileExtractor<'_> {
    fn add_error(&mut self, error: syn::Error) {
        self.extractor.add_error(&self.file, error);
    }
}

impl Visit<'_> for FileExtractor<'_> {
    fn visit_file(&mut self, i: &File) {
        if self.extractor.include_item(&i.attrs) {
            visit::visit_file(self, i);
        }
    }

    fn visit_item_enum(&mut self, i: &ItemEnum) {
        let mut variants = HashMap::new();
        let mut has_missing_variants = false;
        let mut ignored_variants = HashSet::new();

        for variant in &i.variants {
            let Some(error_attr) = variant.attrs.iter().find(|x| {
                x.path().is_ident("error") || x.path().is_ident("display")
            }) else {
                has_missing_variants = true;
                continue;
            };
            let display_attribute_type = if error_attr.path().is_ident("error")
            {
                DisplayAttributeType::ThiserrorError
            } else {
                DisplayAttributeType::DeriveMoreDisplay
            };
            let error_message = error_attr
                .parse_args_with(|input: ParseStream| {
                    if display_attribute_type
                        == DisplayAttributeType::ThiserrorError
                        && input.peek(kw::transparent)
                    {
                        input.parse::<kw::transparent>()?;
                        Ok(None)
                    } else {
                        Ok(Some(input.parse::<LitStr>()?))
                    }
                })
                .transpose();
            match error_message {
                Some(Ok(message)) => {
                    variants.insert(
                        variant.ident.clone(),
                        EnumMessage::Present {
                            message,
                            display_attribute_type,
                        },
                    );
                }
                Some(Err(_)) => {
                    self.add_error(syn::Error::new(
                        error_attr.meta.span(),
                        match display_attribute_type {
                            DisplayAttributeType::ThiserrorError => {
                                "only #[error(transparent)] and #[error(\"lone format string\")] syntaxes are supported"
                            }
                            DisplayAttributeType::DeriveMoreDisplay => {
                                "only #[display(\"lone format string\")] syntax is supported"
                            }
                        },
                    ));
                    ignored_variants.insert(variant.ident.clone());
                }
                None => {
                    ignored_variants.insert(variant.ident.clone());
                }
            }
        }
        if !variants.is_empty() {
            if has_missing_variants {
                for variant in &i.variants {
                    if ignored_variants.contains(&variant.ident) {
                        continue;
                    }
                    variants.entry(variant.ident.clone())
                        .or_insert_with(|| EnumMessage::Absent {
                            variant_span: variant.span(),
                        });
                }
            }
            self.enum_messages.insert(i.ident.clone(), variants);
        }
    }

    fn visit_item_mod(&mut self, i: &ItemMod) {
        if self.extractor.include_item(&i.attrs) {
            visit::visit_item_mod(self, i);
        }
    }

    fn visit_macro(&mut self, i: &Macro) {
        if !i.path.is_ident("i18n_enum") {
            return;
        }
        let body = match i.parse_body() {
            Ok(body) => body,
            Err(err) => {
                self.add_error(err);
                return;
            }
        };
        match body {
            I18nEnumMacroInvocation::Transparent => {}
            I18nEnumMacroInvocation::Enum {
                for_enum,
                root_key,
                variants,
            } => {
                let Some(messages) = self.enum_messages.get(&for_enum) else {
                    return;
                };
                let root_key = root_key.value();
                for variant in variants {
                    if variant.transparent.is_some() {
                        continue;
                    }
                    let Some(message_literal) = messages.get(&variant.variant_name) else {
                        continue;
                    };
                    let (message, errors) = message_literal
                        .as_option()
                        .map(|(message, display_attribute_type)| {
                            variant.transform_format_string(
                                message.value(),
                                display_attribute_type,
                            )
                        })
                        .unwrap_or_else(|| ("".into(), vec![format!("no default message specified for variant {}", variant.variant_name)]));
                    let duplicate_key = match self.extractor.output.entry(format!("{}.{}", root_key, variant.key.value())) {
                        Entry::Vacant(entry) => {
                            entry.insert(TranslationEntry::new(message, variant.key.span()));
                            None
                        },
                        Entry::Occupied(entry) => {
                            (*entry.get().message != message).then(|| (entry.key().clone(), entry.get().key_span))
                        }
                    };
                    if let Some((duplicate_key, original_span)) = duplicate_key {
                        let mut error = syn::Error::new(
                            variant.key.span(),
                            format!("duplicate variant key {}", duplicate_key)
                        );
                        error.combine(syn::Error::new(
                            original_span,
                            "originally used here"
                        ));
                        self.extractor.add_error(&self.file, error);
                    }
                    for error in errors {
                        self.extractor.add_error(
                            &self.file,
                            syn::Error::new(
                                match message_literal {
                                    EnumMessage::Absent { variant_span } => *variant_span,
                                    EnumMessage::Present { message, .. } => message.span(),
                                },
                                error,
                            ),
                        );
                    }
                }
            }
        }
    }
}

enum I18nEnumMacroInvocation {
    Transparent,
    Enum {
        for_enum: Ident,
        root_key: LitStr,
        variants: Punctuated<I18nEnumVariant, Token![,]>,
    },
}

impl Parse for I18nEnumMacroInvocation {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(kw::transparent) {
            input.parse::<kw::transparent>()?;
            input.parse::<Ident>()?;
            let field_content;
            bracketed!(field_content in input);
            field_content.parse::<Ident>()?;
            field_content.parse::<Token![:]>()?;
            field_content.parse::<Type>()?;
            return Ok(I18nEnumMacroInvocation::Transparent);
        }

        let for_enum = input.parse()?;
        input.parse::<Token![,]>()?;

        input.parse::<kw::root_key>()?;
        input.parse::<Token![:]>()?;
        let root_key = input.parse()?;
        input.parse::<Token![,]>()?;

        let variants =
            input.parse_terminated(I18nEnumVariant::parse, Token![,])?;

        Ok(I18nEnumMacroInvocation::Enum {
            for_enum,
            root_key,
            variants,
        })
    }
}

struct I18nEnumVariant {
    variant_name: Ident,
    transparent: Option<kw::transparent>,
    fields_type: FieldsType,
    fields: Punctuated<Ident, Token![,]>,
    key: LitStr,
}

impl Parse for I18nEnumVariant {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let variant_name = input.parse()?;

        let mut transparent = None;
        let mut fields = Punctuated::new();
        let fields_type;
        if input.peek(Token![!]) || input.peek(Token![=>]) {
            // Immediate => also flows into this case for a better error message
            fields_type = FieldsType::Unit;
            input.parse::<Token![!]>()?;
        } else {
            let content;
            if input.peek(token::Brace) {
                fields_type = FieldsType::Named;
                braced!(content in input);
            } else {
                fields_type = FieldsType::Tuple;
                parenthesized!(content in input);
            }
            if content.peek(Token![..]) {
                content.parse::<Token![..]>()?;
            } else if content.peek(kw::transparent) {
                transparent = Some(content.parse()?);
                content.parse::<Ident>()?;
            } else if content.peek(Ident) {
                loop {
                    fields.push_value(content.parse()?);
                    if !content.peek(Token![,]) || !content.peek2(Ident) {
                        break;
                    }
                    fields.push_punct(content.parse()?);
                }
            };
            if fields_type == FieldsType::Named && content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
                content.parse::<Token![..]>()?;
            }
        };

        input.parse::<Token![=>]>()?;

        let key = input.parse()?;

        Ok(Self {
            variant_name,
            transparent,
            fields_type,
            fields,
            key,
        })
    }
}

impl I18nEnumVariant {
    fn transform_format_string(
        &self,
        format_string: String,
        display_attribute_type: DisplayAttributeType,
    ) -> (String, Vec<String>) {
        let mut errors = vec![];
        if !format_string.contains(['\'', '{', '}']) {
            return (format_string, errors);
        }

        let known_names = if self.fields_type == FieldsType::Named {
            self.fields.iter().map(Ident::to_string).collect()
        } else {
            HashSet::new()
        };

        let mut result = String::new();

        let mut prev_push_index = 0;
        let mut format_start = None;
        let mut extra_format_layers = 0usize;
        let mut iter = format_string.bytes().enumerate().peekable();
        while let Some((index, char)) = iter.next() {
            if char == b'\'' {
                result.push_str(&format_string[prev_push_index..index + 1]);
                result.push('\'');
                prev_push_index = index + 1;
            }
            if char == b'{' {
                if format_start.is_some() {
                    extra_format_layers += 1;
                    continue;
                }
                result.push_str(&format_string[prev_push_index..index]);
                if matches!(iter.peek(), Some((_, b'{'))) {
                    iter.next();
                    result.push_str("'{'");
                    prev_push_index = index + 2;
                } else {
                    result.push('{');
                    prev_push_index = index + 1;
                    format_start = Some(index + 1);
                }
                continue;
            }
            if char == b'}' {
                if extra_format_layers > 0 {
                    extra_format_layers -= 1;
                    continue;
                }
                if let Some(prev_start) = format_start {
                    let format_variable = &format_string[prev_start..index];
                    let format_variable = match format_variable.split_once(':')
                    {
                        Some((real_variable, _)) => {
                            errors.push("format specifiers not allowed".into());
                            real_variable
                        }
                        None => format_variable,
                    };
                    let format_variable = match self.fields_type {
                        FieldsType::Unit => {
                            errors.push(
                                "formatting not supported for unit variants"
                                    .into(),
                            );
                            format_variable
                        }
                        FieldsType::Tuple => {
                            let format_variable = match display_attribute_type {
                                DisplayAttributeType::ThiserrorError => {
                                    format_variable
                                }
                                DisplayAttributeType::DeriveMoreDisplay => {
                                    match format_variable.strip_prefix('_') {
                                        Some(stripped) => stripped,
                                        None => {
                                            errors.push(format!("index format variable in #[display] must be prefixed with '_': {{_{format_variable}}}"));
                                            format_variable
                                        }
                                    }
                                }
                            };
                            match format_variable
                                .parse()
                                .map(|x| self.fields.get(x))
                            {
                                Ok(Some(field)) => &field.to_string(),
                                Ok(None) => {
                                    errors.push(format!("index format variable {{{format_variable}}} out of bounds"));
                                    format_variable
                                }
                                Err(_) => {
                                    errors.push(format!("invalid index format variable {{{format_variable}}} (must be usize)"));
                                    format_variable
                                }
                            }
                        }
                        FieldsType::Named => {
                            if !known_names.contains(format_variable) {
                                errors.push(format!("unknown format variable {{{format_variable}}}"))
                            }
                            format_variable
                        }
                    };
                    result.push_str(format_variable);
                    format_start = None;
                    prev_push_index = index;
                    continue;
                } else if matches!(iter.peek(), Some((_, b'}'))) {
                    iter.next();
                    result.push_str(&format_string[prev_push_index..index]);
                    result.push_str("'}'");
                    prev_push_index = index + 2;
                    continue;
                } else {
                    errors.push("unmatched } in format string".into());
                }
            }
        }
        result.push_str(&format_string[prev_push_index..]);

        (result, errors)
    }
}

#[derive(Eq, PartialEq)]
enum FieldsType {
    Unit,
    Tuple,
    Named,
}

mod kw {
    use syn::custom_keyword;

    custom_keyword!(transparent);
    custom_keyword!(root_key);
}
