use std::ops::Range;

use chumsky::{Parser, prelude::*};
use thiserror::Error;

use super::{
    FilterComparison, FilterCondition, FilterExpr, FilterField, FilterLiteral,
    FilterPredicate,
};

#[derive(Debug, Error)]
#[error("invalid filter at byte {position}: {message}")]
pub struct FilterParseError {
    position: usize,
    message: String,
}

fn keyword(
    keyword: &'static str,
) -> BoxedParser<'static, char, (), Simple<char>> {
    let keyword =
        keyword
            .chars()
            .fold(empty().ignored().boxed(), |parser, character| {
                parser
                    .then_ignore(one_of([
                        character.to_ascii_lowercase(),
                        character.to_ascii_uppercase(),
                    ]))
                    .boxed()
            });
    let boundary = filter(|character: &char| {
        !character.is_ascii_alphanumeric() && !"_.".contains(*character)
    })
    .rewind()
    .ignored()
    .or(end());

    keyword.then_ignore(boundary).boxed()
}

fn quoted_literal(
    quote: char,
) -> BoxedParser<'static, char, FilterLiteral, Simple<char>> {
    let escaped = just('\\').ignore_then(any());
    let character = escaped.or(filter(move |character| {
        *character != quote && *character != '\\'
    }));

    character
        .repeated()
        .collect::<String>()
        .delimited_by(just(quote), just(quote))
        .map(FilterLiteral::String)
        .boxed()
}

fn literal_parser() -> BoxedParser<'static, char, FilterLiteral, Simple<char>> {
    let quoted = choice((
        quoted_literal('\''),
        quoted_literal('"'),
        quoted_literal('`'),
    ));
    let bare = filter(|character: &char| {
        !character.is_whitespace() && !",[]()".contains(*character)
    })
    .repeated()
    .at_least(1)
    .collect::<String>()
    .map(FilterLiteral::from_bare);

    quoted.or(bare).padded().boxed()
}

fn field_name_parser() -> BoxedParser<'static, char, String, Simple<char>> {
    filter(|character: &char| {
        character.is_ascii_alphabetic() || "_.".contains(*character)
    })
    .then(
        filter(|character: &char| {
            character.is_ascii_alphanumeric() || "_.".contains(*character)
        })
        .repeated(),
    )
    .map(|(first, rest)| std::iter::once(first).chain(rest).collect::<String>())
    .boxed()
}

fn parser() -> impl Parser<char, FilterExpr, Error = Simple<char>> {
    let field_name = field_name_parser();
    let field = choice((
        field_name.clone(),
        field_name.clone().delimited_by(just('"'), just('"')),
        field_name.clone().delimited_by(just('\''), just('\'')),
        field_name.delimited_by(just('`'), just('`')),
    ))
    .map(FilterField::new)
    .padded();

    let literal = literal_parser();
    let list = literal
        .clone()
        .separated_by(just(',').padded())
        .at_least(1)
        .allow_trailing()
        .delimited_by(just('[').padded(), just(']').padded());

    let comparison = choice((
        just("!=").to(FilterComparison::NotEqual),
        just(">=").to(FilterComparison::GreaterThanOrEqual),
        just("<=").to(FilterComparison::LessThanOrEqual),
        just('>').to(FilterComparison::GreaterThan),
        just('<').to(FilterComparison::LessThan),
        just('=').to(FilterComparison::Equal),
    ))
    .padded()
    .then(literal.clone())
    .map(|(comparison, value)| FilterCondition::Compare { comparison, value });

    let not_in = keyword("NOT")
        .padded()
        .ignore_then(keyword("IN"))
        .padded()
        .ignore_then(list.clone())
        .map(|values| FilterCondition::In {
            values,
            negated: true,
        });
    let in_list = keyword("IN").padded().ignore_then(list).map(|values| {
        FilterCondition::In {
            values,
            negated: false,
        }
    });
    let not_exists = keyword("NOT")
        .padded()
        .ignore_then(keyword("EXISTS"))
        .map(|()| FilterCondition::Exists { negated: true });
    let exists =
        keyword("EXISTS").map(|()| FilterCondition::Exists { negated: false });

    let predicate = field
        .then(choice((not_in, in_list, not_exists, exists, comparison)))
        .map(|(field, condition)| {
            FilterExpr::Predicate(FilterPredicate { field, condition })
        });

    recursive(|expression| {
        let atom = predicate
            .clone()
            .or(expression.delimited_by(just('(').padded(), just(')').padded()))
            .padded();
        let unary = keyword("NOT").padded().repeated().then(atom).map(
            |(operators, expression)| {
                operators.into_iter().fold(expression, |expression, ()| {
                    FilterExpr::Not(Box::new(expression))
                })
            },
        );
        let and = unary
            .clone()
            .then(keyword("AND").padded().ignore_then(unary).repeated())
            .map(|(first, rest)| {
                FilterExpr::and(std::iter::once(first).chain(rest))
                    .expect("an expression always contains one operand")
            });

        and.clone()
            .then(keyword("OR").padded().ignore_then(and).repeated())
            .map(|(first, rest)| {
                FilterExpr::or(std::iter::once(first).chain(rest))
                    .expect("an expression always contains one operand")
            })
    })
    .padded()
    .then_ignore(end())
}

pub fn parse_expression(input: &str) -> Result<FilterExpr, FilterParseError> {
    parser().parse(input).map_err(|errors| {
        let error = errors
            .into_iter()
            .next()
            .unwrap_or_else(|| Simple::custom(0..0, "invalid filter"));
        let Range { start, .. } = error.span();
        FilterParseError {
            position: start,
            message: error.to_string(),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::parse_expression;
    use crate::search::filter::{
        FilterComparison, FilterCondition, FilterExpr, FilterLiteral,
    };

    #[test]
    fn parses_boolean_precedence() {
        let expression = parse_expression(
            "license = MIT OR downloads >= 100 AND open_source = true",
        )
        .unwrap();

        let FilterExpr::Or(expressions) = expression else {
            panic!("expected an OR expression");
        };
        assert_eq!(expressions.len(), 2);
        assert!(matches!(expressions[1], FilterExpr::And(_)));
    }

    #[test]
    fn parses_quoted_and_list_values() {
        let expression = parse_expression(
            r#"name = "value with spaces" AND categories IN [fabric, "forge"]"#,
        )
        .unwrap();

        let FilterExpr::And(expressions) = expression else {
            panic!("expected an AND expression");
        };
        let FilterExpr::Predicate(predicate) = &expressions[0] else {
            panic!("expected a predicate");
        };
        assert!(matches!(
            &predicate.condition,
            FilterCondition::Compare {
                value: FilterLiteral::String(value),
                ..
            } if value == "value with spaces"
        ));
    }

    #[test]
    fn parses_unary_not_with_quoted_field() {
        let expression =
            parse_expression(r#"NOT"project_id"="8xOSkvVU""#).unwrap();

        let FilterExpr::Not(expression) = expression else {
            panic!("expected a NOT expression");
        };
        let FilterExpr::Predicate(predicate) = *expression else {
            panic!("expected a predicate");
        };
        assert_eq!(predicate.field.as_str(), "project_id");
        assert!(matches!(
            predicate.condition,
            FilterCondition::Compare {
                comparison: FilterComparison::Equal,
                value: FilterLiteral::String(value),
            } if value == "8xOSkvVU"
        ));
    }

    #[test]
    fn does_not_parse_not_prefix_in_field_name_as_operator() {
        let expression = parse_expression("notification = true").unwrap();

        let FilterExpr::Predicate(predicate) = expression else {
            panic!("expected a predicate");
        };
        assert_eq!(predicate.field.as_str(), "notification");
    }
}
