use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::visit::Visit;
use syn::{
    Attribute, File, Ident, ItemEnum, ItemMod, LitStr, Macro, Meta, Token,
    Type, braced, bracketed, parenthesized, token, visit,
};
use walkdir::{DirEntry, WalkDir};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct TranslationEntry {
    pub message: String,
}

impl TranslationEntry {
    pub fn new(message: String) -> Self {
        Self { message }
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
            path: file.path(),
            source: &file_contents,
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
}

struct FileExtractor<'a> {
    extractor: &'a mut Extractor,
    path: &'a Path,
    source: &'a str,
    enum_messages: HashMap<Ident, HashMap<Ident, LitStr>>,
}

impl FileExtractor<'_> {
    fn add_error(&mut self, error: syn::Error) {
        self.extractor.errors.push(syn_miette::Error::new_named(
            error,
            self.source,
            self.path.display(),
        ));
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
        for variant in &i.variants {
            let error_message = variant.attrs.iter().find_map(|x| {
                if !x.path().is_ident("error") {
                    return None;
                }
                x.parse_args().ok()
            });
            if let Some(error_message) = error_message {
                variants.insert(variant.ident.clone(), error_message);
            }
        }
        if !variants.is_empty() {
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
                let messages = self.enum_messages.get(&for_enum);
                let root_key = root_key.value();
                for variant in variants {
                    if variant.transparent.is_some() {
                        continue;
                    }
                    self.extractor.output.insert(
                        format!("{}.{}", root_key, variant.key.value()),
                        messages
                            .and_then(|x| x.get(&variant.variant_name))
                            .map(LitStr::value)
                            .map(|x| variant.transform_format_string(x))
                            .map(TranslationEntry::new)
                            .unwrap_or_default(),
                    );
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
        let fields_type;
        let fields = if input.peek(Token![!]) {
            fields_type = FieldsType::Unit;
            input.parse::<Token![!]>()?;
            Punctuated::new()
        } else {
            let content;
            if input.peek(token::Brace) {
                fields_type = FieldsType::Named;
                braced!(content in input);
                if content.peek(Token![..]) {
                    content.parse::<Token![..]>()?;
                    Punctuated::new()
                } else {
                    content.parse_terminated(Ident::parse, Token![,])?
                }
            } else {
                fields_type = FieldsType::Tuple;
                parenthesized!(content in input);
                if content.peek(Token![..]) {
                    content.parse::<Token![..]>()?;
                    Punctuated::new()
                } else if content.peek(kw::transparent) {
                    transparent = Some(content.parse()?);
                    content.parse::<Ident>()?;
                    Punctuated::new()
                } else {
                    content.parse_terminated(Ident::parse, Token![,])?
                }
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
    fn transform_format_string(&self, format_string: String) -> String {
        if !format_string.contains('{') || self.fields.is_empty() {
            return format_string;
        }

        let name_transforms = if matches!(self.fields_type, FieldsType::Tuple) {
            self.fields.iter().map(Ident::to_string).collect()
        } else {
            vec![]
        };

        let mut result = String::new();

        let mut prev_push_index = 0;
        let mut format_start = None;
        let mut extra_format_layers = 0usize;
        let mut format_index = 0;
        let mut iter = format_string.bytes().enumerate();
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
                if matches!(iter.next(), Some((_, b'{'))) {
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
                    let format_variable = name_transforms
                        .get(format_index)
                        .map_or(format_variable, String::as_str);
                    result.push_str(format_variable);
                    format_index += 1;
                    format_start = None;
                    prev_push_index = index;
                    continue;
                } else if matches!(iter.next(), Some((_, b'}'))) {
                    result.push_str(&format_string[prev_push_index..index]);
                    result.push_str("'}'");
                    prev_push_index = index + 2;
                    continue;
                }
            }
        }
        result.push_str(&format_string[prev_push_index..]);

        result
    }
}

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
