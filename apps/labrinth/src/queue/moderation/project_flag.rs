use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    sync::{Arc, LazyLock},
    time::Duration,
};

use eyre::{Result, ensure, eyre};
use regex::Regex;
use xredis::RedisPool;

use super::project_flag_files::{
    ProjectScanFile, load_project_files, scan_project_files,
};

use crate::{
    database::{
        ReadOnlyPgPool,
        models::{
            DBOrganization, DBProject, DBProjectId, DBTeamMember, DBUser,
            project_flag_item::{PatternName, ProjectRuleParams},
        },
    },
    env::ENV,
    file_hosting::FileHost,
    models::{
        ids::{FileId, TeamId, VersionId},
        organizations::Organization,
        projects::Project,
        teams::TeamMember,
        users::User,
    },
    util::error::Context,
};

static DOWNLOADS: LazyLock<tokio::sync::Semaphore> = LazyLock::new(|| {
    tokio::sync::Semaphore::new(
        ENV.PROJECT_FLAG_SCAN_DOWNLOAD_CONCURRENCY.get(),
    )
});

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

#[derive(Clone, serde::Serialize, utoipa::ToSchema)]
pub struct FlagContext {
    pub project: Project,
    pub team: FlagTeam,
    pub users: Vec<User>,
    pub organization: Option<Organization>,
    pub scanned_files: Vec<ScannedFile>,
}

#[derive(Clone, serde::Serialize, utoipa::ToSchema)]
pub struct FlagTeam {
    pub id: TeamId,
    pub members: Vec<TeamMember>,
}

#[derive(Clone, serde::Serialize, utoipa::ToSchema)]
pub struct ScannedFile {
    #[schema(example = "AABBCCDD")]
    pub version_id: VersionId,
    #[schema(example = "EEFFGGHH")]
    pub file_id: FileId,
    #[schema(example = "META-INF/mods.toml")]
    pub file_path: String,
    #[schema(
        example = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    )]
    pub sha256: String,
    pub found_patterns: Vec<PatternName>,
}

#[derive(Clone, Debug)]
pub struct ProjectRuleCall {
    pub ast: rhai::AST,
    pub scan_files_for: BTreeMap<PatternName, Regex>,
}

#[derive(Clone)]
pub struct ProjectFlagEngine {
    rhai: Arc<rhai::Engine>,
    ro_pool: ReadOnlyPgPool,
    redis: RedisPool,
    file_host: Arc<dyn FileHost>,
}

impl ProjectFlagEngine {
    pub fn new(
        ro_pool: ReadOnlyPgPool,
        redis: RedisPool,
        file_host: Arc<dyn FileHost>,
    ) -> Result<Self> {
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
            ro_pool,
            redis,
            file_host,
        })
    }

    pub fn make_call(
        &self,
        params: &ProjectRuleParams,
    ) -> Result<ProjectRuleCall> {
        let ast = self
            .rhai
            .compile(&params.expression)
            .wrap_err("compiling script")?;
        Ok(ProjectRuleCall {
            ast,
            scan_files_for: params.scan_files_for.clone(),
        })
    }

    pub(super) async fn load_contexts(
        &self,
        project_ids: &[DBProjectId],
    ) -> Result<HashMap<DBProjectId, FlagContext>> {
        if project_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let projects =
            DBProject::get_many_ids(project_ids, &*self.ro_pool, &self.redis)
                .await
                .wrap_err("fetching projects for flag evaluation")?;
        let mut organization_ids = projects
            .iter()
            .filter_map(|p| p.inner.organization_id)
            .collect::<Vec<_>>();
        organization_ids.sort_unstable_by_key(|id| id.0);
        organization_ids.dedup();
        let organizations = DBOrganization::get_many_ids(
            &organization_ids,
            &*self.ro_pool,
            &self.redis,
        )
        .await
        .wrap_err("fetching organizations for flag evaluation")?
        .into_iter()
        .map(|organization| (organization.id, organization))
        .collect::<HashMap<_, _>>();
        let mut team_ids = projects
            .iter()
            .map(|p| p.inner.team_id)
            .chain(organizations.values().map(|o| o.team_id))
            .collect::<Vec<_>>();
        team_ids.sort_unstable_by_key(|id| id.0);
        team_ids.dedup();
        let members = DBTeamMember::get_from_team_full_many(
            &team_ids,
            &*self.ro_pool,
            &self.redis,
        )
        .await
        .wrap_err("fetching team members for flag evaluation")?;
        let mut user_ids =
            members.iter().map(|m| m.user_id).collect::<Vec<_>>();
        user_ids.sort_unstable_by_key(|id| id.0);
        user_ids.dedup();
        let users =
            DBUser::get_many_ids(&user_ids, &*self.ro_pool, &self.redis)
                .await
                .wrap_err("fetching team users for flag evaluation")?
                .into_iter()
                .map(|user| (user.id, User::from(user)))
                .collect::<HashMap<_, _>>();
        let mut teams = HashMap::<_, Vec<TeamMember>>::new();
        for member in members {
            let user = users
                .get(&member.user_id)
                .cloned()
                .ok_or_else(|| eyre!("team member user not found"))?;
            teams
                .entry(member.team_id)
                .or_default()
                .push(TeamMember::from_model(member, user, true));
        }
        let mut contexts = HashMap::new();
        for project in projects {
            let id = project.inner.id;
            let team_id = project.inner.team_id;
            let members = teams.get(&team_id).cloned().unwrap_or_default();
            let organization = project
                .inner
                .organization_id
                .map(|id| {
                    let organization =
                        organizations.get(&id).cloned().ok_or_else(|| {
                            eyre!("project organization not found")
                        })?;
                    let members = teams
                        .get(&organization.team_id)
                        .cloned()
                        .unwrap_or_default();
                    eyre::Ok(Organization::from(organization, members))
                })
                .transpose()?;
            contexts.insert(
                id,
                FlagContext {
                    project: Project::from(project),
                    users: members
                        .iter()
                        .map(|member| member.user.clone())
                        .collect(),
                    team: FlagTeam {
                        id: team_id.into(),
                        members,
                    },
                    organization,
                    scanned_files: Vec::new(),
                },
            );
        }
        ensure!(
            project_ids.iter().all(|id| contexts.contains_key(id)),
            "project not found"
        );
        Ok(contexts)
    }

    pub(super) async fn load_files(
        &self,
        project_ids: &[DBProjectId],
    ) -> Result<HashMap<DBProjectId, Vec<ProjectScanFile>>> {
        load_project_files(project_ids, &self.ro_pool).await
    }

    pub(super) async fn prepare_context(
        &self,
        mut context: FlagContext,
        files: &[ProjectScanFile],
        patterns: &BTreeMap<PatternName, Regex>,
    ) -> Result<FlagContext> {
        context.scanned_files = scan_project_files(
            files,
            patterns,
            self.file_host.as_ref(),
            &DOWNLOADS,
        )
        .await?;
        Ok(context)
    }

    pub fn evaluate(
        &self,
        call: &ProjectRuleCall,
        context: &FlagContext,
    ) -> Result<ProjectFlagEvaluation> {
        let log = EvaluationLog::start();
        let context = rhai::serde::to_dynamic(context)
            .wrap_err("serializing flag context for script")?
            .try_cast::<rhai::Map>()
            .ok_or_else(|| eyre!("flag context must serialize to a map"))?;
        let mut scope = rhai::Scope::new();
        for (name, value) in context {
            scope.push_dynamic(name.to_string(), value);
        }
        let output = self
            .rhai
            .eval_ast_with_scope::<rhai::Array>(&mut scope, &call.ast)
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
    }

    pub async fn scan_flags_with_log(
        &self,
        call: &ProjectRuleCall,
        project_id: DBProjectId,
    ) -> Result<ProjectFlagEvaluation> {
        tokio::time::timeout(
            Duration::from_millis(ENV.PROJECT_FLAG_RULE_TIMEOUT),
            async {
                let context = self
                    .load_contexts(&[project_id])
                    .await?
                    .remove(&project_id)
                    .ok_or_else(|| eyre!("project not found"))?;
                let files = self
                    .load_files(&[project_id])
                    .await?
                    .remove(&project_id)
                    .unwrap_or_default();
                let context = self
                    .prepare_context(context, &files, &call.scan_files_for)
                    .await?;
                let engine = self.clone();
                let call = call.clone();
                tokio::task::spawn_blocking(move || {
                    engine.evaluate(&call, &context)
                })
                .await
                .wrap_err("joining project flag evaluation")?
            },
        )
        .await
        .wrap_err("project flag rule evaluation timed out")?
    }

    pub async fn scan_flags(
        &self,
        call: &ProjectRuleCall,
        project_id: DBProjectId,
    ) -> Result<Vec<serde_json::Value>> {
        Ok(self.scan_flags_with_log(call, project_id).await?.flags)
    }
}
