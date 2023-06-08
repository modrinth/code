use serde::{Deserialize, Serialize};

use crate::event::emit::{emit_command, emit_warning};

/// Handles external functions (such as through URL deep linkage)
/// Link starts with: modrinth://(link)
pub async fn handle_url(link: &str) -> crate::Result<()> {
    if let Some(sublink) = link.strip_prefix("modrinth://") {
        handle(sublink).await
    } else {
        Err(crate::ErrorKind::InputError(format!(
            "Invalid command, missing prefix: {link}"
        ))
        .into())
    }
}

/// Handles external functions (such as through URL deep linkage)
/// Link is extracted value (link) in somewhat URL format, such as
/// subdomain1/subdomain2
/// (Does not include modrinth://)
pub async fn handle(sublink: &str) -> crate::Result<()> {
    match sublink.split_once('/') {
        // /mod/{id}   -    Installs a mod of mod id
        Some(("mod", id)) => {
            emit_command(DeepLinkCommandType::InstallMod, id.to_string())
                .await?
        }
        // /version/{id}   -    Installs a specific version of id
        Some(("version", id)) => {
            emit_command(DeepLinkCommandType::InstallModpack, id.to_string())
                .await?
        }
        // /modpack/{id}   -    Installs a modpack of modpack id
        Some(("modpack", id)) => {
            emit_command(DeepLinkCommandType::InstallModpack, id.to_string())
                .await?
        }
        _ => {
            emit_warning(&format!(
                "Invalid command, unrecognized path: {sublink}"
            ))
            .await?;
            return Err(crate::ErrorKind::InputError(format!(
                "Invalid command, unrecognized path: {sublink}"
            ))
            .into());
        }
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Clone)]
pub enum DeepLinkCommandType {
    InstallMod,
    InstallVersion,
    InstallModpack,
}
