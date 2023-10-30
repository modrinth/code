#![allow(dead_code)]
use actix_http::StatusCode;
use actix_web::test::{self, TestRequest};
use labrinth::{
    models::projects::Project,
    models::{
        oauth_clients::OAuthClient, organizations::Organization, pats::Scopes, projects::Version,
    },
};
use serde_json::json;
use sqlx::Executor;

use crate::common::{actix::AppendsMultipart, database::USER_USER_PAT};

use super::{
    actix::{MultipartSegment, MultipartSegmentData},
    asserts::assert_status,
    database::USER_USER_ID,
    environment::TestEnvironment,
    get_json_val_str,
    request_data::get_public_project_creation_data,
};

pub const DUMMY_DATA_UPDATE: i64 = 3;

#[allow(dead_code)]
pub const DUMMY_CATEGORIES: &[&str] = &[
    "combat",
    "decoration",
    "economy",
    "food",
    "magic",
    "mobs",
    "optimization",
];

pub const DUMMY_OAUTH_CLIENT_ALPHA_SECRET: &str = "abcdefghijklmnopqrstuvwxyz";

#[allow(dead_code)]
pub enum DummyJarFile {
    DummyProjectAlpha,
    DummyProjectBeta,
    BasicMod,
    BasicModDifferent,
}

#[allow(dead_code)]
pub enum DummyImage {
    SmallIcon, // 200x200
}

#[derive(Clone)]
pub struct DummyData {
    /// Alpha project:
    /// This is a dummy project created by USER user.
    /// It's approved, listed, and visible to the public.
    pub project_alpha: DummyProjectAlpha,

    /// Beta project:
    /// This is a dummy project created by USER user.
    /// It's not approved, unlisted, and not visible to the public.
    pub project_beta: DummyProjectBeta,

    /// Zeta organization:
    /// This is a dummy organization created by USER user.
    /// There are no projects in it.
    pub organization_zeta: DummyOrganizationZeta,

    /// Alpha OAuth Client:
    /// This is a dummy OAuth client created by USER user.
    ///
    /// All scopes are included in its max scopes
    ///
    /// It has one valid redirect URI
    pub oauth_client_alpha: DummyOAuthClientAlpha,
}

impl DummyData {
    pub fn new(
        project_alpha: Project,
        project_alpha_version: Version,
        project_beta: Project,
        project_beta_version: Version,
        organization_zeta: Organization,
        oauth_client_alpha: OAuthClient,
    ) -> Self {
        DummyData {
            project_alpha: DummyProjectAlpha {
                team_id: project_alpha.team.to_string(),
                project_id: project_alpha.id.to_string(),
                project_slug: project_alpha.slug.unwrap(),
                version_id: project_alpha_version.id.to_string(),
                thread_id: project_alpha.thread_id.to_string(),
                file_hash: project_alpha_version.files[0].hashes["sha1"].clone(),
            },

            project_beta: DummyProjectBeta {
                team_id: project_beta.team.to_string(),
                project_id: project_beta.id.to_string(),
                project_slug: project_beta.slug.unwrap(),
                version_id: project_beta_version.id.to_string(),
                thread_id: project_beta.thread_id.to_string(),
                file_hash: project_beta_version.files[0].hashes["sha1"].clone(),
            },

            organization_zeta: DummyOrganizationZeta {
                organization_id: organization_zeta.id.to_string(),
                team_id: organization_zeta.team_id.to_string(),
                organization_title: organization_zeta.title,
            },

            oauth_client_alpha: DummyOAuthClientAlpha {
                client_id: get_json_val_str(oauth_client_alpha.id),
                client_secret: DUMMY_OAUTH_CLIENT_ALPHA_SECRET.to_string(),
                valid_redirect_uri: oauth_client_alpha
                    .redirect_uris
                    .first()
                    .unwrap()
                    .uri
                    .clone(),
            },
        }
    }
}

#[derive(Clone)]
pub struct DummyProjectAlpha {
    pub project_id: String,
    pub project_slug: String,
    pub version_id: String,
    pub thread_id: String,
    pub file_hash: String,
    pub team_id: String,
}

#[derive(Clone)]
pub struct DummyProjectBeta {
    pub project_id: String,
    pub project_slug: String,
    pub version_id: String,
    pub thread_id: String,
    pub file_hash: String,
    pub team_id: String,
}

#[derive(Clone)]
pub struct DummyOrganizationZeta {
    pub organization_id: String,
    pub organization_title: String,
    pub team_id: String,
}

#[derive(Clone)]
pub struct DummyOAuthClientAlpha {
    pub client_id: String,
    pub client_secret: String,
    pub valid_redirect_uri: String,
}

pub async fn add_dummy_data(test_env: &TestEnvironment) -> DummyData {
    // Adds basic dummy data to the database directly with sql (user, pats)
    let pool = &test_env.db.pool.clone();

    pool.execute(
        include_str!("../files/dummy_data.sql")
            .replace("$1", &Scopes::all().bits().to_string())
            .as_str(),
    )
    .await
    .unwrap();

    let (alpha_project, alpha_version) = add_project_alpha(test_env).await;
    let (beta_project, beta_version) = add_project_beta(test_env).await;

    let zeta_organization = add_organization_zeta(test_env).await;

    let oauth_client_alpha = get_oauth_client_alpha(test_env).await;

    sqlx::query("INSERT INTO dummy_data (update_id) VALUES ($1)")
        .bind(DUMMY_DATA_UPDATE)
        .execute(pool)
        .await
        .unwrap();

    DummyData::new(
        alpha_project,
        alpha_version,
        beta_project,
        beta_version,
        zeta_organization,
        oauth_client_alpha,
    )
}

pub async fn get_dummy_data(test_env: &TestEnvironment) -> DummyData {
    let (alpha_project, alpha_version) = get_project_alpha(test_env).await;
    let (beta_project, beta_version) = get_project_beta(test_env).await;

    let zeta_organization = get_organization_zeta(test_env).await;

    let oauth_client_alpha = get_oauth_client_alpha(test_env).await;

    DummyData::new(
        alpha_project,
        alpha_version,
        beta_project,
        beta_version,
        zeta_organization,
        oauth_client_alpha,
    )
}

pub async fn add_project_alpha(test_env: &TestEnvironment) -> (Project, Version) {
    let (project, versions) = test_env
        .v2
        .add_public_project(
            get_public_project_creation_data("alpha", Some(DummyJarFile::DummyProjectAlpha)),
            USER_USER_PAT,
        )
        .await;
    (project, versions.into_iter().next().unwrap())
}

pub async fn add_project_beta(test_env: &TestEnvironment) -> (Project, Version) {
    // Adds dummy data to the database with sqlx (projects, versions, threads)
    // Generate test project data.
    let jar = DummyJarFile::DummyProjectBeta;
    let json_data = json!(
        {
            "title": "Test Project Beta",
            "slug": "beta",
            "description": "A dummy project for testing with.",
            "body": "This project is not-yet-approved, and versions are draft.",
            "client_side": "required",
            "server_side": "optional",
            "initial_versions": [{
                "file_parts": [jar.filename()],
                "version_number": "1.2.3",
                "version_title": "start",
                "status": "unlisted",
                "requested_status": "unlisted",
                "dependencies": [],
                "game_versions": ["1.20.1"] ,
                "release_channel": "release",
                "loaders": ["fabric"],
                "featured": true
            }],
            "status": "private",
            "requested_status": "private",
            "categories": [],
            "license_id": "MIT"
        }
    );

    // Basic json
    let json_segment = MultipartSegment {
        name: "data".to_string(),
        filename: None,
        content_type: Some("application/json".to_string()),
        data: MultipartSegmentData::Text(serde_json::to_string(&json_data).unwrap()),
    };

    // Basic file
    let file_segment = MultipartSegment {
        name: jar.filename(),
        filename: Some(jar.filename()),
        content_type: Some("application/java-archive".to_string()),
        data: MultipartSegmentData::Binary(jar.bytes()),
    };

    // Add a project.
    let req = TestRequest::post()
        .uri("/v2/project")
        .append_header(("Authorization", USER_USER_PAT))
        .set_multipart(vec![json_segment.clone(), file_segment.clone()])
        .to_request();
    let resp = test_env.call(req).await;

    assert_eq!(resp.status(), 200);

    get_project_beta(test_env).await
}

pub async fn add_organization_zeta(test_env: &TestEnvironment) -> Organization {
    // Add an organzation.
    let req = TestRequest::post()
        .uri("/v2/organization")
        .append_header(("Authorization", USER_USER_PAT))
        .set_json(json!({
            "title": "zeta",
            "description": "A dummy organization for testing with."
        }))
        .to_request();
    let resp = test_env.call(req).await;

    assert_eq!(resp.status(), 200);

    get_organization_zeta(test_env).await
}

pub async fn get_project_alpha(test_env: &TestEnvironment) -> (Project, Version) {
    // Get project
    let req = TestRequest::get()
        .uri("/v2/project/alpha")
        .append_header(("Authorization", USER_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    let project: Project = test::read_body_json(resp).await;

    // Get project's versions
    let req = TestRequest::get()
        .uri("/v2/project/alpha/version")
        .append_header(("Authorization", USER_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    let versions: Vec<Version> = test::read_body_json(resp).await;
    let version = versions.into_iter().next().unwrap();

    (project, version)
}

pub async fn get_project_beta(test_env: &TestEnvironment) -> (Project, Version) {
    // Get project
    let req = TestRequest::get()
        .uri("/v2/project/beta")
        .append_header(("Authorization", USER_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    assert_status(&resp, StatusCode::OK);
    let project: Project = test::read_body_json(resp).await;

    // Get project's versions
    let req = TestRequest::get()
        .uri("/v2/project/beta/version")
        .append_header(("Authorization", USER_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    assert_status(&resp, StatusCode::OK);
    let versions: Vec<Version> = test::read_body_json(resp).await;
    let version = versions.into_iter().next().unwrap();

    (project, version)
}

pub async fn get_organization_zeta(test_env: &TestEnvironment) -> Organization {
    // Get organization
    let req = TestRequest::get()
        .uri("/v2/organization/zeta")
        .append_header(("Authorization", USER_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    let organization: Organization = test::read_body_json(resp).await;

    organization
}

pub async fn get_oauth_client_alpha(test_env: &TestEnvironment) -> OAuthClient {
    let oauth_clients = test_env
        .v3
        .get_user_oauth_clients(USER_USER_ID, USER_USER_PAT)
        .await;
    oauth_clients.into_iter().next().unwrap()
}

impl DummyJarFile {
    pub fn filename(&self) -> String {
        match self {
            DummyJarFile::DummyProjectAlpha => "dummy-project-alpha.jar",
            DummyJarFile::DummyProjectBeta => "dummy-project-beta.jar",
            DummyJarFile::BasicMod => "basic-mod.jar",
            DummyJarFile::BasicModDifferent => "basic-mod-different.jar",
        }
        .to_string()
    }

    pub fn bytes(&self) -> Vec<u8> {
        match self {
            DummyJarFile::DummyProjectAlpha => {
                include_bytes!("../../tests/files/dummy-project-alpha.jar").to_vec()
            }
            DummyJarFile::DummyProjectBeta => {
                include_bytes!("../../tests/files/dummy-project-beta.jar").to_vec()
            }
            DummyJarFile::BasicMod => include_bytes!("../../tests/files/basic-mod.jar").to_vec(),
            DummyJarFile::BasicModDifferent => {
                include_bytes!("../../tests/files/basic-mod-different.jar").to_vec()
            }
        }
    }
}

impl DummyImage {
    pub fn filename(&self) -> String {
        match self {
            DummyImage::SmallIcon => "200x200.png",
        }
        .to_string()
    }

    pub fn extension(&self) -> String {
        match self {
            DummyImage::SmallIcon => "png",
        }
        .to_string()
    }

    pub fn bytes(&self) -> Vec<u8> {
        match self {
            DummyImage::SmallIcon => include_bytes!("../../tests/files/200x200.png").to_vec(),
        }
    }
}
