use std::collections::BTreeMap;

pub use ::cel::Context;
use ::cel::{ExecutionError, Program as CelProgram, Value};
use chumsky::{Parser, prelude::*};
use eyre::{Result, WrapErr, eyre};
use thiserror::Error;

const MAX_PREPROCESSED_SIZE: usize = 1_048_576;

pub struct Program {
    inner: CelProgram,
}

impl Program {
    pub fn compile(source: &str) -> Result<Self> {
        let source = preprocess(source)
            .wrap_err("failed to preprocess cel expression")?;
        let inner =
            CelProgram::compile(&source).map_err(|error| eyre!(error))?;

        Ok(Self { inner })
    }

    pub fn execute<'a>(
        &self,
        context: &Context<'a>,
    ) -> std::result::Result<Value, ExecutionError> {
        self.inner.execute(context)
    }
}

#[derive(Debug)]
struct Definition {
    name: String,
    replacement: String,
}

#[derive(Debug, Error)]
enum PreprocessorError {
    #[error("invalid `#define` on line {line}, column {column}: {message}")]
    InvalidDefinition {
        line: usize,
        column: usize,
        message: String,
    },
    #[error("macro `{name}` is defined more than once")]
    DuplicateDefinition { name: String },
    #[error("recursive macro expansion: {path}")]
    RecursiveExpansion { path: String },
    #[error("preprocessed expression exceeds {MAX_PREPROCESSED_SIZE} bytes")]
    ExpressionTooLarge,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum LexState {
    #[default]
    Normal,
    String {
        quote: u8,
        triple: bool,
        raw: bool,
    },
    BlockComment,
}

fn definition_parser() -> impl Parser<char, Definition, Error = Simple<char>> {
    let horizontal_whitespace = one_of(" \t").repeated();
    let identifier = filter(|character: &char| {
        character.is_ascii_alphabetic() || *character == '_'
    })
    .then(
        filter(|character: &char| {
            character.is_ascii_alphanumeric() || *character == '_'
        })
        .repeated(),
    )
    .map(|(first, rest)| {
        std::iter::once(first).chain(rest).collect::<String>()
    });
    let replacement = any().repeated().at_least(1).collect::<String>();

    horizontal_whitespace
        .clone()
        .ignore_then(just('#'))
        .then_ignore(horizontal_whitespace.clone())
        .then_ignore(just("define"))
        .then_ignore(horizontal_whitespace.clone().at_least(1))
        .ignore_then(identifier)
        .then_ignore(horizontal_whitespace.at_least(1))
        .then(replacement)
        .map(|(name, replacement)| Definition {
            name,
            replacement: replacement.trim().to_string(),
        })
        .then_ignore(end())
}

fn preprocess(source: &str) -> Result<String, PreprocessorError> {
    let mut definitions = BTreeMap::new();
    let mut expression = String::with_capacity(source.len());
    let mut state = LexState::Normal;

    for (line_index, line) in source.split_inclusive('\n').enumerate() {
        let directive = state == LexState::Normal
            && line.trim_start_matches([' ', '\t']).starts_with('#');

        if directive {
            let directive = line.trim_end_matches(['\r', '\n']);
            let definition =
                definition_parser().parse(directive).map_err(|errors| {
                    let error =
                        errors.into_iter().next().unwrap_or_else(|| {
                            Simple::custom(0..0, "invalid directive")
                        });
                    PreprocessorError::InvalidDefinition {
                        line: line_index + 1,
                        column: error.span().start + 1,
                        message: error.to_string(),
                    }
                })?;

            if definitions
                .insert(definition.name.clone(), definition.replacement)
                .is_some()
            {
                return Err(PreprocessorError::DuplicateDefinition {
                    name: definition.name,
                });
            }

            if line.ends_with('\n') {
                expression.push('\n');
            }
        } else {
            expression.push_str(line);
            advance_lex_state(line, &mut state);
        }
    }

    let mut expanded = String::with_capacity(expression.len());
    expand(&expression, &definitions, &mut Vec::new(), &mut expanded)?;
    Ok(expanded)
}

fn expand(
    source: &str,
    definitions: &BTreeMap<String, String>,
    stack: &mut Vec<String>,
    output: &mut String,
) -> Result<(), PreprocessorError> {
    let bytes = source.as_bytes();
    let mut index = 0;
    let mut state = LexState::Normal;

    while index < bytes.len() {
        match state {
            LexState::Normal => {
                if bytes[index..].starts_with(b"//") {
                    push(output, "//")?;
                    index += 2;
                    while index < bytes.len() {
                        let character = source[index..]
                            .chars()
                            .next()
                            .expect("the index is within the string");
                        let mut encoded = [0; 4];
                        push(output, character.encode_utf8(&mut encoded))?;
                        index += character.len_utf8();
                        if character == '\n' {
                            break;
                        }
                    }
                    continue;
                }

                if bytes[index..].starts_with(b"/*") {
                    push(output, "/*")?;
                    index += 2;
                    state = LexState::BlockComment;
                    continue;
                }

                if let Some((length, string_state)) = string_start(bytes, index)
                {
                    push(output, &source[index..index + length])?;
                    index += length;
                    state = string_state;
                    continue;
                }

                if bytes[index].is_ascii_alphabetic() || bytes[index] == b'_' {
                    let start = index;
                    index += 1;
                    while index < bytes.len()
                        && (bytes[index].is_ascii_alphanumeric()
                            || bytes[index] == b'_')
                    {
                        index += 1;
                    }

                    let identifier = &source[start..index];
                    if let Some(replacement) = definitions.get(identifier) {
                        if let Some(cycle_start) =
                            stack.iter().position(|name| name == identifier)
                        {
                            let mut path = stack[cycle_start..].to_vec();
                            path.push(identifier.to_string());
                            return Err(
                                PreprocessorError::RecursiveExpansion {
                                    path: path.join(" -> "),
                                },
                            );
                        }

                        stack.push(identifier.to_string());
                        expand(replacement, definitions, stack, output)?;
                        stack.pop();
                    } else {
                        push(output, identifier)?;
                    }
                    continue;
                }
            }
            LexState::String { quote, triple, raw } => {
                if !raw && bytes[index] == b'\\' {
                    let end = (index + 2).min(bytes.len());
                    push(output, &source[index..end])?;
                    index = end;
                    continue;
                }

                let closing_length = if triple { 3 } else { 1 };
                if bytes[index] == quote
                    && bytes[index..].len() >= closing_length
                    && bytes[index..index + closing_length]
                        .iter()
                        .all(|character| *character == quote)
                {
                    push(output, &source[index..index + closing_length])?;
                    index += closing_length;
                    state = LexState::Normal;
                    continue;
                }
            }
            LexState::BlockComment => {
                if bytes[index..].starts_with(b"*/") {
                    push(output, "*/")?;
                    index += 2;
                    state = LexState::Normal;
                    continue;
                }
            }
        }

        let character = source[index..]
            .chars()
            .next()
            .expect("the index is within the string");
        let mut encoded = [0; 4];
        push(output, character.encode_utf8(&mut encoded))?;
        index += character.len_utf8();
    }

    Ok(())
}

fn push(output: &mut String, value: &str) -> Result<(), PreprocessorError> {
    if output.len().saturating_add(value.len()) > MAX_PREPROCESSED_SIZE {
        return Err(PreprocessorError::ExpressionTooLarge);
    }
    output.push_str(value);
    Ok(())
}

fn advance_lex_state(source: &str, state: &mut LexState) {
    let bytes = source.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        match *state {
            LexState::Normal => {
                if bytes[index..].starts_with(b"//") {
                    return;
                }
                if bytes[index..].starts_with(b"/*") {
                    *state = LexState::BlockComment;
                    index += 2;
                    continue;
                }
                if let Some((length, string_state)) = string_start(bytes, index)
                {
                    *state = string_state;
                    index += length;
                    continue;
                }
            }
            LexState::String { quote, triple, raw } => {
                if !raw && bytes[index] == b'\\' {
                    index = (index + 2).min(bytes.len());
                    continue;
                }

                let closing_length = if triple { 3 } else { 1 };
                if bytes[index] == quote
                    && bytes[index..].len() >= closing_length
                    && bytes[index..index + closing_length]
                        .iter()
                        .all(|character| *character == quote)
                {
                    *state = LexState::Normal;
                    index += closing_length;
                    continue;
                }
            }
            LexState::BlockComment => {
                if bytes[index..].starts_with(b"*/") {
                    *state = LexState::Normal;
                    index += 2;
                    continue;
                }
            }
        }

        index += source[index..]
            .chars()
            .next()
            .expect("the index is within the string")
            .len_utf8();
    }

    if matches!(state, LexState::String { triple: false, .. }) {
        *state = LexState::Normal;
    }
}

fn string_start(bytes: &[u8], index: usize) -> Option<(usize, LexState)> {
    let remaining = &bytes[index..];
    let (prefix_length, raw) = if remaining.len() >= 3
        && matches!(remaining[0].to_ascii_lowercase(), b'b' | b'r')
        && matches!(remaining[1].to_ascii_lowercase(), b'b' | b'r')
        && remaining[0].to_ascii_lowercase()
            != remaining[1].to_ascii_lowercase()
    {
        (2, true)
    } else if remaining.len() >= 2
        && matches!(remaining[0].to_ascii_lowercase(), b'b' | b'r')
    {
        (1, remaining[0].eq_ignore_ascii_case(&b'r'))
    } else {
        (0, false)
    };

    let quote = *remaining.get(prefix_length)?;
    if !matches!(quote, b'\'' | b'"') {
        return None;
    }

    let triple =
        remaining
            .get(prefix_length..prefix_length + 3)
            .is_some_and(|quotes| {
                quotes.iter().all(|character| *character == quote)
            });
    let quote_length = if triple { 3 } else { 1 };

    Some((
        prefix_length + quote_length,
        LexState::String { quote, triple, raw },
    ))
}

#[cfg(test)]
mod tests {
    use super::preprocess;

    #[test]
    fn expands_object_macros() {
        let source =
            "#define ISSUE OBFUSCATED_NAMES\ntrace.issue_type == ISSUE";
        assert_eq!(
            preprocess(source).unwrap(),
            "\ntrace.issue_type == OBFUSCATED_NAMES"
        );
    }

    #[test]
    fn recursively_expands_macros() {
        let source =
            "#define RESULT SEVERITY\n#define SEVERITY \"low\"\nRESULT";
        assert_eq!(preprocess(source).unwrap(), "\n\n\"low\"");
    }

    #[test]
    fn does_not_expand_strings_or_comments() {
        let source =
            "#define VALUE expanded\nVALUE == \"VALUE\" // VALUE\n/* VALUE */";
        assert_eq!(
            preprocess(source).unwrap(),
            "\nexpanded == \"VALUE\" // VALUE\n/* VALUE */"
        );
    }

    #[test]
    fn reports_recursive_macros() {
        let source = "#define FIRST SECOND\n#define SECOND FIRST\nFIRST";
        let error = preprocess(source).unwrap_err();
        assert_eq!(
            error.to_string(),
            "recursive macro expansion: FIRST -> SECOND -> FIRST"
        );
    }
}
