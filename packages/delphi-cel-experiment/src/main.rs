use std::{collections::BTreeMap, error::Error, io};

use cel::{Context, Program};
use serde::{Deserialize, Serialize};

const EFFECT_RULE: &str = r#"
	input.trace.issue_type == "OBFUSCATED_NAMES"
		&& input.trace.severity == "high"
		&& "confidence" in input.trace.data
		&& input.trace.data.confidence >= 0.9
	? {
		"severity": "low",
		"hidden": false
	}
	: null
"#;

#[derive(Debug, Serialize)]
struct RuleInput {
    schema_version: u32,
    trace: TraceInput,
    scan: ScanInput,
    artifact: ArtifactInput,
    scope: ScopeInput,
}

#[derive(Debug, Serialize)]
struct TraceInput {
    key: String,
    issue_type: String,
    severity: DelphiSeverity,
    jar: Option<String>,
    file_path: String,
    data: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct ScanInput {
    delphi_version: i32,
}

#[derive(Debug, Serialize)]
struct ArtifactInput {
    size: u32,
    hashes: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
struct ScopeInput {
    project_id: String,
    version_id: String,
    file_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum DelphiSeverity {
    Low,
    Medium,
    High,
    Severe,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleEffect {
    #[serde(default)]
    severity: Option<DelphiSeverity>,
    #[serde(default)]
    hidden: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = example_input();
    println!(
        "CEL input context:\n{}",
        serde_json::to_string_pretty(&input)?
    );

    let effect = evaluate_rule(EFFECT_RULE, &input)?;
    println!("\nCEL map decoded as RuleEffect:\n{effect:#?}");

    Ok(())
}

fn example_input() -> RuleInput {
    RuleInput {
        schema_version: 1,
        trace: TraceInput {
            key: "known-safe:obfuscated-bootstrap".to_string(),
            issue_type: "OBFUSCATED_NAMES".to_string(),
            severity: DelphiSeverity::High,
            jar: Some("META-INF/jars/embedded.jar".to_string()),
            file_path: "com/example/Bootstrap.class".to_string(),
            data: BTreeMap::from([
                ("confidence".to_string(), serde_json::json!(0.97)),
                ("symbol_count".to_string(), serde_json::json!(42)),
            ]),
        },
        scan: ScanInput { delphi_version: 17 },
        artifact: ArtifactInput {
            size: 412_892,
            hashes: BTreeMap::from([
                ("sha1".to_string(), "0123456789abcdef".to_string()),
                ("sha512".to_string(), "fedcba9876543210".to_string()),
            ]),
        },
        scope: ScopeInput {
            project_id: "AANobbMI".to_string(),
            version_id: "IIJJKKLL".to_string(),
            file_id: "XXYYZZ00".to_string(),
        },
    }
}

fn evaluate_rule(
    expression: &str,
    input: &RuleInput,
) -> Result<Option<RuleEffect>, Box<dyn Error>> {
    let mut context = Context::default();
    context.add_variable("input", input)?;

    let value = Program::compile(expression)?.execute(&context)?;
    let json = value
        .json()
        .map_err(|error| invalid_data(error.to_string()))?;
    match json {
        serde_json::Value::Null => Ok(None),
        value => Ok(Some(serde_json::from_value(value)?)),
    }
}

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_a_map_as_an_effect() {
        let effect = evaluate_rule(EFFECT_RULE, &example_input())
            .expect("rule should evaluate");

        assert_eq!(
            effect,
            Some(RuleEffect {
                severity: Some(DelphiSeverity::Low),
                hidden: false,
            })
        );
    }

    #[test]
    fn returns_none_when_a_rule_does_not_match() {
        let mut input = example_input();
        input.trace.issue_type = "NETWORK_ACCESS".to_string();

        let effect =
            evaluate_rule(EFFECT_RULE, &input).expect("rule should evaluate");

        assert_eq!(effect, None);
    }

    #[test]
    fn rejects_unknown_map_fields() {
        let error = evaluate_rule(r#"{"unknown": true}"#, &example_input())
            .expect_err("unknown fields should be rejected");

        assert!(error.to_string().contains("unknown field"));
    }
}
