use crate::database::models::categories::ReportType;
use crate::database::models::report_item::Report;
use crate::database::models::{
    generate_report_id, DatabaseError, ProjectId, UserId, VersionId,
};
use crate::models::users::DELETED_USER;
use censor::Censor;
use time::OffsetDateTime;

pub async fn censor_check(
    text: &str,
    project: Option<ProjectId>,
    version: Option<VersionId>,
    user: Option<UserId>,
    report_text: String,
    mut transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), DatabaseError> {
    let censor = Censor::Standard + Censor::Sex;
    if censor.check(text) {
        let report_type =
            ReportType::get_id("inappropriate", &mut *transaction)
                .await?
                .expect("No database entry for 'inappropriate' report type");
        Report {
            id: generate_report_id(&mut transaction).await?,
            report_type_id: report_type,
            project_id: project,
            version_id: version,
            user_id: user,
            body: report_text,
            reporter: UserId::from(DELETED_USER),
            created: OffsetDateTime::now_utc(),
        }
        .insert(&mut transaction)
        .await?;
    }
    Ok(())
}
