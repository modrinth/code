use std::sync::Arc;

use sentry::{Hub, SentryFutureExt, TransactionContext, protocol::SpanStatus};
use tracing::Span;
use tracing_anyhow::FutureContext;

/// Runs an operation in its own Sentry transaction within the current trace.
pub trait TaskContext<Out, T, E>
where
    Out: anyhow::Context<T, E>,
    Self: Sized + IntoFuture<Output = Out>,
{
    /// Preserves tracing and error context while giving this operation its own
    /// Sentry span budget. Concurrent operations use independent hubs.
    ///
    /// The span factory must create a new span inside the task's hub; passing a
    /// pre-created span would re-enter the hub captured by the Sentry layer.
    ///
    /// ```ignore
    /// download_profile(cx, profile)
    ///     .transaction_context(|| tracing::info_span!("downloading profile"))
    ///     .await?;
    /// ```
    async fn transaction_context(
        self,
        make_span: impl FnOnce() -> Span,
    ) -> anyhow::Result<T> {
        let hub = Arc::new(Hub::new_from_top(Hub::current()));
        let parent = hub.configure_scope(|scope| scope.get_span());
        let headers: Vec<_> = parent
            .as_ref()
            .into_iter()
            .flat_map(|parent| parent.iter_headers())
            .collect();
        let context = TransactionContext::continue_from_headers(
            "task",
            "task",
            headers.iter().map(|(key, value)| (*key, value.as_str())),
        );
        let transaction = hub.start_transaction(context);
        transaction.set_origin("manual.task");
        hub.configure_scope(|scope| {
            scope.set_span(Some(transaction.clone().into()));
        });

        let future = Hub::run(hub.clone(), || {
            let span = make_span();
            if let Some(metadata) = span.metadata() {
                transaction.set_name(metadata.name());
                transaction.set_op(&format!(
                    "{}::{}",
                    metadata.target(),
                    metadata.name()
                ));
            }
            self.context(span)
        });
        let result = future.bind_hub(hub.clone()).await;
        if transaction.get_status().is_none() {
            transaction.set_status(if result.is_ok() {
                SpanStatus::Ok
            } else {
                SpanStatus::InternalError
            });
        }
        Hub::run(hub, || transaction.finish());
        result
    }
}

impl<Out, T, E, Fut> TaskContext<Out, T, E> for Fut
where
    Out: anyhow::Context<T, E>,
    Fut: Sized + IntoFuture<Output = Out>,
{
}
