#![allow(dead_code)]
use std::io::{Cursor, Write};

use actix_http::StatusCode;
use actix_web::test::{self, TestRequest};
use labrinth::models::{
    oauth_clients::OAuthClient,
    organizations::Organization,
    pats::Scopes,
    v3::projects::{Project, Version},
};
use serde_json::json;
use sqlx::Executor;
use zip::{write::FileOptions, CompressionMethod, ZipWriter};

use crate::common::database::USER_USER_PAT;
use labrinth::util::actix::{AppendsMultipart, MultipartSegment, MultipartSegmentData};

use super::{api_v3::request_data::get_public_project_creation_data, environment::TestEnvironment};

use super::{asserts::assert_status, database::USER_USER_ID, get_json_val_str};

pub const DUMMY_DATA_UPDATE: i64 = 5;

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
pub enum TestFile {
    DummyProjectAlpha,
    DummyProjectBeta,
    BasicMod,
    BasicModDifferent,
    // Randomly generates a valid .jar with a random hash.
    // Unlike the other dummy jar files, this one is not a static file.
    // and BasicModRandom.bytes() will return a different file each time.
    BasicModRandom { filename: String, bytes: Vec<u8> },
    BasicModpackRandom { filename: String, bytes: Vec<u8> },
}

impl TestFile {
    pub fn build_random_jar() -> Self {
        let filename = format!("random-mod-{}.jar", rand::random::<u64>());

        let fabric_mod_json = serde_json::json!({
            "schemaVersion": 1,
            "id": filename,
            "version": "1.0.1",

            "name": filename,
            "description": "Does nothing",
            "authors": [
              "user"
            ],
            "contact": {
              "homepage": "https://www.modrinth.com",
              "sources": "https://www.modrinth.com",
              "issues": "https://www.modrinth.com"
            },

            "license": "MIT",
            "icon": "none.png",

            "environment": "client",
            "entrypoints": {
              "main": [
                "io.github.modrinth.Modrinth"
              ]
            },
            "depends": {
              "minecraft": ">=1.20-"
            }
          }
        )
        .to_string();

        // Create a simulated zip file
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut cursor);
            zip.start_file(
                "fabric.mod.json",
                FileOptions::default().compression_method(CompressionMethod::Stored),
            )
            .unwrap();
            zip.write_all(fabric_mod_json.as_bytes()).unwrap();
            zip.finish().unwrap();
        }
        let bytes = cursor.into_inner();

        TestFile::BasicModRandom { filename, bytes }
    }

    pub fn build_random_mrpack() -> Self {
        let filename = format!("random-modpack-{}.mrpack", rand::random::<u64>());

        let modrinth_index_json = serde_json::json!({
            "formatVersion": 1,
            "game": "minecraft",
            "versionId": "1.20.1-9.6",
            "name": filename,
            "files": [],
            "dependencies": {
                "fabric-loader": "0.14.22",
                "minecraft": "1.20.1"
            }
        }
        )
        .to_string();

        // Create a simulated zip file
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut cursor);
            zip.start_file(
                "modrinth.index.json",
                FileOptions::default().compression_method(CompressionMethod::Stored),
            )
            .unwrap();
            zip.write_all(modrinth_index_json.as_bytes()).unwrap();
            zip.finish().unwrap();
        }
        let bytes = cursor.into_inner();

        TestFile::BasicModpackRandom { filename, bytes }
    }
}

#[derive(Clone)]
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
        .v3
        .add_public_project(
            get_public_project_creation_data("alpha", Some(TestFile::DummyProjectAlpha)),
            USER_USER_PAT,
        )
        .await;
    (project, versions.into_iter().next().unwrap())
}

pub async fn add_project_beta(test_env: &TestEnvironment) -> (Project, Version) {
    // Adds dummy data to the database with sqlx (projects, versions, threads)
    // Generate test project data.
    let jar = TestFile::DummyProjectBeta;
    // TODO: this shouldnt be hardcoded (nor should other similar ones be)
    let json_data = json!(
        {
            "title": "Test Project Beta",
            "slug": "beta",
            "description": "A dummy project for testing with.",
            "body": "This project is not-yet-approved, and versions are draft.",
            "initial_versions": [{
                "file_parts": [jar.filename()],
                "version_number": "1.2.3",
                "version_title": "start",
                "status": "unlisted",
                "dependencies": [],
                "client_side": "required",
                "server_side": "optional",
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
        .uri("/v3/project")
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
        .uri("/v3/organization")
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
        .uri("/v3/project/alpha")
        .append_header(("Authorization", USER_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    let project: Project = test::read_body_json(resp).await;

    // Get project's versions
    let req = TestRequest::get()
        .uri("/v3/project/alpha/version")
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
        .uri("/v3/project/beta")
        .append_header(("Authorization", USER_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    assert_status(&resp, StatusCode::OK);
    let project: serde_json::Value = test::read_body_json(resp).await;
    let project: Project = serde_json::from_value(project).unwrap();

    // Get project's versions
    let req = TestRequest::get()
        .uri("/v3/project/beta/version")
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
        .uri("/v3/organization/zeta")
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

impl TestFile {
    pub fn filename(&self) -> String {
        match self {
            TestFile::DummyProjectAlpha => "dummy-project-alpha.jar",
            TestFile::DummyProjectBeta => "dummy-project-beta.jar",
            TestFile::BasicMod => "basic-mod.jar",
            TestFile::BasicModDifferent => "basic-mod-different.jar",
            TestFile::BasicModRandom { filename, .. } => filename,
            TestFile::BasicModpackRandom { filename, .. } => filename,
        }
        .to_string()
    }

    pub fn bytes(&self) -> Vec<u8> {
        match self {
            TestFile::DummyProjectAlpha => {
                include_bytes!("../../tests/files/dummy-project-alpha.jar").to_vec()
            }
            TestFile::DummyProjectBeta => {
                include_bytes!("../../tests/files/dummy-project-beta.jar").to_vec()
            }
            TestFile::BasicMod => include_bytes!("../../tests/files/basic-mod.jar").to_vec(),
            TestFile::BasicModDifferent => {
                include_bytes!("../../tests/files/basic-mod-different.jar").to_vec()
            }
            TestFile::BasicModRandom { bytes, .. } => bytes.clone(),
            TestFile::BasicModpackRandom { bytes, .. } => bytes.clone(),
        }
    }

    pub fn project_type(&self) -> String {
        match self {
            TestFile::DummyProjectAlpha => "mod",
            TestFile::DummyProjectBeta => "mod",
            TestFile::BasicMod => "mod",
            TestFile::BasicModDifferent => "mod",
            TestFile::BasicModRandom { .. } => "mod",

            TestFile::BasicModpackRandom { .. } => "modpack",
        }
        .to_string()
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
