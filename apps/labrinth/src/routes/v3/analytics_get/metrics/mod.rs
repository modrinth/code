mod affiliate_code_clicks;
mod affiliate_code_conversions;
mod affiliate_code_revenue;
mod project_downloads;
mod project_playtime;
mod project_revenue;
mod project_views;

use serde::{Deserialize, Serialize};

use crate::models::ids::{AffiliateCodeId, ProjectId};

pub(crate) use affiliate_code_clicks::fetch as fetch_affiliate_code_clicks;
pub use affiliate_code_clicks::{
    AffiliateCodeClicks, AffiliateCodeClicksField, AffiliateCodeClicksFilters,
};
pub(crate) use affiliate_code_conversions::fetch as fetch_affiliate_code_conversions;
pub use affiliate_code_conversions::{
    AffiliateCodeConversions, AffiliateCodeConversionsField,
    AffiliateCodeConversionsFilters,
};
pub(crate) use affiliate_code_revenue::fetch as fetch_affiliate_code_revenue;
pub use affiliate_code_revenue::{
    AffiliateCodeRevenue, AffiliateCodeRevenueField,
    AffiliateCodeRevenueFilters,
};
pub use project_downloads::{
    DownloadSource, ProjectDownloads, ProjectDownloadsField,
    ProjectDownloadsFilters,
};
pub(crate) use project_downloads::{
    fetch as fetch_project_downloads, normalize_download_source,
};
pub(crate) use project_playtime::fetch as fetch_project_playtime;
pub use project_playtime::{
    ProjectPlaytime, ProjectPlaytimeField, ProjectPlaytimeFilters,
};
pub(crate) use project_revenue::fetch as fetch_project_revenue;
pub use project_revenue::{
    ProjectRevenue, ProjectRevenueField, ProjectRevenueFilters,
};
pub(crate) use project_views::fetch as fetch_project_views;
pub use project_views::{ProjectViews, ProjectViewsField, ProjectViewsFilters};

/// What metrics the caller would like to receive from this analytics get
/// request.
#[derive(Debug, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ReturnMetrics {
    /// How many times a project page has been viewed.
    pub project_views: Option<Metrics<ProjectViewsField, ProjectViewsFilters>>,
    /// How many times a project has been downloaded.
    pub project_downloads:
        Option<Metrics<ProjectDownloadsField, ProjectDownloadsFilters>>,
    /// How long users have been playing a project.
    pub project_playtime:
        Option<Metrics<ProjectPlaytimeField, ProjectPlaytimeFilters>>,
    /// How much payout revenue a project has generated.
    pub project_revenue:
        Option<Metrics<ProjectRevenueField, ProjectRevenueFilters>>,
    /// How many times an affiliate code has been clicked.
    pub affiliate_code_clicks:
        Option<Metrics<AffiliateCodeClicksField, AffiliateCodeClicksFilters>>,
    /// How many times a product has been purchased with an affiliate code.
    pub affiliate_code_conversions: Option<
        Metrics<AffiliateCodeConversionsField, AffiliateCodeConversionsFilters>,
    >,
    /// How much payout revenue an affiliate code has generated.
    pub affiliate_code_revenue:
        Option<Metrics<AffiliateCodeRevenueField, AffiliateCodeRevenueFilters>>,
}

/// See [`ReturnMetrics`].
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Metrics<BucketBy, FilterBy> {
    /// When collecting metrics, what fields do we want to group the results by?
    ///
    /// For example, if we have two views entries:
    /// - `{ "project_id": "abcdefgh", "domain": "youtube.com", "count": 5 }`
    /// - `{ "project_id": "abcdefgh", "domain": "discord.com", "count": 3 }`
    ///
    /// If we bucket by `domain`, then we will get two results:
    /// - `{ "project_id": "abcdefgh", "domain": "youtube.com", "count": 5 }`
    /// - `{ "project_id": "abcdefgh", "domain": "discord.com", "count": 3 }`
    ///
    /// If we do not bucket by `domain`, we will only get one, which is an
    /// aggregate of the two rows:
    /// - `{ "project_id": "abcdefgh", "count": 8 }`
    #[serde(default = "Vec::default")]
    pub bucket_by: Vec<BucketBy>,
    /// Filters to apply before aggregating this metric.
    ///
    /// Values within one field are ORed together. Different fields are ANDed
    /// together. An empty list means that field is not filtered.
    #[serde(default)]
    pub filter_by: FilterBy,
}

/// Metrics collected in a single time slice.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(untagged)] // the presence of `source_project`, `source_affiliate_code` determines the kind
pub enum AnalyticsData {
    /// Project metrics.
    Project(ProjectAnalytics),
    AffiliateCode(AffiliateCodeAnalytics),
}

/// Project metrics.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectAnalytics {
    /// What project these metrics are for.
    pub source_project: ProjectId,
    /// Metrics collected.
    #[serde(flatten)]
    pub metrics: ProjectMetrics,
}

/// Project metrics of a specific kind.
///
/// If a field is not included in [`Metrics::bucket_by`], it will be [`None`].
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case", tag = "metric_kind")]
pub enum ProjectMetrics {
    /// [`ReturnMetrics::project_views`].
    Views(ProjectViews),
    /// [`ReturnMetrics::project_downloads`].
    Downloads(ProjectDownloads),
    /// [`ReturnMetrics::project_playtime`].
    Playtime(ProjectPlaytime),
    /// [`ReturnMetrics::project_revenue`].
    Revenue(ProjectRevenue),
}

/// Affiliate code metrics.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AffiliateCodeAnalytics {
    /// What affiliate code these metrics are for.
    pub source_affiliate_code: AffiliateCodeId,
    /// Metrics collected.
    #[serde(flatten)]
    pub metrics: AffiliateCodeMetrics,
}

/// Affiliate code metrics of a specific kind.
///
/// If a field is not included in [`Metrics::bucket_by`], it will be [`None`].
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case", tag = "metric_kind")]
pub enum AffiliateCodeMetrics {
    Clicks(AffiliateCodeClicks),
    Conversions(AffiliateCodeConversions),
    Revenue(AffiliateCodeRevenue),
}
