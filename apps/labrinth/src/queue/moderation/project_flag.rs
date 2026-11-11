use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    sync::{Arc, LazyLock},
    time::Duration,
};

use eyre::{Result, eyre};
use regex::Regex;
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
            project_flag_item::{PatternName, ProjectFlag, ProjectRuleParams},
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
};

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
    pub project: Project,
    pub team: FlagTeam,
    pub users: Vec<User>,
    pub organization: Option<Organization>,
    pub scanned_files: Vec<ScannedFile>,
    pub versions: Vec<Version>,
    pub disclosures: Vec<ProjectDisclosureData>,
    pub attributions: Vec<FlagAttributionGroup>,
    pub moderation_thread: FlagModerationThread,
}

#[derive(Clone, serde::Serialize, utoipa::ToSchema)]
pub struct FlagModerationThread {
    pub id: ThreadId,
    pub messages: Vec<ThreadMessage>,
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
                    log.push(message, ENV.PROJECT_FLAG_RULE_MAX_LOG_BYTES);
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
        let ids = project_ids.iter().map(|id| id.0).collect::<Vec<_>>();
        let version_ids = sqlx::query_scalar!(
            r#"SELECT id FROM versions WHERE mod_id = ANY($1) ORDER BY date_published DESC, id DESC"#,
            &ids,
        ).fetch_all(&*self.ro_pool).await
            .wrap_err("fetching version IDs for flag evaluation")?
            .into_iter().map(DBVersionId).collect::<Vec<_>>();
        let mut versions = HashMap::<_, Vec<Version>>::new();
        for version in
            DBVersion::get_many(&version_ids, &*self.ro_pool, &self.redis)
                .await
                .wrap_err("fetching versions for flag evaluation")?
        {
            versions
                .entry(version.inner.project_id)
                .or_default()
                .push(Version::from(version));
        }
        for versions in versions.values_mut() {
            versions.sort_by(|a, b| {
                b.date_published
                    .cmp(&a.date_published)
                    .then_with(|| b.id.0.cmp(&a.id.0))
            });
        }
        let mut disclosures = HashMap::<_, Vec<ProjectDisclosureData>>::new();
        for disclosure in DBProjectDisclosure::get_many_for_projects(
            project_ids,
            false,
            &*self.ro_pool,
        )
        .await
        .wrap_err("fetching disclosures for flag evaluation")?
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
            .wrap_err("fetching moderation threads for flag evaluation")?
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
            let thread = threads
                .remove(&project.thread_id)
                .ok_or_else(|| eyre!("project moderation thread not found"))?;
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

    pub(super) fn evaluate_with_scan_error(
        &self,
        call: &ProjectRuleCall,
        context: &FlagContext,
        scan_error: Option<&str>,
    ) -> (Result<Vec<ProjectFlag>>, Vec<String>) {
        let log = EvaluationLog::start();
        let flags = (|| {
            if let Some(error) = scan_error
                && !call.scan_files_for.is_empty()
            {
                return Err(eyre!("project file scan failed: {error}"));
            }
            let context = rhai::serde::to_dynamic(context)
                .wrap_err("serializing flag context for script")?
                .try_cast::<rhai::Map>()
                .ok_or_else(|| eyre!("flag context must serialize to a map"))?;
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
                .wrap_err("running script");
            let output = match scan_error {
                Some(error) => output.wrap_err_with(|| {
                    format!("file scan unavailable for script: {error}")
                }),
                None => output,
            }?;
            output
                .into_iter()
                .map(|value| {
                    let value =
                        rhai::serde::from_dynamic::<serde_json::Value>(&value)
                            .wrap_err("converting project flag to json")?;
                    serde_json::from_value::<ProjectFlag>(value)
                        .wrap_err("decoding project flag")
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
    ) -> Result<Vec<ProjectFlag>> {
        Ok(self.scan_flags_with_log(call, project_id).await?.flags)
    }
}
