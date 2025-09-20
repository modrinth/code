use std::collections::HashMap;

use actix_web::{HttpRequest, web};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    auth::get_user_from_headers,
    database::redis::RedisPool,
    models::{
        ids::{AffiliateCodeId, ProjectId},
        pats::Scopes,
    },
    queue::session::AuthQueue,
    routes::ApiError,
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("analytics").route("", web::post().to(get)));
}

// request

#[derive(Debug, Serialize, Deserialize)]
struct GetRequest {
    time_range: Range,
    filters: Filters,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Range {}

#[derive(Debug, Serialize, Deserialize)]
struct Filters {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum AnalyticsKind {}

// response

#[derive(Debug, Serialize, Deserialize)]
struct GetResponse(Vec<TimeSlice>);

#[derive(Debug, Serialize, Deserialize)]
struct TimeSlice(Vec<AnalyticsData>);

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum AnalyticsData {
    Project(ProjectAnalytics),
    AffiliateCode(AffiliateCodeAnalytics),
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectAnalytics {
    source_project: ProjectId,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    game_versions: HashMap<String, u64>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    game_loaders: HashMap<String, u64>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    revenue: HashMap<RevenueSource, u64>,
}

impl ProjectAnalytics {
    fn new(source_project: ProjectId) -> Self {
        Self {
            source_project,
            game_versions: HashMap::new(),
            game_loaders: HashMap::new(),
            revenue: HashMap::new(),
        }
    }
}

impl From<ProjectAnalytics> for AnalyticsData {
    fn from(value: ProjectAnalytics) -> Self {
        Self::Project(value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct AffiliateCodeAnalytics {
    source_affiliate_code: AffiliateCodeId,
    #[serde(skip_serializing_if = "Option::is_none")]
    clicks: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    conversions: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    revenue: Option<u64>,
}

impl AffiliateCodeAnalytics {
    fn new(source_affiliate_code: AffiliateCodeId) -> Self {
        Self {
            source_affiliate_code,
            clicks: None,
            conversions: None,
            revenue: None,
        }
    }
}

impl From<AffiliateCodeAnalytics> for AnalyticsData {
    fn from(value: AffiliateCodeAnalytics) -> Self {
        Self::AffiliateCode(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RevenueSource {
    Adverts,
    ModrinthPlus,
}

// logic

async fn get(
    req: HttpRequest,
    clickhouse: web::Data<clickhouse::Client>,
    data: web::Query<GetRequest>,
    session_queue: web::Data<AuthQueue>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<web::Json<GetResponse>, ApiError> {
    let (_, user) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::ANALYTICS,
    )
    .await?;
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn response_format() {
        let test_project_1 = ProjectId(123);
        let test_project_2 = ProjectId(456);
        let test_affiliate_code = AffiliateCodeId(789);

        let src = GetResponse(vec![
            TimeSlice(vec![
                ProjectAnalytics {
                    game_versions: [
                        ("1.20.1".to_string(), 400),
                        ("1.20.2".to_string(), 300),
                    ]
                    .into_iter()
                    .collect(),
                    ..ProjectAnalytics::new(test_project_1)
                }
                .into(),
                ProjectAnalytics {
                    game_versions: [
                        ("1.20.1".to_string(), 200),
                        ("1.20.2".to_string(), 100),
                    ]
                    .into_iter()
                    .collect(),
                    ..ProjectAnalytics::new(test_project_2)
                }
                .into(),
                AffiliateCodeAnalytics {
                    clicks: Some(300),
                    conversions: Some(200),
                    ..AffiliateCodeAnalytics::new(test_affiliate_code)
                }
                .into(),
            ]),
            TimeSlice(vec![]),
        ]);
        let target = json!([
            [
                {
                    "source_project": test_project_1.to_string(),
                    "game_versions": {
                        "1.20.1": 400,
                        "1.20.2": 300,
                    }
                },
                {
                    "source_project": test_project_2.to_string(),
                    "game_versions": {
                        "1.20.1": 200,
                        "1.20.2": 100,
                    }
                },
                {
                    "source_affiliate_code": test_affiliate_code.to_string(),
                    "clicks": 300,
                    "conversions": 200
                }
            ],
            []
        ]);

        assert_eq!(serde_json::to_value(src).unwrap(), target);
    }
}
