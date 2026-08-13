use crate::state::{JavaVersion, MemorySettings};
use regex::{Captures, Regex};
use std::collections::BTreeMap;
use std::sync::LazyLock;

static ENV_VAR_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$(\w+)").expect("valid env var regex"));

#[derive(Debug, Clone)]
pub(crate) struct HookVariables {
    pub instance_name: String,
    pub instance_id: String,
    pub instance_dir: String,
    pub java_path: String,
    pub java_args: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HookEnvironment {
    lookup_env: BTreeMap<String, String>,
    injected_env: BTreeMap<String, String>,
}

impl HookEnvironment {
    pub(crate) fn from_current_env(
        custom_env_vars: &[(String, String)],
        variables: HookVariables,
    ) -> Self {
        Self::new(
            std::env::vars_os().map(|(key, value)| {
                (
                    key.to_string_lossy().into_owned(),
                    value.to_string_lossy().into_owned(),
                )
            }),
            custom_env_vars,
            variables,
        )
    }

    fn new(
        process_env: impl IntoIterator<Item = (String, String)>,
        custom_env_vars: &[(String, String)],
        variables: HookVariables,
    ) -> Self {
        let mut lookup_env =
            process_env.into_iter().collect::<BTreeMap<_, _>>();
        let mut injected_env = BTreeMap::new();

        for (key, value) in custom_env_vars {
            lookup_env.insert(key.clone(), value.clone());
            injected_env.insert(key.clone(), value.clone());
        }

        let hook_vars = [
            ("INST_NAME", variables.instance_name),
            ("INST_ID", variables.instance_id),
            ("INST_DIR", variables.instance_dir.clone()),
            ("INST_MC_DIR", variables.instance_dir),
            ("INST_JAVA", variables.java_path),
            ("INST_JAVA_ARGS", variables.java_args),
        ];

        for (key, value) in hook_vars {
            let key = key.to_string();
            lookup_env.insert(key.clone(), value.clone());
            injected_env.insert(key, value);
        }

        Self {
            lookup_env,
            injected_env,
        }
    }

    pub(crate) fn expand(&self, input: &str) -> String {
        ENV_VAR_PATTERN
            .replace_all(input, |captures: &Captures| {
                self.lookup_env
                    .get(&captures[1])
                    .cloned()
                    .unwrap_or_else(|| captures[0].to_string())
            })
            .into_owned()
    }

    pub(crate) fn injected_envs(&self) -> Vec<(String, String)> {
        self.injected_env
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()
    }
}

pub(crate) fn build_hook_java_args(
    java_args: &[String],
    memory: MemorySettings,
    java_version: &JavaVersion,
) -> String {
    let mut args = vec![format!("-Xmx{}M", memory.maximum)];

    args.extend(java_args.iter().filter(|arg| !arg.is_empty()).cloned());

    if java_version.parsed_version >= 9 {
        args.push(
            "--add-opens=java.base/java.lang.reflect=ALL-UNNAMED".to_string(),
        );
    }

    if java_version.parsed_version >= 25 {
        args.push(
            "--add-opens=jdk.internal/jdk.internal.misc=ALL-UNNAMED"
                .to_string(),
        );
    }

    args.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_variables() -> HookVariables {
        HookVariables {
            instance_name: "Test Instance".to_string(),
            instance_id: "test-instance".to_string(),
            instance_dir: "/profiles/test-instance".to_string(),
            java_path: "/java/bin/java".to_string(),
            java_args: "-Xmx4096M".to_string(),
        }
    }

    #[test]
    fn expands_builtin_and_custom_variables() {
        let env = HookEnvironment::new(
            [("HOME".to_string(), "/home/alex".to_string())],
            &[("CUSTOM_VAR".to_string(), "custom".to_string())],
            sample_variables(),
        );

        assert_eq!(
            env.expand("$HOME/$INST_ID/$CUSTOM_VAR"),
            "/home/alex/test-instance/custom"
        );
    }

    #[test]
    fn leaves_unknown_variables_untouched() {
        let env = HookEnvironment::new([], &[], sample_variables());

        assert_eq!(env.expand("$UNKNOWN/$INST_NAME"), "$UNKNOWN/Test Instance");
    }

    #[test]
    fn expands_empty_variables_to_empty_strings() {
        let env = HookEnvironment::new(
            [("EMPTY_VAR".to_string(), String::new())],
            &[],
            sample_variables(),
        );

        assert_eq!(env.expand("prefix$EMPTY_VAR-suffix"), "prefix-suffix");
    }
}
