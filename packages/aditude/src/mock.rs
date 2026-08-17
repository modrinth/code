//! See [`AditudeMock`].

use crate::{v1, v2};

/// Mock data returned by [`crate::Aditude`].
#[derive(Debug, Clone, Default)]
pub struct AditudeMock {
    /// Response for [`crate::Aditude::get_metrics_v1`].
    pub metrics_v1_response: Option<Vec<v1::MetricsResponse>>,
    /// Response for [`crate::Aditude::get_metrics_v2`].
    pub metrics_v2_response: Option<v2::Metrics>,
}
