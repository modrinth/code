use std::collections::BTreeMap;

pub use ::cel::Context;
use ::cel::{ExecutionError, Program as CelProgram, Value};
use chumsky::{Parser, prelude::*};
use eyre::{Result, WrapErr, eyre};
use thiserror::Error;

const MAX_PREPROCESSED_SIZE: usize = 1_048_576;

pub struct Program {
    inner: CelProgram,
    bindings: Vec<CompiledBinding>,
}

impl Program {
    pub fn compile(source: &str) -> Result<Self> {
        let preprocessed = preprocess(source)
            .wrap_err("failed to preprocess cel expression")?;
        let bindings = preprocessed
            .bindings
            .into_iter()
            .map(|binding| {
                let inner = CelProgram::compile(&binding.expression)
                    .map_err(|error| eyre!(error))
                    .wrap_err_with(|| {
                        format!(
                            "failed to compile `#bind {}` on line {}",
                            binding.name, binding.line
                        )
                    })?;

                Ok(CompiledBinding {
                    name: binding.name,
                    inner,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let inner = CelProgram::compile(&preprocessed.expression)
            .map_err(|error| eyre!(error))?;

        Ok(Self { inner, bindings })
    }

    pub fn execute<'a>(
        &self,
        context: &Context<'a>,
    ) -> std::result::Result<Value, ExecutionError> {
        let mut context = context.new_inner_scope();

        for binding in &self.bindings {
            let value = binding.inner.execute(&context).map_err(|error| {
                ExecutionError::function_error(
                    &format!("#bind {}", binding.name),
                    error,
                )
            })?;
            context.add_variable_from_value(&binding.name, value);
        }

        self.inner.execute(&context)
    }
}

struct CompiledBinding {
    name: String,
    inner: CelProgram,
}

#[derive(Debug)]
struct Preprocessed {
    expression: String,
    bindings: Vec<Binding>,
}

#[derive(Debug)]
struct Definition {
    name: String,
    replacement: String,
}

#[derive(Debug)]
struct Binding {
    name: String,
    expression: String,
    line: usize,
}

#[derive(Debug)]
enum Directive {
    Define(Definition),
    Bind { name: String, expression: String },
}

#[derive(Debug, Error)]
enum PreprocessorError {
    #[error(
        "invalid preprocessor directive on line {line}, column {column}: {message}"
    )]
    InvalidDirective {
        line: usize,
        column: usize,
        message: String,
    },
    #[error("macro `{name}` is defined more than once")]
    DuplicateDefinition { name: String },
    #[error("binding `{name}` is declared more than once")]
    DuplicateBinding { name: String },
    #[error("`{name}` cannot be both a macro and a binding")]
    ConflictingDirective { name: String },
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

fn directive_parser() -> impl Parser<char, Directive, Error = Simple<char>> {
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
    let expression = any().repeated().at_least(1).collect::<String>();
    let kind = just("define").to(true).or(just("bind").to(false));

    horizontal_whitespace
        .clone()
        .ignore_then(just('#'))
        .then_ignore(horizontal_whitespace.clone())
        .ignore_then(kind)
        .then_ignore(horizontal_whitespace.clone().at_least(1))
        .then(identifier)
        .then_ignore(horizontal_whitespace.at_least(1))
        .then(expression)
        .map(|((define, name), expression)| {
            if define {
                Directive::Define(Definition {
                    name,
                    replacement: expression.trim().to_string(),
                })
            } else {
                Directive::Bind {
                    name,
                    expression: expression.trim().to_string(),
                }
            }
        })
        .then_ignore(end())
}

fn preprocess(source: &str) -> Result<Preprocessed, PreprocessorError> {
    let mut definitions = BTreeMap::new();
    let mut bindings = Vec::new();
    let mut expression = String::with_capacity(source.len());
    let mut state = LexState::Normal;

    for (line_index, line) in source.split_inclusive('\n').enumerate() {
        let directive = state == LexState::Normal
            && line.trim_start_matches([' ', '\t']).starts_with('#');

        if directive {
            let directive = line.trim_end_matches(['\r', '\n']);
            let directive =
                directive_parser().parse(directive).map_err(|errors| {
                    let error =
                        errors.into_iter().next().unwrap_or_else(|| {
                            Simple::custom(0..0, "invalid directive")
                        });
                    PreprocessorError::InvalidDirective {
                        line: line_index + 1,
                        column: error.span().start + 1,
                        message: error.to_string(),
                    }
                })?;

            match directive {
                Directive::Define(definition) => {
                    if bindings.iter().any(|binding: &Binding| {
                        binding.name == definition.name
                    }) {
                        return Err(PreprocessorError::ConflictingDirective {
                            name: definition.name,
                        });
                    }
                    if definitions
                        .insert(definition.name.clone(), definition.replacement)
                        .is_some()
                    {
                        return Err(PreprocessorError::DuplicateDefinition {
                            name: definition.name,
                        });
                    }
                }
                Directive::Bind { name, expression } => {
                    if definitions.contains_key(&name) {
                        return Err(PreprocessorError::ConflictingDirective {
                            name,
                        });
                    }
                    if bindings
                        .iter()
                        .any(|binding: &Binding| binding.name == name)
                    {
                        return Err(PreprocessorError::DuplicateBinding {
                            name,
                        });
                    }
                    bindings.push(Binding {
                        name,
                        expression,
                        line: line_index + 1,
                    });
                }
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
    for binding in &mut bindings {
        let mut expanded = String::with_capacity(binding.expression.len());
        expand(
            &binding.expression,
            &definitions,
            &mut Vec::new(),
            &mut expanded,
        )?;
        binding.expression = expanded;
    }

    Ok(Preprocessed {
        expression: expanded,
        bindings,
    })
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
    use super::{Context, Program, Value, preprocess};

    #[test]
    fn expands_object_macros() {
        let source =
            "#define ISSUE OBFUSCATED_NAMES\ntrace.issue_type == ISSUE";
        assert_eq!(
            preprocess(source).unwrap().expression,
            "\ntrace.issue_type == OBFUSCATED_NAMES"
        );
    }

    #[test]
    fn recursively_expands_macros() {
        let source =
            "#define RESULT SEVERITY\n#define SEVERITY \"low\"\nRESULT";
        assert_eq!(preprocess(source).unwrap().expression, "\n\n\"low\"");
    }

    #[test]
    fn does_not_expand_strings_or_comments() {
        let source =
            "#define VALUE expanded\nVALUE == \"VALUE\" // VALUE\n/* VALUE */";
        assert_eq!(
            preprocess(source).unwrap().expression,
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

    #[test]
    fn evaluates_runtime_bindings_in_order() {
        let source = "#bind DOUBLE input * 2\n#bind RESULT DOUBLE + 1\nRESULT";
        let program = Program::compile(source).unwrap();
        let mut context = Context::default();
        context.add_variable("input", 4).unwrap();

        assert_eq!(program.execute(&context).unwrap(), Value::Int(9));
    }

    #[test]
    fn expands_definitions_in_runtime_bindings() {
        let source =
            "#define MULTIPLIER 3\n#bind RESULT input * MULTIPLIER\nRESULT";
        let program = Program::compile(source).unwrap();
        let mut context = Context::default();
        context.add_variable("input", 4).unwrap();

        assert_eq!(program.execute(&context).unwrap(), Value::Int(12));
    }

    #[test]
    fn bindings_shadow_context_variables_after_initialization() {
        let source = "#bind VALUE VALUE + 1\nVALUE";
        let program = Program::compile(source).unwrap();
        let mut context = Context::default();
        context.add_variable("VALUE", 4).unwrap();

        assert_eq!(program.execute(&context).unwrap(), Value::Int(5));
    }

    #[test]
    fn reports_runtime_binding_errors_with_the_binding_name() {
        let program =
            Program::compile("#bind RESULT missing + 1\nRESULT").unwrap();
        let error = program.execute(&Context::default()).unwrap_err();

        assert!(error.to_string().contains("#bind RESULT"));
    }
}
