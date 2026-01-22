/// Macro to create a tracing span for a SQLx operation with OpenTelemetry-compatible fields.
///
/// - `$name`: The operation name (e.g., "sqlx.execute").
/// - `$statement`: The SQL statement being executed.
/// - `$attributes`: Connection or pool attributes for peer and db context.
///
/// This macro is used internally by the crate to instrument all major SQLx operations.
#[macro_export]
macro_rules! instrument {
    ($name:expr, $statement:expr, $attributes:expr) => {
        tracing::info_span!(
            $name,
            // Database name (if available)
            "db.name" = $attributes.database,
            // Operation type (filled by SQLx or left empty)
            "db.operation" = ::tracing::field::Empty,
            // The SQL query text
            "db.query.text" = $statement,
            // Number of affected rows (to be filled after execution)
            "db.response.affected_rows" = ::tracing::field::Empty,
            // Number of returned rows (to be filled after execution)
            "db.response.returned_rows" = ::tracing::field::Empty,
            // Status code of the response (to be filled after execution)
            "db.response.status_code" = ::tracing::field::Empty,
            // Table name (optional, left empty)
            "db.sql.table" = ::tracing::field::Empty,
            // Database system (e.g., "postgresql", "sqlite")
            "db.system.name" = DB::SYSTEM,
            // Error type, message, and stacktrace (to be filled on error)
            "error.type" = ::tracing::field::Empty,
            "error.message" = ::tracing::field::Empty,
            "error.stacktrace" = ::tracing::field::Empty,
            // Peer (server) host and port
            "net.peer.name" = $attributes.host,
            "net.peer.port" = $attributes.port,
            // OpenTelemetry semantic fields
            "otel.kind" = "client",
            "otel.status_code" = ::tracing::field::Empty,
            "otel.status_description" = ::tracing::field::Empty,
            // Peer service name (if set)
            "peer.service" = $attributes.name,
        )
    };
}

/// Records that a single row was returned in the current tracing span.
/// Used for fetch_one operations.
pub fn record_one<T>(_value: &T) {
    let span = tracing::Span::current();
    span.record("db.response.returned_rows", 1);
}

/// Records whether an optional row was returned in the current tracing span.
/// Used for fetch_optional operations.
pub fn record_optional<T>(value: &Option<T>) {
    let span = tracing::Span::current();
    span.record(
        "db.response.returned_rows",
        if value.is_some() { 1 } else { 0 },
    );
}

/// Records error details in the current tracing span for a SQLx error.
/// Sets OpenTelemetry status and error fields for observability backends.
pub fn record_error(err: &sqlx::Error) {
    let span = tracing::Span::current();
    // Mark the span as an error for OpenTelemetry
    span.record("otel.status_code", "error");
    span.record("otel.status_description", err.to_string());
    // Classify error type as client or server
    match err {
        sqlx::Error::ColumnIndexOutOfBounds { .. }
        | sqlx::Error::ColumnDecode { .. }
        | sqlx::Error::ColumnNotFound(_)
        | sqlx::Error::Decode { .. }
        | sqlx::Error::Encode { .. }
        | sqlx::Error::RowNotFound
        | sqlx::Error::TypeNotFound { .. } => {
            span.record("error.type", "client");
        }
        _ => {
            span.record("error.type", "server");
        }
    }
    // Attach error message and stacktrace for debugging
    span.record("error.message", err.to_string());
    span.record("error.stacktrace", format!("{err:?}"));
}
