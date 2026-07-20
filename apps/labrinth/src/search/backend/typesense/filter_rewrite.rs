use std::sync::LazyLock;

use eyre::{Result, WrapErr, eyre};
use itertools::Itertools;
use regex::Regex;
use serde_json::Value;

use crate::search::SearchField;

#[derive(Clone, Default)]
struct JoinedFilterClause {
    project: Vec<String>,
    version: Vec<String>,
}

pub(super) fn rewrite_filter_for_join(
    filter: &str,
    versions_collection: &str,
) -> Result<String> {
    const MAX_CLAUSES: usize = 256;

    fn parse(expression: &str) -> Result<Vec<JoinedFilterClause>> {
        let expression = trim_outer_parentheses(expression.trim());

        let or_parts = split_top_level(expression, "||");
        if or_parts.len() > 1 {
            let mut clauses = Vec::new();
            for part in or_parts {
                clauses.extend(parse(part)?);
                if clauses.len() > MAX_CLAUSES {
                    return Err(eyre!(
                        "search filter has too many boolean clauses"
                    ));
                }
            }
            return Ok(clauses);
        }

        let and_parts = split_top_level(expression, "&&");
        if and_parts.len() > 1 {
            let mut clauses = vec![JoinedFilterClause::default()];
            for part in and_parts {
                let right = parse(part)?;
                if clauses.len().saturating_mul(right.len()) > MAX_CLAUSES {
                    return Err(eyre!(
                        "search filter has too many boolean clauses"
                    ));
                }
                clauses = clauses
                    .into_iter()
                    .cartesian_product(right)
                    .map(|(mut left, right)| {
                        left.project.extend(right.project);
                        left.version.extend(right.version);
                        left
                    })
                    .collect();
            }
            return Ok(clauses);
        }

        let field = filter_field(expression).ok_or_else(|| {
            eyre!("could not determine filter field in `{expression}`")
        })?;
        let mut clause = JoinedFilterClause::default();
        if is_version_filter_field(field) {
            clause.version.push(version_filter_expression(expression));
        } else {
            clause.project.push(expression.to_string());
        }
        Ok(vec![clause])
    }

    let clauses = parse(filter)?;
    Ok(clauses
        .into_iter()
        .map(|clause| {
            let mut parts = clause.project;
            if !clause.version.is_empty() {
                parts.push(format!(
                    "${versions_collection}({})",
                    clause.version.join(" && ")
                ));
            }
            if parts.len() == 1 {
                parts.pop().unwrap_or_default()
            } else {
                format!("({})", parts.join(" && "))
            }
        })
        .join(" || "))
}

fn is_version_filter_field(field: &str) -> bool {
    <SearchField as strum::IntoEnumIterator>::iter().any(|search_field| {
        search_field.is_version_field()
            && search_field.typesense_spec().path == field
    })
}

fn version_filter_expression(expression: &str) -> String {
    let Some((field, value)) = expression.split_once(':') else {
        return expression.to_string();
    };
    let Some(value) = value.strip_prefix('=') else {
        return expression.to_string();
    };
    format!("{field}:{value}")
}

fn filter_field(expression: &str) -> Option<&str> {
    let operator = expression.find(':')?;
    let field = expression[..operator].trim();
    (!field.is_empty()
        && field.chars().all(|character| {
            character.is_ascii_alphanumeric() || "_.".contains(character)
        }))
    .then_some(field)
}

fn trim_outer_parentheses(mut expression: &str) -> &str {
    while expression.starts_with('(')
        && expression.ends_with(')')
        && matching_outer_parentheses(expression)
    {
        expression = expression[1..expression.len() - 1].trim();
    }
    expression
}

fn matching_outer_parentheses(expression: &str) -> bool {
    let mut depth = 0;
    let mut quote = None;
    let mut escaped = false;

    for (index, character) in expression.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        if let Some(active_quote) = quote {
            if character == active_quote {
                quote = None;
            }
            continue;
        }
        if matches!(character, '\'' | '"' | '`') {
            quote = Some(character);
            continue;
        }
        match character {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 && index + character.len_utf8() < expression.len()
                {
                    return false;
                }
            }
            _ => {}
        }
    }

    depth == 0
}

fn split_top_level<'a>(expression: &'a str, operator: &str) -> Vec<&'a str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut parentheses = 0;
    let mut brackets = 0;
    let mut quote = None;
    let mut escaped = false;
    let bytes = expression.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        let character = expression[index..].chars().next().unwrap_or_default();
        let width = character.len_utf8();
        if escaped {
            escaped = false;
            index += width;
            continue;
        }
        if character == '\\' {
            escaped = true;
            index += width;
            continue;
        }
        if let Some(active_quote) = quote {
            if character == active_quote {
                quote = None;
            }
            index += width;
            continue;
        }
        if matches!(character, '\'' | '"' | '`') {
            quote = Some(character);
            index += width;
            continue;
        }
        match character {
            '(' => parentheses += 1,
            ')' => parentheses -= 1,
            '[' => brackets += 1,
            ']' => brackets -= 1,
            _ => {}
        }

        if parentheses == 0
            && brackets == 0
            && expression[index..].starts_with(operator)
        {
            parts.push(expression[start..index].trim());
            index += operator.len();
            start = index;
            continue;
        }
        index += width;
    }

    if parts.is_empty() {
        vec![expression]
    } else {
        parts.push(expression[start..].trim());
        parts
    }
}

/// Translates a Meilisearch filter expression into Typesense `filter_by`
/// syntax.
///
/// Transformations (applied in order):
/// 1. `field (NOT )IN [v1, v2]`  →  `field:[v1, v2]` / `field:!=[v1, v2]`
/// 2. `field op value` for op ∈ {`!=`, `>=`, `<=`, `>`, `<`, `=`}
///    →  `field:op value`
/// 3. `AND` / `OR` (case-insensitive)  →  `&&` / `||`
pub(super) fn meili_to_typesense(filter: &str) -> String {
    static IN_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"(?i)\b([a-zA-Z_.][a-zA-Z0-9_.]*)\s+(NOT\s+)?IN\s*\[([^\]]*)\]",
        )
        .expect("valid regex")
    });
    static EXISTS_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)\b([a-zA-Z_.][a-zA-Z0-9_.]*)\s+(NOT\s+)?EXISTS\b")
            .expect("valid regex")
    });
    static CMP_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"([a-zA-Z_.][a-zA-Z0-9_.]*)\s*(!=|>=|<=|>|<|=)\s*")
            .expect("valid regex")
    });
    static AND_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?i)\bAND\b").expect("valid regex"));
    static OR_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?i)\bOR\b").expect("valid regex"));

    // Step 1 – IN / NOT IN
    let s = IN_RE.replace_all(filter, |caps: &regex::Captures<'_>| {
        let field = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
        let is_not = caps.get(2).is_some();
        let values = caps.get(3).map(|m| m.as_str()).unwrap_or_default();
        if is_not {
            format!("{field}:!=[{values}]")
        } else {
            format!("{field}:[{values}]")
        }
    });

    let s = EXISTS_RE.replace_all(&s, |caps: &regex::Captures<'_>| {
        let field = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
        let is_not = caps.get(2).is_some();

        match field {
            "minecraft_java_server.ping.data" => format!(
                "minecraft_java_server.is_online:= {}",
                if is_not { "false" } else { "true" }
            ),
            _ => caps
                .get(0)
                .map(|m| m.as_str())
                .unwrap_or_default()
                .to_string(),
        }
    });

    // Step 2 – comparison operators (field op value → field:op value).
    let s = CMP_RE.replace_all(&s, |caps: &regex::Captures<'_>| {
        let field = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
        let op = caps.get(2).map(|m| m.as_str()).unwrap_or_default();
        format!("{field}:{op} ")
    });

    // Step 3 – logical operators
    let s = AND_RE.replace_all(&s, " && ");
    let s = OR_RE.replace_all(&s, " || ");
    s.into_owned()
}

/// Converts the legacy Meilisearch `facets` JSON array into a Typesense
/// `filter_by` string. The outer array items are AND-ed together; the inner
/// array items are OR-ed together.
pub(super) fn facets_to_typesense(facets_json: &str) -> Result<String> {
    let facets = serde_json::from_str::<Vec<Vec<Value>>>(facets_json)
        .wrap_err("failed to parse facets JSON")?;

    let and_parts: Vec<String> = facets
        .into_iter()
        .map(|or_group| {
            let or_parts: Vec<String> = or_group
                .into_iter()
                .map(|facet| {
                    let conditions: Vec<String> = if facet.is_array() {
                        serde_json::from_value::<Vec<String>>(facet)
                            .unwrap_or_default()
                    } else {
                        vec![
                            serde_json::from_value::<String>(facet)
                                .unwrap_or_default(),
                        ]
                    };
                    let and_conds: Vec<String> = conditions
                        .into_iter()
                        .map(|condition| {
                            condition_to_typesense_filter(&condition)
                        })
                        .collect();
                    if and_conds.len() == 1 {
                        and_conds.into_iter().next().unwrap_or_default()
                    } else {
                        format!("({})", and_conds.join(" && "))
                    }
                })
                .collect();
            if or_parts.len() == 1 {
                or_parts.into_iter().next().unwrap_or_default()
            } else {
                format!("({})", or_parts.join(" || "))
            }
        })
        .collect();

    Ok(and_parts.join(" && "))
}

/// Converts a single facet condition such as `"categories:mods"`,
/// `"categories=mods"`, or `"downloads!=100"` into a Typesense filter clause.
fn condition_to_typesense_filter(condition: &str) -> String {
    // Match multi-character operators before their single-character prefixes,
    // and range/inequality operators before the plain `=` equality arm.
    for operator in ["!=", ">=", "<=", ">", "<"] {
        if let Some((field, value)) = condition.split_once(operator) {
            return format!("{}:{} {}", field.trim(), operator, value.trim());
        }
    }
    if let Some((field, value)) = condition.split_once(':') {
        return format!("{}:= {}", field.trim(), value.trim());
    }
    if let Some((field, value)) = condition.split_once('=') {
        return format!("{}:= {}", field.trim(), value.trim());
    }
    condition.to_string()
}

#[cfg(test)]
mod tests {
    use super::rewrite_filter_for_join;

    #[test]
    fn project_filters_do_not_join_versions() {
        assert_eq!(
            rewrite_filter_for_join("license:= MIT", "versions").unwrap(),
            "license:= MIT"
        );
    }

    #[test]
    fn correlated_version_filters_share_one_join() {
        assert_eq!(
            rewrite_filter_for_join(
                "categories:= fabric && game_versions:= 1.21",
                "versions",
            )
            .unwrap(),
            "$versions(categories: fabric && game_versions: 1.21)"
        );
    }

    #[test]
    fn project_and_version_filters_are_partitioned() {
        assert_eq!(
            rewrite_filter_for_join(
                "license:= MIT && categories:= fabric",
                "versions",
            )
            .unwrap(),
            "(license:= MIT && $versions(categories: fabric))"
        );
    }

    #[test]
    fn mixed_boolean_filters_preserve_version_correlation() {
        assert_eq!(
            rewrite_filter_for_join(
                "(license:= MIT || categories:= fabric) && game_versions:= 1.21",
                "versions",
            )
            .unwrap(),
            "(license:= MIT && $versions(game_versions: 1.21)) || $versions(categories: fabric && game_versions: 1.21)"
        );
    }

    #[test]
    fn negative_categories_use_inherited_version_categories() {
        assert_eq!(
            rewrite_filter_for_join("categories:!= fabric", "versions")
                .unwrap(),
            "$versions(categories:!= fabric)"
        );
    }
}
