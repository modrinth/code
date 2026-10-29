use std::{cell::RefCell, sync::Arc, time::Duration};

use eyre::Result;

use crate::{env::ENV, models::projects::Project, util::error::Context};

thread_local! {
    static EVALUATION_LOG: RefCell<Option<Vec<String>>> = const { RefCell::new(None) };
}

struct EvaluationLog(Option<Vec<String>>);

impl EvaluationLog {
    fn start() -> Self {
        Self(EVALUATION_LOG.replace(Some(Vec::new())))
    }

    fn take(&self) -> Vec<String> {
        EVALUATION_LOG.with(|log| log.borrow_mut().take().unwrap_or_default())
    }
}

impl Drop for EvaluationLog {
    fn drop(&mut self) {
        EVALUATION_LOG.replace(self.0.take());
    }
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ProjectFlagEvaluation {
    pub flags: Vec<serde_json::Value>,
    pub log: Vec<String>,
}

#[derive(Debug)]
pub struct ProjectFlagEngine {
    rhai: Arc<rhai::Engine>,
}

impl ProjectFlagEngine {
    pub fn new() -> Result<Self> {
        let mut rhai = rhai::Engine::new();
        rhai.disable_symbol("eval"); // eval considered evil
        rhai.set_max_operations(1_000_000);
        rhai.set_max_call_levels(64);
        rhai.set_max_string_size(1_048_576);
        rhai.set_max_array_size(10_000);
        rhai.set_max_map_size(10_000);
        rhai.on_print(|message| {
            EVALUATION_LOG.with(|log| {
                if let Some(log) = log.borrow_mut().as_mut() {
                    log.push(message.to_owned());
                }
            });
        });
        Ok(Self {
            rhai: Arc::new(rhai),
        })
    }

    pub fn compile(&self, expression: &str) -> Result<rhai::AST> {
        self.rhai.compile(expression).wrap_err("compiling script")
    }

    pub async fn scan_flags_with_log(
        &self,
        ast: &rhai::AST,
        project: &Project,
    ) -> Result<ProjectFlagEvaluation> {
        let rhai = Arc::clone(&self.rhai);
        let ast = ast.clone();
        let project = project.clone();
        tokio::time::timeout(
            Duration::from_millis(ENV.PROJECT_FLAG_RULE_TIMEOUT),
            tokio::task::spawn_blocking(move || {
                let log = EvaluationLog::start();
                let project = rhai::serde::to_dynamic(&project)
                    .wrap_err("serializing project for script")?;
                let mut scope = rhai::Scope::new();
                scope.push("project", project);
                let output = rhai
                    .eval_ast_with_scope::<rhai::Array>(&mut scope, &ast)
                    .wrap_err("running script")?;
                let flags = output
                    .into_iter()
                    .map(|value| {
                        rhai::serde::from_dynamic::<serde_json::Value>(&value)
                            .wrap_err("converting project flag to json")
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(ProjectFlagEvaluation {
                    flags,
                    log: log.take(),
                })
            }),
        )
        .await
        .wrap_err("project flag rule evaluation timed out")?
        .wrap_err("joining project flag evaluation")?
    }

    pub async fn scan_flags(
        &self,
        ast: &rhai::AST,
        project: &Project,
    ) -> Result<Vec<serde_json::Value>> {
        Ok(self.scan_flags_with_log(ast, project).await?.flags)
    }
}
