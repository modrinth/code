use sqlx::{Database, Execute, SqlStr};

/// Retains query metadata before SQLx consumes the query to obtain its SQL text.
pub(crate) struct TracedQuery<DB: Database> {
    pub(crate) sql: SqlStr,
    statement: Option<DB::Statement>,
    arguments: Result<Option<DB::Arguments>, sqlx::error::BoxDynError>,
    persistent: bool,
}

impl<DB: Database> TracedQuery<DB> {
    pub(crate) fn new<'q>(mut query: impl Execute<'q, DB>) -> Self {
        let statement = query.statement().cloned();
        let persistent = query.persistent();
        let arguments = query.take_arguments();
        Self {
            sql: query.sql(),
            statement,
            arguments,
            persistent,
        }
    }
}

impl<'q, DB: Database> Execute<'q, DB> for TracedQuery<DB> {
    fn sql(self) -> SqlStr {
        self.sql
    }

    fn statement(&self) -> Option<&DB::Statement> {
        self.statement.as_ref()
    }

    fn take_arguments(
        &mut self,
    ) -> Result<Option<DB::Arguments>, sqlx::error::BoxDynError> {
        std::mem::replace(&mut self.arguments, Ok(None))
    }

    fn persistent(&self) -> bool {
        self.persistent
    }
}
