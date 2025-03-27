use std::path::PathBuf;

use crate::{
    event::{
        emit::{emit_command, emit_warning},
        CommandPayload,
    },
    util::io,
};

pub async fn handle_url(sublink: &str) -> crate::Result<CommandPayload> {
    let parts: Vec<&str> = sublink.split('/').filter(|s| !s.is_empty()).collect();

    if parts.is_empty() {
        emit_warning(&format!("Invalid command, unrecognized path: {sublink}")).await?;
        return Err(crate::ErrorKind::InputError(format!(
            "Invalid command, unrecognized path: {sublink}"
        ))
            .into());
    }

    match parts.as_slice() {
        // /mod/{id} - Installs a mod of mod id
        ["mod", id] => Ok(CommandPayload::InstallMod { id: id.to_string() }),
        // /version/{id} - Installs a specific version of id
        ["version", id] => Ok(CommandPayload::InstallVersion { id: id.to_string() }),
        // /modpack/{id} - Installs a modpack of modpack id
        ["modpack", id] => Ok(CommandPayload::InstallModpack { id: id.to_string() }),
        // /instance/{uuid}/{action_type} - Performs a specific action on a specific instance.
        // {action_type} is of "play" | "edit"
        ["instance", id, action_type] => Ok(CommandPayload::ManageInstance { id: id.to_string(), action_type: action_type.to_string() }),
        _ => {
            emit_warning(&format!(
                "Invalid command, unrecognized path: {sublink}"
            ))
                .await?;
            Err(crate::ErrorKind::InputError(format!(
                "Invalid command, unrecognized path: {sublink}"
            ))
                .into())
        }
    }
}

pub async fn parse_command(
    command_string: &str,
) -> crate::Result<CommandPayload> {
    tracing::debug!("Parsing command: {}", &command_string);

    // modrinth://some-command
    // This occurs when following a web redirect link
    if let Some(sublink) = command_string.strip_prefix("modrinth://") {
        Ok(handle_url(sublink).await?)
    } else {
        // We assume anything else is a filepath to an .mrpack file
        let path = PathBuf::from(command_string);
        let path = io::canonicalize(path)?;
        if let Some(ext) = path.extension() {
            if ext == "mrpack" {
                return Ok(CommandPayload::RunMRPack { path });
            }
        }
        emit_warning(&format!(
            "Invalid command, unrecognized filetype: {}",
            path.display()
        ))
            .await?;
        Err(crate::ErrorKind::InputError(format!(
            "Invalid command, unrecognized filetype: {}",
            path.display()
        ))
            .into())
    }
}

pub async fn parse_and_emit_command(command_string: &str) -> crate::Result<()> {
    let command = parse_command(command_string).await?;
    emit_command(command).await?;
    Ok(())
}