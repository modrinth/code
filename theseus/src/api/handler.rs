use serde::{Deserialize, Serialize};
use std::{env, path::{Path, PathBuf}, ffi::OsString};
use crate::event::{emit::{emit_command, emit_warning}, CommandPayload};
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
            emit_command(CommandPayload::InstallMod { id:  id.to_string() })
                .await?
        }
        // /version/{id}   -    Installs a specific version of id
        Some(("version", id)) => {
            emit_command( CommandPayload::InstallVersion { id:  id.to_string() })
                .await?
        }
        // /modpack/{id}   -    Installs a modpack of modpack id
        Some(("modpack", id)) => {
            emit_command(  CommandPayload::InstallModpack { id:  id.to_string() })
                .await?
        },
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

pub async fn parse_mrpack_command(path_string : &str) -> crate::Result<CommandPayload> {
    emit_warning(&path_string).await?;
    let path = PathBuf::from(path_string);
    emit_warning(&path.to_string_lossy()).await?;
    let path = path.canonicalize()?;
    emit_warning(&path.to_string_lossy()).await?;
    Ok(CommandPayload::RunMRPack { path })
}

