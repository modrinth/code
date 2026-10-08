use anyhow::Result;
use tracing::{info_span, warn};
use tracing_anyhow::FutureContext;

use crate::{AppState, export};

pub async fn export(app: &mut AppState) -> Result<()> {
	export::mojang::export(app)
		.context(info_span!("exporting Mojang artifacts"))
		.await
		.inspect_err(|err| warn!("error: {err:?}"))
		.ok();

	export::fabric::export(app)
		.context(info_span!("exporting Fabric artifacts"))
		.await
		.inspect_err(|err| warn!("error: {err:?}"))
		.ok();

	Ok(())
}
