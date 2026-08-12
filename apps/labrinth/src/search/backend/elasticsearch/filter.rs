use eyre::{Result, eyre};
use serde_json::{Value, json};

use crate::search::filter::{
    FilterComparison, FilterCondition, FilterExpr, FilterLiteral,
    FilterPredicate,
};
use crate::search::indexing::normalize_for_search;

const MAX_DNF_CLAUSES: usize = 64;
const MAX_FILTER_DEPTH: usize = 64;
const MAX_FILTER_NODES: usize = 1024;

#[derive(Clone, Copy, PartialEq, Eq)]
enum FilterScope {
    Project,
    Version,
    Mixed,
}

pub(super) struct ElasticsearchFilter {
    pub query: Value,
    pub has_version_filter: bool,
}

pub(super) fn serialize_filter(
    filter: &FilterExpr,
) -> Result<ElasticsearchFilter> {
    let (nodes, depth) = filter_complexity(filter);
    if nodes > MAX_FILTER_NODES {
        return Err(eyre!("search filter has too many expressions"));
    }
    if depth > MAX_FILTER_DEPTH {
        return Err(eyre!("search filter is nested too deeply"));
    }

    let mut inner_hits_index = 0;
    let query = plan(filter, &mut inner_hits_index)?;
    Ok(ElasticsearchFilter {
        query,
        has_version_filter: inner_hits_index != 0,
    })
}

fn plan(filter: &FilterExpr, inner_hits_index: &mut usize) -> Result<Value> {
    match filter_scope(filter) {
        FilterScope::Project => lower(filter),
        FilterScope::Version => version_query(lower(filter)?, inner_hits_index),
        FilterScope::Mixed => plan_mixed(filter, inner_hits_index),
    }
}

fn plan_mixed(
    filter: &FilterExpr,
    inner_hits_index: &mut usize,
) -> Result<Value> {
    match filter {
        FilterExpr::Or(expressions) => expressions
            .iter()
            .map(|expression| plan(expression, inner_hits_index))
            .collect::<Result<Vec<_>>>()
            .map(or_query),
        FilterExpr::And(expressions)
            if expressions.iter().all(|expression| {
                filter_scope(expression) != FilterScope::Mixed
            }) =>
        {
            plan_partitioned_and(expressions, inner_hits_index)
        }
        _ => {
            let clauses = to_dnf(filter)?;
            clauses
                .into_iter()
                .map(|clause| plan_clause(clause, inner_hits_index))
                .collect::<Result<Vec<_>>>()
                .map(or_query)
        }
    }
}

fn plan_partitioned_and(
    expressions: &[FilterExpr],
    inner_hits_index: &mut usize,
) -> Result<Value> {
    let mut project = Vec::new();
    let mut version = Vec::new();
    for expression in expressions {
        match filter_scope(expression) {
            FilterScope::Project => project.push(lower(expression)?),
            FilterScope::Version => version.push(lower(expression)?),
            FilterScope::Mixed => {
                return Err(eyre!("could not partition mixed search filter"));
            }
        }
    }
    if !version.is_empty() {
        project.push(version_query(and_query(version), inner_hits_index)?);
    }
    Ok(and_query(project))
}

fn plan_clause(
    predicates: Vec<&FilterPredicate>,
    inner_hits_index: &mut usize,
) -> Result<Value> {
    let mut project = Vec::new();
    let mut version = Vec::new();
    for predicate in predicates {
        let query = predicate_query(predicate)?;
        if is_version_filter_field(predicate.field.as_str()) {
            version.push(query);
        } else {
            project.push(query);
        }
    }
    if !version.is_empty() {
        project.push(version_query(and_query(version), inner_hits_index)?);
    }
    Ok(and_query(project))
}

fn lower(filter: &FilterExpr) -> Result<Value> {
    match filter {
        FilterExpr::And(expressions) => expressions
            .iter()
            .map(lower)
            .collect::<Result<Vec<_>>>()
            .map(and_query),
        FilterExpr::Or(expressions) => expressions
            .iter()
            .map(lower)
            .collect::<Result<Vec<_>>>()
            .map(or_query),
        FilterExpr::Predicate(predicate) => predicate_query(predicate),
        FilterExpr::Not(_) => {
            Err(eyre!("search filter contains an unnormalized negation"))
        }
    }
}

fn predicate_query(predicate: &FilterPredicate) -> Result<Value> {
    let source_field = predicate.field.as_str();
    let field = exact_field(source_field);
    match &predicate.condition {
        FilterCondition::Compare { comparison, value } => {
            let value = literal_value(source_field, value)?;
            Ok(match comparison {
                FilterComparison::Equal => {
                    json!({"term": {(field): {"value": value}}})
                }
                FilterComparison::NotEqual => not_query(json!({
                    "term": {(field): {"value": value}}
                })),
                FilterComparison::GreaterThan => {
                    json!({"range": {(field): {"gt": value}}})
                }
                FilterComparison::GreaterThanOrEqual => {
                    json!({"range": {(field): {"gte": value}}})
                }
                FilterComparison::LessThan => {
                    json!({"range": {(field): {"lt": value}}})
                }
                FilterComparison::LessThanOrEqual => {
                    json!({"range": {(field): {"lte": value}}})
                }
            })
        }
        FilterCondition::In { values, negated } => {
            let values = values
                .iter()
                .map(|value| literal_value(source_field, value))
                .collect::<Result<Vec<_>>>()?;
            let query = json!({"terms": {(field): values}});
            Ok(if *negated { not_query(query) } else { query })
        }
        FilterCondition::Exists { negated } => {
            let query = json!({"exists": {"field": field}});
            Ok(if *negated { not_query(query) } else { query })
        }
    }
}

fn literal_value(field: &str, literal: &FilterLiteral) -> Result<Value> {
    match literal {
        FilterLiteral::String(value) if field == "author" => {
            Ok(Value::String(normalize_for_search(value)))
        }
        FilterLiteral::String(value) => Ok(Value::String(value.clone())),
        FilterLiteral::Number(value) => serde_json::from_str(value)
            .map_err(|error| eyre!("invalid numeric filter literal: {error}")),
        FilterLiteral::Bool(value) => Ok(Value::Bool(*value)),
    }
}

fn exact_field(field: &str) -> &str {
    match field {
        "name" => "name.keyword",
        "author" => "indexed_author.keyword",
        "summary" => "summary.keyword",
        "slug" => "slug.keyword",
        _ => field,
    }
}

fn version_query(query: Value, inner_hits_index: &mut usize) -> Result<Value> {
    let name = format!("matching_versions_{}", *inner_hits_index);
    *inner_hits_index += 1;
    Ok(json!({
        "has_child": {
            "type": "version",
            "score_mode": "none",
            "query": query,
            "inner_hits": {
                "name": name,
                "size": 1,
                "_source": ["version_id", "version_published_timestamp"],
                "sort": [
                    {"version_published_timestamp": {"order": "desc"}},
                    {"version_id": {"order": "desc"}}
                ]
            }
        }
    }))
}

fn and_query(queries: Vec<Value>) -> Value {
    match queries.len() {
        0 => json!({"match_all": {}}),
        1 => queries.into_iter().next().unwrap_or_default(),
        _ => json!({"bool": {"filter": queries}}),
    }
}

fn or_query(queries: Vec<Value>) -> Value {
    match queries.len() {
        0 => json!({"match_none": {}}),
        1 => queries.into_iter().next().unwrap_or_default(),
        _ => json!({
            "bool": {
                "should": queries,
                "minimum_should_match": 1
            }
        }),
    }
}

fn not_query(query: Value) -> Value {
    json!({
        "bool": {
            "must": [{"match_all": {}}],
            "must_not": [query]
        }
    })
}

fn filter_scope(filter: &FilterExpr) -> FilterScope {
    match filter {
        FilterExpr::Predicate(predicate) => {
            if is_version_filter_field(predicate.field.as_str()) {
                FilterScope::Version
            } else {
                FilterScope::Project
            }
        }
        FilterExpr::And(expressions) | FilterExpr::Or(expressions) => {
            let mut scopes = expressions.iter().map(filter_scope);
            let Some(first) = scopes.next() else {
                return FilterScope::Project;
            };
            if scopes.all(|scope| scope == first) {
                first
            } else {
                FilterScope::Mixed
            }
        }
        FilterExpr::Not(expression) => filter_scope(expression),
    }
}

fn is_version_filter_field(field: &str) -> bool {
    matches!(
        field,
        "categories"
            | "project_types"
            | "environment"
            | "game_versions"
            | "client_side"
            | "server_side"
    )
}

fn to_dnf(filter: &FilterExpr) -> Result<Vec<Vec<&FilterPredicate>>> {
    match filter {
        FilterExpr::Predicate(predicate) => Ok(vec![vec![predicate]]),
        FilterExpr::Or(expressions) => {
            let mut clauses = Vec::new();
            for expression in expressions.iter() {
                clauses.extend(to_dnf(expression)?);
                if clauses.len() > MAX_DNF_CLAUSES {
                    return Err(eyre!(
                        "search filter has too many boolean clauses"
                    ));
                }
            }
            Ok(clauses)
        }
        FilterExpr::And(expressions) => {
            let mut clauses = vec![Vec::new()];
            for expression in expressions.iter() {
                let right = to_dnf(expression)?;
                if clauses.len().saturating_mul(right.len()) > MAX_DNF_CLAUSES {
                    return Err(eyre!(
                        "search filter has too many boolean clauses"
                    ));
                }
                clauses = clauses
                    .into_iter()
                    .flat_map(|left| {
                        right.iter().map(move |right| {
                            let mut clause = left.clone();
                            clause.extend(right);
                            clause
                        })
                    })
                    .collect();
            }
            Ok(clauses)
        }
        FilterExpr::Not(_) => {
            Err(eyre!("search filter contains an unnormalized negation"))
        }
    }
}

fn filter_complexity(filter: &FilterExpr) -> (usize, usize) {
    match filter {
        FilterExpr::Predicate(_) => (1, 1),
        FilterExpr::And(expressions) | FilterExpr::Or(expressions) => {
            expressions.iter().map(filter_complexity).fold(
                (1, 1),
                |(nodes, depth), (child_nodes, child_depth)| {
                    (nodes + child_nodes, depth.max(child_depth + 1))
                },
            )
        }
        FilterExpr::Not(expression) => {
            let (nodes, depth) = filter_complexity(expression);
            (nodes + 1, depth + 1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::serialize_filter;
    use crate::search::filter::{normalize, parse_expression};
    use serde_json::Value;

    fn serialize(input: &str) -> Value {
        let filter = normalize(parse_expression(input).unwrap());
        serialize_filter(&filter).unwrap().query
    }

    #[test]
    fn correlated_version_filters_use_one_join() {
        let query = serialize("categories = fabric AND game_versions = 1.21");
        assert_eq!(query.to_string().matches("has_child").count(), 1);
    }

    #[test]
    fn project_filters_do_not_use_a_join() {
        let query = serialize("license = MIT");
        assert_eq!(query.to_string().matches("has_child").count(), 0);
        assert_eq!(query["term"]["license"]["value"], "MIT");
    }

    #[test]
    fn author_filters_use_the_normalized_exact_field() {
        let query = serialize("author = User");
        assert_eq!(query["term"]["indexed_author.keyword"]["value"], "user");
    }

    #[test]
    fn mixed_boolean_filters_preserve_version_correlation() {
        let query = serialize(
            "(license = MIT OR categories = fabric) AND game_versions = 1.21",
        );
        assert_eq!(query.to_string().matches("has_child").count(), 2);
    }
}
