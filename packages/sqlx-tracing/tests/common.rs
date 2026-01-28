use std::time::Duration;

use opentelemetry::trace::{FutureExt, TraceContextExt, Tracer};

pub async fn should_trace<'c, DB, E>(
    name: &'static str,
    system: &'static str,
    observability: &opentelemetry_testing::ObservabilityContainer,
    provider: &opentelemetry_testing::OpenTelemetryProvider,
    executor: E,
) where
    DB: sqlx::Database,
    E: sqlx::Executor<'c, Database = DB>,
    for<'q> DB::Arguments<'q>: 'q + sqlx::IntoArguments<'q, DB>,
    (i32,): Send + Unpin + for<'r> sqlx::FromRow<'r, DB::Row>,
{
    let scope = format!("should_{name}_{system}");
    let tracer = opentelemetry::global::tracer(scope.clone());
    let span = tracer.span_builder(name).start(&tracer);
    let ctx = opentelemetry::Context::new().with_span(span);

    let result: Option<i32> = sqlx::query_scalar("select 1")
        .fetch_optional(executor)
        .with_context(ctx)
        .await
        .unwrap();

    assert_eq!(result, Some(1));

    provider.flush();

    tokio::time::sleep(Duration::from_secs(1)).await;

    let traces = observability.json_traces();
    let scope_span = traces.find_scope_span(&scope).unwrap();
    let entry = scope_span.first_span().unwrap();
    assert_eq!(entry.name, name);
    let next = traces
        .find_child(&entry.span_id, "sqlx.fetch_optional")
        .unwrap();
    assert_eq!(next.string_attribute("db.system.name").unwrap(), system);
    assert_eq!(next.string_attribute("db.query.text").unwrap(), "select 1");
    assert_eq!(
        next.int_attribute("db.response.returned_rows").unwrap(),
        "1"
    );
}
