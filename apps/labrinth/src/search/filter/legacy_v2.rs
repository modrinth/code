use serde_json::Value;
use thiserror::Error;

use super::{FilterExpr, FilterParseError, parse_expression};

#[derive(Debug, Error)]
pub enum LegacyV2FacetsError {
    #[error("invalid facets JSON")]
    Json(#[from] serde_json::Error),
    #[error("facet condition must be a string")]
    InvalidCondition,
    #[error(transparent)]
    Filter(#[from] FilterParseError),
}

pub fn from_legacy_v2_facets_json(
    input: &str,
) -> Result<Option<FilterExpr>, LegacyV2FacetsError> {
    let facets = serde_json::from_str::<Vec<Vec<Value>>>(input)?;
    let mut groups = Vec::new();

    for or_group in facets {
        let mut alternatives = Vec::new();
        for facet in or_group {
            let expression = match facet {
                Value::String(condition) => Some(parse_condition(&condition)?),
                Value::Array(conditions) => {
                    let mut predicates = Vec::new();
                    for condition in conditions {
                        let condition = condition
                            .as_str()
                            .ok_or(LegacyV2FacetsError::InvalidCondition)?;
                        predicates.push(parse_condition(condition)?);
                    }
                    FilterExpr::and(predicates)
                }
                _ => return Err(LegacyV2FacetsError::InvalidCondition),
            };
            if let Some(expression) = expression {
                alternatives.push(expression);
            }
        }
        if let Some(expression) = FilterExpr::or(alternatives) {
            groups.push(expression);
        }
    }

    Ok(FilterExpr::and(groups))
}

fn parse_condition(condition: &str) -> Result<FilterExpr, LegacyV2FacetsError> {
    if ["!=", ">=", "<=", ">", "<", "="]
        .iter()
        .any(|operator| condition.contains(operator))
    {
        parse_expression(condition).map_err(Into::into)
    } else if let Some((field, value)) = condition.split_once(':') {
        parse_expression(&format!("{} = {}", field.trim(), value.trim()))
            .map_err(Into::into)
    } else {
        parse_expression(condition).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::from_legacy_v2_facets_json;
    use crate::search::filter::FilterExpr;

    #[test]
    fn converts_v2_boolean_structure() {
        let expression = from_legacy_v2_facets_json(
            r#"[["categories:fabric", "categories:forge"], [["game_versions:1.21", "project_types:mod"]]]"#,
        )
        .unwrap()
        .unwrap();

        let FilterExpr::And(groups) = expression else {
            panic!("expected outer facets to be joined with AND");
        };
        assert!(matches!(groups[0], FilterExpr::Or(_)));
        assert!(matches!(groups[1], FilterExpr::And(_)));
    }

    #[test]
    fn preserves_colons_inside_comparison_values() {
        from_legacy_v2_facets_json(
            r#"[["license='https://example.com/license'"]]"#,
        )
        .unwrap()
        .unwrap();
    }
}
