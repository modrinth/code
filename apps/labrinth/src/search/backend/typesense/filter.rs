use std::fmt::{self, Display, Formatter};

use eyre::{Result, eyre};

use crate::search::SearchField;
use crate::search::filter::{
    FilterComparison, FilterCondition, FilterExpr, FilterLiteral,
    FilterPredicate,
};

const MAX_DNF_CLAUSES: usize = 64;
const MAX_FILTER_DEPTH: usize = 64;
const MAX_FILTER_NODES: usize = 1024;
const MAX_SERIALIZED_FILTER_BYTES: usize = 64 * 1024;

#[derive(Clone, Copy, PartialEq, Eq)]
enum FilterScope {
    Project,
    Version,
    Mixed,
}

enum TypesenseFilter<'a> {
    And(Vec<Self>),
    Or(Vec<Self>),
    Predicate {
        predicate: &'a FilterPredicate,
        version: bool,
    },
    Join {
        collection: &'a str,
        filter: Box<Self>,
    },
}

pub(super) fn serialize_filter(
    filter: &FilterExpr,
    versions_collection: &str,
) -> Result<String> {
    let (nodes, depth) = filter_complexity(filter);
    if nodes > MAX_FILTER_NODES {
        return Err(eyre!("search filter has too many expressions"));
    }
    if depth > MAX_FILTER_DEPTH {
        return Err(eyre!("search filter is nested too deeply"));
    }

    let filter = plan(filter, versions_collection)?;
    let serialized = filter.to_string();
    if serialized.len() > MAX_SERIALIZED_FILTER_BYTES {
        return Err(eyre!("search filter is too large"));
    }
    Ok(serialized)
}

fn plan<'a>(
    filter: &'a FilterExpr,
    versions_collection: &'a str,
) -> Result<TypesenseFilter<'a>> {
    match filter_scope(filter) {
        FilterScope::Project => lower(filter, false),
        FilterScope::Version => Ok(TypesenseFilter::Join {
            collection: versions_collection,
            filter: Box::new(lower(filter, true)?),
        }),
        FilterScope::Mixed => plan_mixed(filter, versions_collection),
    }
}

fn plan_mixed<'a>(
    filter: &'a FilterExpr,
    versions_collection: &'a str,
) -> Result<TypesenseFilter<'a>> {
    match filter {
        FilterExpr::Or(expressions) => expressions
            .iter()
            .map(|expression| plan(expression, versions_collection))
            .collect::<Result<Vec<_>>>()
            .map(TypesenseFilter::Or),
        FilterExpr::And(expressions)
            if expressions.iter().all(|expression| {
                filter_scope(expression) != FilterScope::Mixed
            }) =>
        {
            plan_partitioned_and(expressions, versions_collection)
        }
        _ => {
            let clauses = to_dnf(filter)?;
            clauses
                .into_iter()
                .map(|clause| plan_clause(clause, versions_collection))
                .collect::<Result<Vec<_>>>()
                .map(TypesenseFilter::Or)
        }
    }
}

fn plan_partitioned_and<'a>(
    expressions: &'a [FilterExpr],
    versions_collection: &'a str,
) -> Result<TypesenseFilter<'a>> {
    let mut project = Vec::new();
    let mut version = Vec::new();
    for expression in expressions {
        match filter_scope(expression) {
            FilterScope::Project => project.push(lower(expression, false)?),
            FilterScope::Version => version.push(lower(expression, true)?),
            FilterScope::Mixed => {
                return Err(eyre!("could not partition mixed search filter"));
            }
        }
    }
    if let Some(filter) = and_filter(version) {
        project.push(TypesenseFilter::Join {
            collection: versions_collection,
            filter: Box::new(filter),
        });
    }
    and_filter(project).ok_or_else(|| eyre!("search filter is empty"))
}

fn plan_clause<'a>(
    predicates: Vec<&'a FilterPredicate>,
    versions_collection: &'a str,
) -> Result<TypesenseFilter<'a>> {
    let mut project = Vec::new();
    let mut version = Vec::new();
    for predicate in predicates {
        let planned = lower_predicate(predicate)?;
        if is_version_filter_field(predicate.field.as_str()) {
            version.push(planned);
        } else {
            project.push(planned);
        }
    }
    if let Some(filter) = and_filter(version) {
        project.push(TypesenseFilter::Join {
            collection: versions_collection,
            filter: Box::new(filter),
        });
    }
    and_filter(project).ok_or_else(|| eyre!("search filter is empty"))
}

fn lower(filter: &FilterExpr, version: bool) -> Result<TypesenseFilter<'_>> {
    match filter {
        FilterExpr::And(expressions) => expressions
            .iter()
            .map(|expression| lower(expression, version))
            .collect::<Result<Vec<_>>>()
            .map(TypesenseFilter::And),
        FilterExpr::Or(expressions) => expressions
            .iter()
            .map(|expression| lower(expression, version))
            .collect::<Result<Vec<_>>>()
            .map(TypesenseFilter::Or),
        FilterExpr::Predicate(predicate) => {
            validate_predicate(predicate)?;
            Ok(TypesenseFilter::Predicate { predicate, version })
        }
    }
}

fn lower_predicate(predicate: &FilterPredicate) -> Result<TypesenseFilter<'_>> {
    validate_predicate(predicate)?;
    Ok(TypesenseFilter::Predicate {
        predicate,
        version: is_version_filter_field(predicate.field.as_str()),
    })
}

fn validate_predicate(predicate: &FilterPredicate) -> Result<()> {
    if matches!(predicate.condition, FilterCondition::Exists { .. }) {
        return Err(eyre!(
            "filter field `{}` does not support `EXISTS`",
            predicate.field.as_str()
        ));
    }
    Ok(())
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
    }
}

fn is_version_filter_field(field: &str) -> bool {
    <SearchField as strum::IntoEnumIterator>::iter().any(|search_field| {
        search_field.is_version_field()
            && search_field.typesense_spec().path == field
    })
}

fn to_dnf(filter: &FilterExpr) -> Result<Vec<Vec<&FilterPredicate>>> {
    match filter {
        FilterExpr::Predicate(predicate) => Ok(vec![vec![predicate]]),
        FilterExpr::Or(expressions) => {
            let mut clauses = Vec::new();
            for expression in expressions {
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
            for expression in expressions {
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
    }
}

fn and_filter(
    mut expressions: Vec<TypesenseFilter<'_>>,
) -> Option<TypesenseFilter<'_>> {
    match expressions.len() {
        0 => None,
        1 => expressions.pop(),
        _ => Some(TypesenseFilter::And(expressions)),
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
    }
}

impl Display for TypesenseFilter<'_> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_with_precedence(formatter, 0)
    }
}

impl TypesenseFilter<'_> {
    fn precedence(&self) -> u8 {
        match self {
            Self::Or(_) => 1,
            Self::And(_) => 2,
            Self::Predicate { .. } | Self::Join { .. } => 3,
        }
    }

    fn fmt_with_precedence(
        &self,
        formatter: &mut Formatter<'_>,
        parent_precedence: u8,
    ) -> fmt::Result {
        let precedence = self.precedence();
        let parenthesize = precedence < parent_precedence;
        if parenthesize {
            formatter.write_str("(")?;
        }

        match self {
            Self::And(expressions) => {
                format_expressions(formatter, expressions, " && ", precedence)?;
            }
            Self::Or(expressions) => {
                format_expressions(formatter, expressions, " || ", precedence)?;
            }
            Self::Predicate { predicate, version } => {
                format_predicate(formatter, predicate, *version)?;
            }
            Self::Join { collection, filter } => {
                write!(formatter, "${collection}(")?;
                filter.fmt_with_precedence(formatter, 0)?;
                formatter.write_str(")")?;
            }
        }

        if parenthesize {
            formatter.write_str(")")?;
        }
        Ok(())
    }
}

fn format_expressions(
    formatter: &mut Formatter<'_>,
    expressions: &[TypesenseFilter<'_>],
    separator: &str,
    precedence: u8,
) -> fmt::Result {
    for (index, expression) in expressions.iter().enumerate() {
        if index != 0 {
            formatter.write_str(separator)?;
        }
        expression.fmt_with_precedence(formatter, precedence)?;
    }
    Ok(())
}

fn format_predicate(
    formatter: &mut Formatter<'_>,
    predicate: &FilterPredicate,
    version: bool,
) -> fmt::Result {
    formatter.write_str(predicate.field.as_str())?;
    match &predicate.condition {
        FilterCondition::Compare { comparison, value } => {
            let operator = match comparison {
                FilterComparison::Equal if version => ":",
                FilterComparison::Equal => ":=",
                FilterComparison::NotEqual => ":!=",
                FilterComparison::GreaterThan => ":>",
                FilterComparison::GreaterThanOrEqual => ":>=",
                FilterComparison::LessThan => ":<",
                FilterComparison::LessThanOrEqual => ":<=",
            };
            formatter.write_str(operator)?;
            format_literal(formatter, value)
        }
        FilterCondition::In { values, negated } => {
            formatter.write_str(if *negated { ":!=" } else { ":" })?;
            formatter.write_str("[")?;
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    formatter.write_str(",")?;
                }
                format_literal(formatter, value)?;
            }
            formatter.write_str("]")
        }
        FilterCondition::Exists { .. } => unreachable!(
            "unsupported predicates are rejected before serialization"
        ),
    }
}

fn format_literal(
    formatter: &mut Formatter<'_>,
    literal: &FilterLiteral,
) -> fmt::Result {
    match literal {
        FilterLiteral::String(value) => {
            formatter.write_str("`")?;
            for character in value.chars() {
                if character == '`' {
                    formatter.write_str("\\")?;
                }
                write!(formatter, "{character}")?;
            }
            formatter.write_str("`")
        }
        FilterLiteral::Number(value) => formatter.write_str(value),
        FilterLiteral::Bool(value) => Display::fmt(value, formatter),
    }
}

#[cfg(test)]
mod tests {
    use super::serialize_filter;
    use crate::search::filter::{normalize, parse_expression};

    fn serialize(input: &str) -> String {
        let filter = normalize(parse_expression(input).unwrap());
        serialize_filter(&filter, "versions").unwrap()
    }

    #[test]
    fn project_filters_do_not_join_versions() {
        assert_eq!(serialize("license = MIT"), "license:=`MIT`");
    }

    #[test]
    fn correlated_version_filters_share_one_join() {
        assert_eq!(
            serialize("categories = fabric AND game_versions = 1.21"),
            "$versions(categories:`fabric` && game_versions:1.21)"
        );
    }

    #[test]
    fn mixed_boolean_filters_preserve_version_correlation() {
        let filter = serialize(
            "(license = MIT OR categories = fabric) AND game_versions = 1.21",
        );

        assert_eq!(filter.matches("$versions(").count(), 2);
        assert!(filter.contains("categories:`fabric` && game_versions:1.21"));
        assert!(filter.contains("license:=`MIT`"));
    }

    #[test]
    fn string_values_are_escaped() {
        assert_eq!(
            serialize(r#"license = "value, with (syntax) and `tick`""#),
            r#"license:=`value, with (syntax) and \`tick\``"#,
        );
    }

    #[test]
    fn cartesian_version_filter_uses_one_join() {
        let filter = serialize(
            "(project_types = modpack AND game_versions = 1.20.1 AND categories = fabric AND categories = technology) OR \
             (project_types = modpack AND game_versions = 1.20.1 AND categories = forge AND categories = technology) OR \
             (project_types = modpack AND game_versions = 1.21.1 AND categories = fabric AND categories = technology) OR \
             (project_types = modpack AND game_versions = 1.21.1 AND categories = forge AND categories = technology)",
        );

        assert_eq!(filter.matches("$versions(").count(), 1);
        assert!(filter.contains("categories:[`fabric`,`forge`]"));
        assert!(filter.contains("game_versions:[`1.20.1`,`1.21.1`]"));
        assert!(filter.contains("categories:`technology`"));
    }
}
