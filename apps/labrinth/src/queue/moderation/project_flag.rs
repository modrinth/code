use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    sync::Arc,
    time::Duration,
};

use eyre::{Result, eyre};
use regex::Regex;
use xredis::RedisPool;

use super::project_flag_files::scan_project_files;
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

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct FlagContext {
    pub project: Project,
    pub team: FlagTeam,
    pub users: Vec<User>,
    pub organization: Option<Organization>,
    pub scanned_files: Vec<ScannedFile>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct FlagTeam {
    pub id: TeamId,
    pub members: Vec<TeamMember>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
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

    pub async fn scan_flags_with_log(
        &self,
        call: &ProjectRuleCall,
        project_id: DBProjectId,
    ) -> Result<ProjectFlagEvaluation> {
        tokio::time::timeout(
            Duration::from_millis(ENV.PROJECT_FLAG_RULE_TIMEOUT),
            async {
                let project =
                    DBProject::get_id(project_id, &*self.ro_pool, &self.redis)
                        .await
                        .wrap_err("fetching project for flag evaluation")?
                        .ok_or_else(|| eyre!("project not found"))?;
                let team_id = project.inner.team_id;
                let organization = match project.inner.organization_id {
                    Some(id) => Some(
                        DBOrganization::get_id(id, &*self.ro_pool, &self.redis)
                            .await
                            .wrap_err("fetching project organization for flag evaluation")?
                            .ok_or_else(|| eyre!("project organization not found"))?,
                    ),
                    None => None,
                };
                let mut team_ids = vec![team_id];
                if let Some(organization) = &organization {
                    team_ids.push(organization.team_id);
                }
                let members = DBTeamMember::get_from_team_full_many(
                    &team_ids,
                    &*self.ro_pool,
                    &self.redis,
                )
                .await
                .wrap_err("fetching team members for flag evaluation")?;
                let mut user_ids = members.iter().map(|member| member.user_id).collect::<Vec<_>>();
                user_ids.sort_unstable_by_key(|id| id.0);
                user_ids.dedup();
                let users = DBUser::get_many_ids(&user_ids, &*self.ro_pool, &self.redis)
                    .await
                    .wrap_err("fetching team users for flag evaluation")?
                    .into_iter()
                    .map(|user| (user.id, User::from(user)))
                    .collect::<HashMap<_, _>>();
                let mut project_members = Vec::new();
                let mut organization_members = Vec::new();
                for member in members {
                    let member_team_id = member.team_id;
                    let user = users.get(&member.user_id)
                        .cloned()
                        .ok_or_else(|| eyre!("team member user not found"))?;
                    let member = TeamMember::from_model(member, user, true);
                    if member_team_id == team_id {
                        project_members.push(member);
                    } else {
                        organization_members.push(member);
                    }
                }
                let users = project_members.iter().map(|member| member.user.clone()).collect();
                let team = FlagTeam {
                    id: team_id.into(),
                    members: project_members,
                };
                let organization = organization.map(|organization| {
                    Organization::from(organization, organization_members)
                });
                let project = Project::from(project);
                let scanned_files = scan_project_files(
                    project_id,
                    &call.scan_files_for,
                    &self.ro_pool,
                    self.file_host.as_ref(),
                )
                .await?;
                let context = FlagContext {
                    project,
                    team,
                    users,
                    organization,
                    scanned_files,
                };
                let rhai = Arc::clone(&self.rhai);
                let ast = call.ast.clone();
                tokio::task::spawn_blocking(move || {
                    let log = EvaluationLog::start();
                    let context = rhai::serde::to_dynamic(context)
                        .wrap_err("serializing flag context for script")?
                        .try_cast::<rhai::Map>()
                        .ok_or_else(|| {
                            eyre!("flag context must serialize to a map")
                        })?;
                    let mut scope = rhai::Scope::new();
                    for (name, value) in context {
                        scope.push_dynamic(name.to_string(), value);
                    }
                    let output = rhai
                        .eval_ast_with_scope::<rhai::Array>(&mut scope, &ast)
                        .wrap_err("running script")?;
                    let flags = output
                        .into_iter()
                        .map(|value| {
                            rhai::serde::from_dynamic::<serde_json::Value>(
                                &value,
                            )
                            .wrap_err("converting project flag to json")
                        })
                        .collect::<Result<Vec<_>>>()?;
                    Ok(ProjectFlagEvaluation {
                        flags,
                        log: log.take(),
                    })
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
