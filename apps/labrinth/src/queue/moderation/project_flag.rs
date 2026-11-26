use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, HashMap},
    sync::{Arc, LazyLock},
    time::Duration,
};

use anyhow::{Result, anyhow};
use regex::Regex;
use rhai::packages::Package;
use rhai_chrono::ChronoPackage;
use rhai_url::UrlPackage;
use xredis::RedisPool;

pub use super::project_flag_attribution::FlagAttributionGroup;
use super::project_flag_attribution::load_attributions;
use super::project_flag_files::{
    ProjectScanFile, load_project_files, scan_project_files,
};

use crate::{
    database::{
        ReadOnlyPgPool,
        models::{
            DBOrganization, DBProject, DBProjectDisclosure, DBProjectId,
            DBTeamMember, DBThread, DBUser, DBVersion, DBVersionId,
            project_flag_item::{
                DBProjectFlagModule, DBProjectFlagRule, DecisionQuestion,
                PatternName, ProjectFlag, ProjectRuleParams,
            },
        },
    },
    env::ENV,
    file_hosting::FileHost,
    models::{
        ids::{FileId, TeamId, ThreadId, VersionId},
        organizations::Organization,
        projects::{Project, Version},
        teams::TeamMember,
        threads::ThreadMessage,
        users::User,
        v3::disclosures::ProjectDisclosureData,
    },
    util::error::Context,
    workflow::project_flag::scan_cache::ProjectFlagScanKind,
};

#[derive(Clone, Copy)]
enum ContextSource {
    Cached,
    Authoritative,
}

pub(crate) fn sort_versions_newest_first(versions: &mut [Version]) {
    versions.sort_by(|a, b| {
        b.date_published
            .cmp(&a.date_published)
            .then_with(|| b.id.0.cmp(&a.id.0))
    });
}

static DOWNLOADS: LazyLock<Arc<tokio::sync::Semaphore>> = LazyLock::new(|| {
    Arc::new(tokio::sync::Semaphore::new(
        ENV.PROJECT_FLAG_SCAN_DOWNLOAD_CONCURRENCY.get(),
    ))
});

thread_local! {
    static EVALUATION_LOG: RefCell<Option<CappedLog>> = const { RefCell::new(None) };
}

/// Collects string messages into a log, up to a certain `max_bytes` capacity,
/// then will output `(omitted N bytes)` instead.
#[derive(Default)]
struct CappedLog {
    messages: Vec<String>,
    bytes: usize,
    omitted_bytes: usize,
}

impl CappedLog {
    fn push(&mut self, message: &str, max_bytes: usize) {
        if self.omitted_bytes > 0 {
            self.omitted_bytes =
                self.omitted_bytes.saturating_add(message.len());
            return;
        }
        let mut retained =
            message.len().min(max_bytes.saturating_sub(self.bytes));
        while !message.is_char_boundary(retained) {
            retained -= 1;
        }
        if retained > 0 {
            self.messages.push(message[..retained].to_owned());
            self.bytes += retained;
        }
        self.omitted_bytes = message.len() - retained;
    }

    fn finish(mut self) -> Vec<String> {
        if self.omitted_bytes > 0 {
            self.messages
                .push(format!("(omitted {} bytes)", self.omitted_bytes));
        }
        self.messages
    }
}

#[cfg(test)]
mod evaluation_log_tests {
    use super::CappedLog;

    #[test]
    fn retains_messages_within_budget() {
        let mut log = CappedLog::default();
        log.push("hello", 10);
        log.push("world", 10);
        assert_eq!(log.finish(), vec!["hello", "world"]);
    }

    #[test]
    fn counts_all_bytes_after_truncation() {
        let mut log = CappedLog::default();
        log.push("hello", 7);
        log.push("world", 7);
        log.push("é", 7);
        assert_eq!(log.finish(), vec!["hello", "wo", "(omitted 5 bytes)"]);
    }

    #[test]
    fn truncates_at_utf8_boundary_and_does_not_resume() {
        let mut log = CappedLog::default();
        log.push("aé", 2);
        log.push("b", 2);
        assert_eq!(log.finish(), vec!["a", "(omitted 3 bytes)"]);
    }

    #[test]
    fn zero_budget_only_retains_summary() {
        let mut log = CappedLog::default();
        log.push("hello", 0);
        log.push("world", 0);
        assert_eq!(log.finish(), vec!["(omitted 10 bytes)"]);
    }

    #[test]
    fn empty_messages_do_not_allocate_entries() {
        let mut log = CappedLog::default();
        log.push("", 10);
        assert!(log.finish().is_empty());
    }
}

struct EvaluationLog(Option<CappedLog>);

impl EvaluationLog {
    fn start() -> Self {
        Self(EVALUATION_LOG.replace(Some(CappedLog::default())))
    }

    fn take(&self) -> Vec<String> {
        EVALUATION_LOG
            .with(|log| log.borrow_mut().take().unwrap_or_default().finish())
    }
}

impl Drop for EvaluationLog {
    fn drop(&mut self) {
        EVALUATION_LOG.replace(self.0.take());
    }
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ProjectFlagEvaluation {
    pub flags: Vec<ProjectFlag>,
    pub log: Vec<String>,
}

#[derive(Clone, serde::Serialize, utoipa::ToSchema)]
pub struct FlagContext {
    pub ai: FlagAiContext,
    pub project: Project,
    pub team: FlagTeam,
    pub users: Vec<FlagUser>,
    pub organization: Option<FlagOrganization>,
    pub scanned_files: Vec<ScannedFile>,
    pub versions: Vec<Version>,
    pub disclosures: Vec<ProjectDisclosureData>,
    pub attributions: Vec<FlagAttributionGroup>,
    pub moderation_thread: FlagModerationThread,
}

/// Cached AI results only. Missing or disabled scans are null, never zero.
#[derive(Clone, Default, serde::Serialize, utoipa::ToSchema)]
pub struct FlagAiContext {
    pub images: Vec<FlagAiImage>,
    /// Named answers for the rule currently being evaluated.
    #[schema(example = json!({"advertising": 0.9, "unavailable": null}))]
    pub decisions: BTreeMap<String, Option<f64>>,
    /// Context inspection answers, scoped by rule ID to avoid name collisions.
    pub decisions_by_rule: BTreeMap<String, BTreeMap<String, Option<f64>>>,
}

#[derive(Clone, serde::Serialize, utoipa::ToSchema)]
pub struct FlagAiImage {
    pub url: String,
    #[schema(example = 0.02)]
    pub explicit: Option<f64>,
    #[schema(example = 0.85)]
    pub ai_generated: Option<f64>,
}

#[derive(Clone, serde::Serialize, utoipa::ToSchema)]
pub struct FlagModerationThread {
    pub id: ThreadId,
    pub messages: Vec<ThreadMessage>,
}

#[derive(Clone, serde::Serialize, utoipa::ToSchema)]
pub struct FlagTeam {
    pub id: TeamId,
    pub members: Vec<FlagTeamMember>,
}

#[derive(Clone, serde::Serialize, utoipa::ToSchema)]
pub struct FlagUser {
    #[serde(flatten)]
    pub user: User,
    /// Other directly user-owned projects; excludes the evaluated project.
    pub projects: Vec<Project>,
}

#[derive(Clone, serde::Serialize, utoipa::ToSchema)]
pub struct FlagTeamMember {
    pub team_id: TeamId,
    pub user: FlagUser,
    pub role: String,
    pub is_owner: bool,
    pub permissions: Option<crate::models::teams::ProjectPermissions>,
    pub organization_permissions:
        Option<crate::models::teams::OrganizationPermissions>,
    pub accepted: bool,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub payouts_split: Option<rust_decimal::Decimal>,
    pub ordering: i64,
}

impl FlagTeamMember {
    fn new(member: TeamMember, projects: Vec<Project>) -> Self {
        Self {
            team_id: member.team_id,
            user: FlagUser {
                user: member.user,
                projects,
            },
            role: member.role,
            is_owner: member.is_owner,
            permissions: member.permissions,
            organization_permissions: member.organization_permissions,
            accepted: member.accepted,
            payouts_split: member.payouts_split,
            ordering: member.ordering,
        }
    }
}

#[derive(Clone, serde::Serialize, utoipa::ToSchema)]
pub struct FlagOrganization {
    pub id: crate::models::ids::OrganizationId,
    pub slug: String,
    pub name: String,
    pub team_id: TeamId,
    pub description: String,
    pub icon_url: Option<String>,
    pub raw_icon_url: Option<String>,
    pub color: Option<u32>,
    pub members: Vec<FlagTeamMember>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moderation_notes:
        Option<Option<crate::models::v3::moderation_notes::ModerationNote>>,
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
    pub decision_questions: HashMap<String, DecisionQuestion>,
}

thread_local! {
    static MODULE_DEPTH: Cell<usize> = const { Cell::new(0) };
    static MODULE_COUNT: Cell<usize> = const { Cell::new(0) };
}

struct ModuleDepthGuard;

impl Drop for ModuleDepthGuard {
    fn drop(&mut self) {
        MODULE_DEPTH.with(|depth| depth.set(depth.get() - 1));
    }
}

struct DatabaseModuleResolver {
    scripts: HashMap<String, String>,
}

impl rhai::ModuleResolver for DatabaseModuleResolver {
    fn resolve(
        &self,
        engine: &rhai::Engine,
        _source: Option<&str>,
        path: &str,
        pos: rhai::Position,
    ) -> Result<Arc<rhai::Module>, Box<rhai::EvalAltResult>> {
        let script = self.scripts.get(path).ok_or_else(|| {
            Box::new(rhai::EvalAltResult::ErrorModuleNotFound(path.into(), pos))
        })?;
        let depth = MODULE_DEPTH.with(Cell::get);

        let count = MODULE_COUNT.with(Cell::get);
        if depth >= 32 || count >= 128 {
            return Err(Box::new(rhai::EvalAltResult::ErrorTooManyModules(
                pos,
            )));
        }
        MODULE_COUNT.with(|value| value.set(count + 1));
        MODULE_DEPTH.with(|value| value.set(depth + 1));
        let _guard = ModuleDepthGuard;
        let mut ast = engine.compile(script).map_err(|error| {
            Box::new(rhai::EvalAltResult::ErrorInModule(
                path.into(),
                error.into(),
                pos,
            ))
        })?;
        ast.set_source(path);
        rhai::Module::eval_ast_as_new(rhai::Scope::new(), &ast, engine)
            .map(Arc::new)
            .map_err(|error| {
                Box::new(rhai::EvalAltResult::ErrorInModule(
                    path.into(),
                    error,
                    pos,
                ))
            })
    }
}

#[derive(Clone)]
pub struct ProjectFlagEngine {
    rhai: Arc<rhai::Engine>,
    ro_pool: ReadOnlyPgPool,
    redis: RedisPool,
    file_host: Arc<dyn FileHost>,
    scanner_config: crate::env::ProjectFlags,
}

impl ProjectFlagEngine {
    pub fn new(
        ro_pool: ReadOnlyPgPool,
        redis: RedisPool,
        file_host: Arc<dyn FileHost>,
        scanner_config: crate::env::ProjectFlags,
    ) -> Result<Self> {
        Ok(Self {
            rhai: Arc::new(Self::script_engine(HashMap::new())),
            ro_pool,
            redis,
            file_host,
            scanner_config,
        })
    }

    pub async fn with_module_snapshot(&self) -> Result<Self> {
        let scripts = DBProjectFlagModule::list(i64::MAX, 0, &self.ro_pool)
            .await?
            .into_iter()
            .map(|module| (module.name, module.script))
            .collect();
        Ok(self.with_modules(scripts))
    }

    pub fn with_modules(&self, scripts: HashMap<String, String>) -> Self {
        Self {
            rhai: Arc::new(Self::script_engine(scripts)),
            ..self.clone()
        }
    }

    fn script_engine(scripts: HashMap<String, String>) -> rhai::Engine {
        let mut rhai = rhai::Engine::new();
        ChronoPackage::new().register_into_engine(&mut rhai);
        UrlPackage::new().register_into_engine(&mut rhai);
        rhai.set_module_resolver(DatabaseModuleResolver { scripts });
        rhai.set_max_modules(128);
        rhai.set_max_expr_depths(64, 64);
        rhai.disable_symbol("eval"); // eval considered evil
        rhai.set_max_operations(1_000_000);
        rhai.set_max_call_levels(64);
        rhai.set_max_string_size(1_048_576);
        rhai.set_max_array_size(10_000);
        rhai.set_max_map_size(10_000);
        rhai.on_print(|message| {
            EVALUATION_LOG.with(|log| {
                if let Some(log) = log.borrow_mut().as_mut() {
                    log.push(message, ENV.PROJECT_FLAG_RULE_MAX_LOG_BYTES);
                }
            });
        });
        rhai
    }

    pub fn validate_module_script(&self, script: &str) -> Result<()> {
        self.rhai
            .compile(script)
            .context("compiling module script")?;
        Ok(())
    }

    pub fn make_call(
        &self,
        params: &ProjectRuleParams,
    ) -> Result<ProjectRuleCall> {
        params.validate_decision_questions()?;
        let ast = self
            .rhai
            .compile(&params.expression)
            .context("compiling script")?;
        Ok(ProjectRuleCall {
            ast,
            scan_files_for: params.scan_files_for.clone(),
            decision_questions: params.decision_questions.clone(),
        })
    }

    pub async fn load_context(
        &self,
        project_id: DBProjectId,
    ) -> Result<Option<FlagContext>> {
        Ok(self.load_contexts(&[project_id]).await?.remove(&project_id))
    }

    pub(crate) async fn load_contexts(
        &self,
        project_ids: &[DBProjectId],
    ) -> Result<HashMap<DBProjectId, FlagContext>> {
        let rules = DBProjectFlagRule::all(&self.ro_pool).await?;
        self.load_contexts_for_rules(project_ids, &rules).await
    }

    pub(crate) async fn load_contexts_for_rules(
        &self,
        project_ids: &[DBProjectId],
        rules: &[DBProjectFlagRule],
    ) -> Result<HashMap<DBProjectId, FlagContext>> {
        self.load_contexts_from_source(
            project_ids,
            rules,
            ContextSource::Cached,
        )
        .await
    }

    async fn load_contexts_from_source(
        &self,
        project_ids: &[DBProjectId],
        rules: &[DBProjectFlagRule],
        source: ContextSource,
    ) -> Result<HashMap<DBProjectId, FlagContext>> {
        if project_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let projects = match source {
            ContextSource::Cached => {
                DBProject::get_many_ids(
                    project_ids,
                    &*self.ro_pool,
                    &self.redis,
                )
                .await
            }
            ContextSource::Authoritative => {
                let ids = project_ids
                    .iter()
                    .copied()
                    .map(crate::models::ids::ProjectId::from)
                    .collect::<Vec<_>>();
                DBProject::get_many_uncached(&ids, &*self.ro_pool, &self.redis)
                    .await
            }
        }
        .context("fetching projects for flag evaluation")?;
        let ids = project_ids.iter().map(|id| id.0).collect::<Vec<_>>();
        let version_ids = sqlx::query_scalar!(
            r#"SELECT id FROM versions WHERE mod_id = ANY($1) ORDER BY date_published DESC, id DESC"#,
            &ids,
        ).fetch_all(&*self.ro_pool).await
            .context("fetching version IDs for flag evaluation")?
            .into_iter().map(DBVersionId).collect::<Vec<_>>();
        let mut versions = HashMap::<_, Vec<Version>>::new();
        let loaded_versions = match source {
            ContextSource::Cached => {
                DBVersion::get_many(&version_ids, &*self.ro_pool, &self.redis)
                    .await
            }
            ContextSource::Authoritative => {
                DBVersion::get_many_uncached(
                    &version_ids,
                    &*self.ro_pool,
                    &self.redis,
                )
                .await
            }
        }
        .context("fetching versions for flag evaluation")?;
        for version in loaded_versions {
            versions
                .entry(version.inner.project_id)
                .or_default()
                .push(Version::from(version));
        }
        for versions in versions.values_mut() {
            sort_versions_newest_first(versions);
        }
        let mut disclosures = HashMap::<_, Vec<ProjectDisclosureData>>::new();
        for disclosure in DBProjectDisclosure::get_many_for_projects(
            project_ids,
            false,
            &*self.ro_pool,
        )
        .await
        .context("fetching disclosures for flag evaluation")?
        {
            disclosures
                .entry(disclosure.project_id)
                .or_default()
                .push(ProjectDisclosureData::from_db(disclosure, true, true));
        }
        let mut attributions =
            load_attributions(project_ids, &self.ro_pool).await?;
        let thread_ids = projects
            .iter()
            .map(|project| project.thread_id)
            .collect::<Vec<_>>();
        let mut threads = DBThread::get_many(&thread_ids, &*self.ro_pool)
            .await
            .context("fetching moderation threads for flag evaluation")?
            .into_iter()
            .map(|thread| (thread.id, thread))
            .collect::<HashMap<_, _>>();
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
        .context("fetching organizations for flag evaluation")?
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
        .context("fetching team members for flag evaluation")?;
        let mut user_ids =
            members.iter().map(|m| m.user_id).collect::<Vec<_>>();
        user_ids.sort_unstable_by_key(|id| id.0);
        user_ids.dedup();
        let users =
            DBUser::get_many_ids(&user_ids, &*self.ro_pool, &self.redis)
                .await
                .context("fetching team users for flag evaluation")?
                .into_iter()
                .map(|user| (user.id, User::from(user)))
                .collect::<HashMap<_, _>>();
        let mut owner_ids = members
            .iter()
            .filter(|member| member.is_owner && member.accepted)
            .map(|member| member.user_id.0)
            .collect::<Vec<_>>();
        owner_ids.sort_unstable();
        owner_ids.dedup();
        let ownership = sqlx::query!(
            r#"
            SELECT tm.user_id, m.id
            FROM team_members tm
            INNER JOIN mods m ON m.team_id = tm.team_id
            WHERE tm.user_id = ANY($1)
                AND tm.is_owner = TRUE AND tm.accepted = TRUE
                AND m.organization_id IS NULL
            ORDER BY m.downloads DESC, m.id
            "#,
            &owner_ids,
        )
        .fetch_all(&*self.ro_pool)
        .await
        .context("fetching owner project IDs for flag evaluation")?;
        let mut owned_project_ids = ownership
            .iter()
            .map(|row| DBProjectId(row.id))
            .collect::<Vec<_>>();
        owned_project_ids.sort_unstable_by_key(|id| id.0);
        owned_project_ids.dedup();
        let owned_projects = DBProject::get_many_ids(
            &owned_project_ids,
            &*self.ro_pool,
            &self.redis,
        )
        .await
        .context("fetching owner projects for flag evaluation")?
        .into_iter()
        .map(|project| (project.inner.id, Project::from(project)))
        .collect::<HashMap<_, _>>();
        let mut user_projects = HashMap::<_, Vec<Project>>::new();
        for row in ownership {
            if let Some(project) = owned_projects.get(&DBProjectId(row.id)) {
                user_projects
                    .entry(row.user_id)
                    .or_default()
                    .push(project.clone());
            }
        }
        let mut teams = HashMap::<_, Vec<TeamMember>>::new();
        for member in members {
            let user = users
                .get(&member.user_id)
                .cloned()
                .ok_or_else(|| anyhow!("team member user not found"))?;
            teams
                .entry(member.team_id)
                .or_default()
                .push(TeamMember::from_model(member, user, true));
        }
        let mut contexts = HashMap::new();
        for project in projects {
            let id = project.inner.id;
            let team_id = project.inner.team_id;
            let flag_members = |team_id| {
                teams
                    .get(&team_id)
                    .into_iter()
                    .flatten()
                    .cloned()
                    .map(|member| {
                        let projects = if member.is_owner && member.accepted {
                            user_projects
                                .get(
                                    &crate::database::models::DBUserId::from(
                                        member.user.id,
                                    )
                                    .0,
                                )
                                .into_iter()
                                .flatten()
                                .filter(|project| project.id != id.into())
                                .cloned()
                                .collect()
                        } else {
                            Vec::new()
                        };
                        FlagTeamMember::new(member, projects)
                    })
                    .collect::<Vec<_>>()
            };
            let members = flag_members(team_id);
            let organization = project
                .inner
                .organization_id
                .map(|id| {
                    let organization =
                        organizations.get(&id).cloned().ok_or_else(|| {
                            anyhow!("project organization not found")
                        })?;
                    let members = flag_members(organization.team_id);
                    let organization =
                        Organization::from(organization, Vec::new());
                    anyhow::Ok(FlagOrganization {
                        id: organization.id,
                        slug: organization.slug,
                        name: organization.name,
                        team_id: organization.team_id,
                        description: organization.description,
                        icon_url: organization.icon_url,
                        raw_icon_url: organization.raw_icon_url,
                        color: organization.color,
                        members,
                        moderation_notes: organization.moderation_notes,
                    })
                })
                .transpose()?;
            let thread =
                threads.remove(&project.thread_id).ok_or_else(|| {
                    anyhow!("project moderation thread not found")
                })?;
            let moderation_thread = FlagModerationThread {
                id: thread.id.into(),
                messages: thread
                    .messages
                    .into_iter()
                    .map(|message| ThreadMessage {
                        id: message.id.into(),
                        author_id: message.author_id.map(Into::into),
                        body: message.body,
                        created: message.created,
                        hide_identity: message.hide_identity,
                    })
                    .collect(),
            };
            contexts.insert(
                id,
                FlagContext {
                    ai: FlagAiContext::default(),
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
                    versions: versions.remove(&id).unwrap_or_default(),
                    disclosures: disclosures.remove(&id).unwrap_or_default(),
                    attributions: attributions.remove(&id).unwrap_or_default(),
                    moderation_thread,
                },
            );
        }

        for (id, context) in &mut contexts {
            self.load_ai(context, *id, rules).await?;
        }
        Ok(contexts)
    }

    pub(crate) async fn load_decisions(
        &self,
        context: &FlagContext,
        questions: &HashMap<String, DecisionQuestion>,
    ) -> Result<BTreeMap<String, Option<f64>>> {
        use crate::workflow::project_flag::{
            scan_cache::load_scan,
            text_scan::{ProjectText, decode_result, text_input_hash},
        };
        let content =
            ProjectText::from_project(&context.project, &context.versions)?;
        let version = self
            .scanner_config
            .infer_decisions
            .as_ref()
            .map(|c| i64::try_from(c.model_version))
            .transpose()
            .context("invalid decisions model version")?;
        let mut answers = BTreeMap::new();
        for (name, question) in questions {
            let hash = text_input_hash(&content, question)
                .map_err(|e| anyhow!("{e:#}"))?;
            let answer = if let Some(version) = version {
                load_scan(
                    &self.ro_pool,
                    ProjectFlagScanKind::Text,
                    hash.as_bytes(),
                    version,
                )
                .await
                .map_err(|e| anyhow!("{e:#}"))?
                .map(decode_result)
                .transpose()
                .map_err(|e| anyhow!("{e:#}"))?
                .map(|r| r.probability)
            } else {
                None
            };
            answers.insert(name.clone(), answer);
        }
        Ok(answers)
    }

    /// Reloads the source project and versions without trusting Redis. Related
    /// team/user context retains the normal loader's caching behavior.
    pub(crate) async fn load_reconciliation_contexts(
        &self,
        project_ids: &[DBProjectId],
        rules: &[DBProjectFlagRule],
    ) -> Result<HashMap<DBProjectId, FlagContext>> {
        self.load_contexts_from_source(
            project_ids,
            rules,
            ContextSource::Authoritative,
        )
        .await
    }

    pub(crate) async fn apply_reconciliation(
        &self,
        pool: &crate::database::PgPool,
        context: &FlagContext,
        result: &crate::database::models::project_flag_item::DBProjectFlagRunResult,
        ruleset_id: uuid::Uuid,
        file_manifest_hash: blake3::Hash,
        approval_due: bool,
        approval_workflow: Option<&temporalio_client::Client>,
    ) -> Result<()> {
        crate::routes::v3::projects::mutation::apply_project_flag_reconciliation_after_delay(
            context, result, ruleset_id, file_manifest_hash, pool, &self.redis,
                        approval_due && self.scanner_config.auto_approve_enabled,
                        approval_workflow, self.scanner_config.auto_withhold_enabled,
        ).await.map_err(|error| match error {
            crate::routes::ApiError::Internal(report) => report.context("applying project flag reconciliation"),
            error => anyhow!("applying project flag reconciliation: {error}"),
        })
    }

    pub(crate) fn validate_reconciliation_ai(
        &self,
        context: &FlagContext,
        rules: &[DBProjectFlagRule],
    ) -> Result<()> {
        for rule in rules {
            rule.params.validate_decision_questions()?;
            let rule_id = crate::models::ids::ProjectFlagRuleId::from(rule.id)
                .to_string();
            for question in rule.params.decision_questions.keys() {
                validate_reconciliation_decision(
                    &context.ai,
                    &rule_id,
                    question,
                    self.scanner_config.infer_decisions.is_some(),
                )?;
            }
        }
        for image in &context.ai.images {
            for (kind, required, answer) in [
                (
                    ProjectFlagScanKind::Explicit,
                    self.scanner_config.infer_decisions.is_some(),
                    image.explicit,
                ),
                (
                    ProjectFlagScanKind::AiGenerated,
                    self.scanner_config.infer_ai_generated.is_some(),
                    image.ai_generated,
                ),
            ] {
                if required {
                    require_probability(answer).with_context(|| {
                        format!("missing or invalid required `{kind}` image answer for `{}`", image.url)
                    })?;
                }
            }
        }
        Ok(())
    }

    pub(crate) async fn load_ai(
        &self,
        context: &mut FlagContext,
        id: DBProjectId,
        rules: &[DBProjectFlagRule],
    ) -> Result<()> {
        use crate::workflow::project_flag::{
            image_scan::{
                image_input_hash, image_scan_probability, project_images,
            },
            scan_cache::load_scan,
        };
        context.ai = FlagAiContext::default();
        for image in project_images(&self.ro_pool, id.into())
            .await
            .map_err(|e| anyhow!("{e:#}"))?
        {
            let mut result = FlagAiImage {
                url: image.url.to_string(),
                explicit: None,
                ai_generated: None,
            };
            for (kind, version, target) in [
                (
                    ProjectFlagScanKind::Explicit,
                    self.scanner_config
                        .infer_decisions
                        .as_ref()
                        .map(|c| c.model_version),
                    &mut result.explicit,
                ),
                (
                    ProjectFlagScanKind::AiGenerated,
                    self.scanner_config
                        .infer_ai_generated
                        .as_ref()
                        .map(|c| c.model_version),
                    &mut result.ai_generated,
                ),
            ] {
                if let Some(version) = version {
                    let version = i64::try_from(version)
                        .context("invalid image model version")?;
                    let hash = image_input_hash(
                        image.url.as_str(),
                        kind,
                        &self.scanner_config,
                    );
                    *target = load_scan(
                        &self.ro_pool,
                        kind,
                        hash.as_bytes(),
                        version,
                    )
                    .await
                    .map_err(|e| anyhow!("{e:#}"))?
                    .as_ref()
                    .map(image_scan_probability)
                    .transpose()
                    .map_err(|e| anyhow!("{e:#}"))?;
                }
            }
            context.ai.images.push(result);
        }
        for rule in rules {
            context.ai.decisions_by_rule.insert(
                crate::models::ids::ProjectFlagRuleId::from(rule.id)
                    .to_string(),
                self.load_decisions(context, &rule.params.decision_questions)
                    .await?,
            );
        }
        Ok(())
    }

    pub(crate) async fn load_files(
        &self,
        project_ids: &[DBProjectId],
    ) -> Result<HashMap<DBProjectId, Vec<ProjectScanFile>>> {
        load_project_files(project_ids, &self.ro_pool).await
    }

    pub(crate) async fn prepare_context(
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
        let (flags, log) = self.evaluate_with_log(call, context);
        Ok(ProjectFlagEvaluation { flags: flags?, log })
    }

    pub(super) fn evaluate_with_log(
        &self,
        call: &ProjectRuleCall,
        context: &FlagContext,
    ) -> (Result<Vec<ProjectFlag>>, Vec<String>) {
        self.evaluate_with_scan_error(call, context, None)
    }

    pub(crate) fn evaluate_with_scan_error(
        &self,
        call: &ProjectRuleCall,
        context: &FlagContext,
        scan_error: Option<&str>,
    ) -> (Result<Vec<ProjectFlag>>, Vec<String>) {
        MODULE_COUNT.with(|count| count.set(0));
        let log = EvaluationLog::start();
        let flags = (|| {
            if let Some(error) = scan_error
                && !call.scan_files_for.is_empty()
            {
                return Err(anyhow!("project file scan failed: {error}"));
            }
            let context = rhai::serde::to_dynamic(context)
                .context("serializing flag context for script")?
                .try_cast::<rhai::Map>()
                .ok_or_else(|| {
                    anyhow!("flag context must serialize to a map")
                })?;
            let mut scope = rhai::Scope::new();
            for (name, value) in context {
                if scan_error.is_some() && name == "scanned_files" {
                    continue;
                }
                scope.push_dynamic(name.to_string(), value);
            }
            let output = self
                .rhai
                .eval_ast_with_scope::<rhai::Array>(&mut scope, &call.ast)
                .context("running script");
            let output = match scan_error {
                Some(error) => output.with_context(|| {
                    format!("file scan unavailable for script: {error}")
                }),
                None => output,
            }?;
            output
                .into_iter()
                .map(|value| {
                    let value =
                        rhai::serde::from_dynamic::<serde_json::Value>(&value)
                            .context("converting project flag to json")?;
                    serde_json::from_value::<ProjectFlag>(value)
                        .context("decoding project flag")
                })
                .collect::<Result<Vec<_>>>()
        })();
        (flags, log.take())
    }

    pub async fn scan_flags_with_log(
        &self,
        call: &ProjectRuleCall,
        project_id: DBProjectId,
    ) -> Result<ProjectFlagEvaluation> {
        tokio::time::timeout(
            Duration::from_millis(ENV.PROJECT_FLAG_RULE_TIMEOUT),
            async {
                let mut context = self
                    .load_contexts_for_rules(&[project_id], &[])
                    .await?
                    .remove(&project_id)
                    .ok_or_else(|| anyhow!("project not found"))?;
                context.ai.decisions = self
                    .load_decisions(&context, &call.decision_questions)
                    .await?;
                let files = self
                    .load_files(&[project_id])
                    .await?
                    .remove(&project_id)
                    .unwrap_or_default();
                let (context, scan_error) = match self
                    .prepare_context(
                        context.clone(),
                        &files,
                        &call.scan_files_for,
                    )
                    .await
                {
                    Ok(context) => (context, None),
                    Err(error) => (context, Some(format!("{error:#}"))),
                };
                let engine = self.clone();
                let call = call.clone();
                tokio::task::spawn_blocking(move || {
                    let (flags, log) = engine.evaluate_with_scan_error(
                        &call,
                        &context,
                        scan_error.as_deref(),
                    );
                    Ok(ProjectFlagEvaluation { flags: flags?, log })
                })
                .await
                .context("joining project flag evaluation")?
            },
        )
        .await
        .context("project flag rule evaluation timed out")?
    }

    pub async fn scan_flags(
        &self,
        call: &ProjectRuleCall,
        project_id: DBProjectId,
    ) -> Result<Vec<ProjectFlag>> {
        Ok(self.scan_flags_with_log(call, project_id).await?.flags)
    }
}

fn validate_reconciliation_decision(
    ai: &FlagAiContext,
    rule_id: &str,
    question: &str,
    model_configured: bool,
) -> Result<()> {
    if model_configured
        && ai
            .decisions_by_rule
            .get(rule_id)
            .and_then(|answers| answers.get(question))
            == Some(&None)
    {
        return Err(anyhow!(crate::workflow::project_flag::run_rules::ReconciliationInputsChanged)
            .context(format!("missing current decision cache for rule `{rule_id}` question `{question}`")));
    }
    require_decision(ai, rule_id, question)
}

fn require_decision(
    ai: &FlagAiContext,
    rule_id: &str,
    question: &str,
) -> Result<()> {
    let answer = ai
        .decisions_by_rule
        .get(rule_id)
        .and_then(|answers| answers.get(question))
        .copied()
        .flatten();
    require_probability(answer).with_context(|| {
        format!("missing or invalid required AI answer for rule `{rule_id}` question `{question}`")
    })
}

fn require_probability(answer: Option<f64>) -> Result<()> {
    match answer {
        Some(value) if value.is_finite() && (0.0..=1.0).contains(&value) => {
            Ok(())
        }
        _ => Err(anyhow!("required AI probability is missing or invalid")),
    }
}

#[cfg(test)]
mod ai_context_tests {
    use super::*;

    #[test]
    fn reconciliation_replans_after_selected_context_changes_since_scan() {
        use crate::database::models::project_flag_item::ContextField;
        use crate::workflow::project_flag::{
            run_rules::ReconciliationInputsChanged,
            text_scan::{ProjectText, text_input_hash},
        };

        let mut content = ProjectText {
            title: "scanned title".into(),
            summary: String::new(),
            description: String::new(),
            links: serde_json::json!({}),
            text_fields: BTreeMap::new(),
        };
        let question = DecisionQuestion {
            question: "Question?".into(),
            include_context: vec![ContextField::Title],
        };
        let scanned_hash = text_input_hash(&content, &question).unwrap();
        let cache = HashMap::from([(scanned_hash, 0.5)]);
        let mut ai = FlagAiContext::default();
        for (title, replan) in
            [("scanned title", false), ("edited title", true)]
        {
            content.title = title.into();
            let hash = text_input_hash(&content, &question).unwrap();
            ai.decisions_by_rule.insert(
                "1".into(),
                BTreeMap::from([(
                    "question".into(),
                    cache.get(&hash).copied(),
                )]),
            );
            let result =
                validate_reconciliation_decision(&ai, "1", "question", true);
            if replan {
                assert!(
                    result.unwrap_err().is::<ReconciliationInputsChanged>()
                );
            } else {
                result.unwrap();
            }
        }
    }

    #[test]
    fn reconciliation_only_replans_expected_missing_enabled_answers() {
        use crate::workflow::project_flag::run_rules::ReconciliationInputsChanged;

        let mut ai = FlagAiContext::default();
        assert!(
            !validate_reconciliation_decision(&ai, "1", "question", true)
                .unwrap_err()
                .is::<ReconciliationInputsChanged>()
        );
        ai.decisions_by_rule
            .insert("1".into(), BTreeMap::from([("question".into(), None)]));
        assert!(
            validate_reconciliation_decision(&ai, "1", "question", true)
                .unwrap_err()
                .is::<ReconciliationInputsChanged>()
        );
        assert!(
            !validate_reconciliation_decision(&ai, "1", "question", false)
                .unwrap_err()
                .is::<ReconciliationInputsChanged>()
        );
        for value in [f64::NAN, -0.1, 1.1] {
            ai.decisions_by_rule
                .get_mut("1")
                .unwrap()
                .insert("question".into(), Some(value));
            assert!(
                !validate_reconciliation_decision(&ai, "1", "question", true)
                    .unwrap_err()
                    .is::<ReconciliationInputsChanged>()
            );
        }
        ai.decisions_by_rule
            .get_mut("1")
            .unwrap()
            .insert("question".into(), Some(0.5));
        assert!(
            validate_reconciliation_decision(&ai, "1", "question", true)
                .is_ok()
        );
    }

    #[test]
    fn required_decisions_must_exist_in_the_correct_rule_namespace() {
        let mut ai = FlagAiContext::default();
        assert!(require_decision(&ai, "1", "question").is_err());
        ai.decisions_by_rule.insert(
            "2".into(),
            BTreeMap::from([("question".into(), Some(0.5))]),
        );
        assert!(require_decision(&ai, "1", "question").is_err());
        ai.decisions_by_rule
            .insert("1".into(), BTreeMap::from([("question".into(), None)]));
        assert!(require_decision(&ai, "1", "question").is_err());
        ai.decisions_by_rule
            .get_mut("1")
            .unwrap()
            .insert("question".into(), Some(0.0));
        assert!(require_decision(&ai, "1", "question").is_ok());
        assert!(require_decision(&ai, "1", "other").is_err());
    }

    #[test]
    fn required_probabilities_reject_missing_and_invalid_answers() {
        for answer in [
            None,
            Some(f64::NAN),
            Some(f64::INFINITY),
            Some(-0.1),
            Some(1.1),
        ] {
            assert!(require_probability(answer).is_err());
        }
        for probability in [0.0, 0.5, 1.0] {
            assert!(require_probability(Some(probability)).is_ok());
        }
    }

    #[test]
    fn absent_probabilities_serialize_as_null() {
        let image = FlagAiImage {
            url: "https://cdn.example/image.png".into(),
            explicit: None,
            ai_generated: None,
        };
        let value = serde_json::to_value(image).unwrap();
        assert_eq!(value["explicit"], serde_json::Value::Null);
        assert_eq!(value["ai_generated"], serde_json::Value::Null);
    }

    #[test]
    fn rule_namespaces_preserve_colliding_question_names() {
        let mut ai = FlagAiContext::default();
        ai.decisions_by_rule.insert(
            "1".into(),
            BTreeMap::from([("question".into(), Some(0.1))]),
        );
        ai.decisions_by_rule.insert(
            "2".into(),
            BTreeMap::from([("question".into(), Some(0.9))]),
        );
        ai.decisions = ai.decisions_by_rule["1"].clone();
        assert_eq!(ai.decisions["question"], Some(0.1));
        ai.decisions = ai.decisions_by_rule["2"].clone();
        assert_eq!(ai.decisions["question"], Some(0.9));
    }
}
