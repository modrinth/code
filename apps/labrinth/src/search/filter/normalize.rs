use std::collections::{BTreeMap, BTreeSet};

use super::{
    FilterComparison, FilterCondition, FilterExpr, FilterField, FilterLiteral,
    FilterPredicate,
};

pub fn normalize(expression: FilterExpr) -> FilterExpr {
    match expression {
        FilterExpr::And(expressions) => normalize_and(expressions),
        FilterExpr::Or(expressions) => normalize_or(expressions),
        FilterExpr::Predicate(predicate) => normalize_predicate(predicate),
    }
}

fn normalize_predicate(predicate: FilterPredicate) -> FilterExpr {
    if predicate.field.as_str() == "minecraft_java_server.ping.data"
        && let FilterCondition::Exists { negated } = predicate.condition
    {
        return FilterExpr::Predicate(FilterPredicate {
            field: FilterField::new("minecraft_java_server.is_online"),
            condition: FilterCondition::Compare {
                comparison: FilterComparison::Equal,
                value: FilterLiteral::Bool(!negated),
            },
        });
    }

    FilterExpr::Predicate(predicate)
}

fn normalize_and(expressions: Vec<FilterExpr>) -> FilterExpr {
    let mut normalized = expressions
        .into_iter()
        .map(normalize)
        .flat_map(|expression| match expression {
            FilterExpr::And(children) => children,
            expression => vec![expression],
        })
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();

    FilterExpr::and(normalized).expect("an AND expression is non-empty")
}

fn normalize_or(expressions: Vec<FilterExpr>) -> FilterExpr {
    let mut normalized = expressions
        .into_iter()
        .map(normalize)
        .flat_map(|expression| match expression {
            FilterExpr::Or(children) => children,
            expression => vec![expression],
        })
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();

    if let Some(expression) = factor_common_predicates(&normalized) {
        return normalize(expression);
    }

    if let Some(expression) = compact_cartesian_product(&normalized) {
        return expression;
    }

    FilterExpr::or(normalized).expect("an OR expression is non-empty")
}

fn factor_common_predicates(expressions: &[FilterExpr]) -> Option<FilterExpr> {
    let clauses = expressions
        .iter()
        .map(predicate_clause)
        .collect::<Option<Vec<_>>>()?;
    if clauses.len() < 2 {
        return None;
    }

    let common = clauses
        .iter()
        .skip(1)
        .fold(clauses[0].clone(), |common, clause| {
            common.intersection(clause).cloned().collect()
        });
    if common.is_empty() {
        return None;
    }

    let remaining = clauses
        .iter()
        .map(|clause| clause.difference(&common).cloned().collect::<Vec<_>>())
        .collect::<Vec<_>>();
    if remaining.iter().any(Vec::is_empty) {
        return FilterExpr::and(common.into_iter().map(FilterExpr::Predicate));
    }

    let alternatives = remaining.into_iter().map(|clause| {
        FilterExpr::and(clause.into_iter().map(FilterExpr::Predicate))
            .expect("a factored OR clause is non-empty")
    });
    let alternatives =
        FilterExpr::or(alternatives).expect("a factored OR has alternatives");

    FilterExpr::and(
        common
            .into_iter()
            .map(FilterExpr::Predicate)
            .chain([alternatives]),
    )
}

fn compact_cartesian_product(expressions: &[FilterExpr]) -> Option<FilterExpr> {
    let clauses = expressions
        .iter()
        .map(predicate_clause)
        .collect::<Option<Vec<_>>>()?;
    if clauses.len() < 2 {
        return None;
    }

    let common = clauses
        .iter()
        .skip(1)
        .fold(clauses[0].clone(), |common, clause| {
            common.intersection(clause).cloned().collect()
        });
    let remaining = clauses
        .iter()
        .map(|clause| clause.difference(&common).cloned().collect::<Vec<_>>())
        .collect::<Vec<_>>();

    if remaining.iter().any(Vec::is_empty) {
        return FilterExpr::and(common.into_iter().map(FilterExpr::Predicate));
    }

    let mut values_by_field =
        BTreeMap::<FilterField, BTreeSet<FilterLiteral>>::new();
    let expected_fields = remaining[0]
        .iter()
        .map(equality_parts)
        .collect::<Option<Vec<_>>>()?
        .into_iter()
        .map(|(field, _)| field.clone())
        .collect::<BTreeSet<_>>();

    let mut unique_clauses = BTreeSet::new();
    for clause in &remaining {
        let parts = clause
            .iter()
            .map(equality_parts)
            .collect::<Option<Vec<_>>>()?;
        let fields = parts
            .iter()
            .map(|(field, _)| (*field).clone())
            .collect::<BTreeSet<_>>();
        if fields != expected_fields || fields.len() != parts.len() {
            return None;
        }
        for (field, value) in parts {
            values_by_field
                .entry(field.clone())
                .or_default()
                .insert(value.clone());
        }
        unique_clauses.insert(clause.clone());
    }

    let combinations = values_by_field
        .values()
        .try_fold(1usize, |count, values| count.checked_mul(values.len()))?;
    if combinations != unique_clauses.len() {
        return None;
    }

    let compacted = values_by_field.into_iter().map(|(field, values)| {
        let values = values.into_iter().collect::<Vec<_>>();
        let condition = if values.len() == 1 {
            FilterCondition::Compare {
                comparison: FilterComparison::Equal,
                value: values.into_iter().next().expect("one value exists"),
            }
        } else {
            FilterCondition::In {
                values,
                negated: false,
            }
        };
        FilterPredicate { field, condition }
    });

    FilterExpr::and(
        common
            .into_iter()
            .chain(compacted)
            .map(FilterExpr::Predicate),
    )
}

fn predicate_clause(
    expression: &FilterExpr,
) -> Option<BTreeSet<FilterPredicate>> {
    match expression {
        FilterExpr::Predicate(predicate) => {
            Some(BTreeSet::from([predicate.clone()]))
        }
        FilterExpr::And(expressions) => expressions
            .iter()
            .map(|expression| match expression {
                FilterExpr::Predicate(predicate) => Some(predicate.clone()),
                _ => None,
            })
            .collect(),
        FilterExpr::Or(_) => None,
    }
}

fn equality_parts(
    predicate: &FilterPredicate,
) -> Option<(&FilterField, &FilterLiteral)> {
    match &predicate.condition {
        FilterCondition::Compare {
            comparison: FilterComparison::Equal,
            value,
        } => Some((&predicate.field, value)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::normalize;
    use crate::search::filter::{
        FilterCondition, FilterExpr, FilterField, FilterLiteral,
        FilterPredicate, parse_expression,
    };

    #[test]
    fn compacts_cartesian_product() {
        let expression = parse_expression(
            "(game_versions = 1.20.1 AND categories = fabric AND categories = technology) OR \
             (game_versions = 1.20.1 AND categories = forge AND categories = technology) OR \
             (game_versions = 1.21.1 AND categories = fabric AND categories = technology) OR \
             (game_versions = 1.21.1 AND categories = forge AND categories = technology)",
        )
        .unwrap();
        let FilterExpr::And(predicates) = normalize(expression) else {
            panic!("expected a compacted conjunction");
        };

        assert_eq!(predicates.len(), 3);
        assert!(predicates.contains(&FilterExpr::Predicate(FilterPredicate {
            field: FilterField::new("categories"),
            condition: FilterCondition::In {
                values: vec![
                    FilterLiteral::String("fabric".into()),
                    FilterLiteral::String("forge".into()),
                ],
                negated: false,
            },
        })));
        assert!(predicates.contains(&FilterExpr::Predicate(FilterPredicate {
            field: FilterField::new("game_versions"),
            condition: FilterCondition::In {
                values: vec![
                    FilterLiteral::String("1.20.1".into()),
                    FilterLiteral::String("1.21.1".into()),
                ],
                negated: false,
            },
        })));
    }

    #[test]
    fn factors_common_predicates_before_compacting() {
        let expression = parse_expression(
            "(license = MIT AND game_versions = 1.20.1 AND categories = fabric) OR \
             (license = MIT AND game_versions = 1.21.1 AND categories = forge)",
        )
        .unwrap();
        let FilterExpr::And(expressions) = normalize(expression) else {
            panic!("expected a factored conjunction");
        };

        assert!(expressions.iter().any(|expression| matches!(
            expression,
            FilterExpr::Predicate(predicate)
                if predicate.field.as_str() == "license"
        )));
        assert!(
            expressions
                .iter()
                .any(|expression| matches!(expression, FilterExpr::Or(_)))
        );
    }

    #[test]
    fn compacts_production_sized_cartesian_product() {
        let versions = [
            "26.2", "26.1.2", "26.1.1", "26.1", "1.21.11", "1.21.10", "1.21.8",
            "1.21.7", "1.21.5", "1.21.4", "1.21.3", "1.21.1", "1.21", "1.20.6",
            "1.20.4", "1.20.2", "1.20.1", "1.20", "1.19.4", "1.19.3", "1.19.2",
            "1.18.2", "1.17.1", "1.12.2", "1.8.9",
        ];
        let input = versions
            .iter()
            .flat_map(|version| {
                ["fabric", "forge"].map(|loader| {
                    format!(
                        "(project_types = modpack AND game_versions = {version} AND categories = {loader} AND categories = technology)"
                    )
                })
            })
            .collect::<Vec<_>>()
            .join(" OR ");
        let normalized = normalize(parse_expression(&input).unwrap());
        let expected = normalize(
            parse_expression(&format!(
                "project_types = modpack AND game_versions IN [{}] AND categories IN [fabric, forge] AND categories = technology",
                versions.join(", ")
            ))
            .unwrap(),
        );

        assert_eq!(normalized, expected);
    }
}
