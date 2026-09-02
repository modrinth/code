use super::super::synced_options::{
    nbt_from_bytes, nbt_to_bytes, read_nbt_file,
};
use crate::ErrorKind;
use quartz_nbt::{NbtCompound, NbtList, NbtTag};
use std::path::Path;

pub(crate) async fn read_servers(
    path: &Path,
) -> crate::Result<Vec<NbtCompound>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    servers_from_root(read_nbt_file(path).await?)
}

pub(super) fn servers_from_bytes(
    bytes: Vec<u8>,
) -> crate::Result<Vec<NbtCompound>> {
    servers_from_root(nbt_from_bytes(bytes)?)
}

fn servers_from_root(root: NbtCompound) -> crate::Result<Vec<NbtCompound>> {
    let list = root.get::<_, &NbtList>("servers").map_err(|_| {
        ErrorKind::InputError(
            "servers.dat does not contain a valid servers list".to_string(),
        )
    })?;
    list.iter()
        .map(|tag| match tag {
            NbtTag::Compound(compound) => Ok(compound.clone()),
            _ => Err(ErrorKind::InputError(
                "servers.dat contains an invalid server entry".to_string(),
            )
            .into()),
        })
        .collect()
}

pub(crate) async fn write_servers(
    path: &Path,
    servers: &[NbtCompound],
) -> crate::Result<()> {
    if let Some(parent) = path.parent() {
        crate::util::io::create_dir_all(parent).await?;
    }
    crate::util::io::write(path, servers_to_bytes(servers)?).await?;
    Ok(())
}

pub(super) fn servers_to_bytes(
    servers: &[NbtCompound],
) -> crate::Result<Vec<u8>> {
    let mut list = NbtList::new();
    for server in servers {
        list.push(server.clone());
    }
    let mut root = NbtCompound::new();
    root.insert("servers", list);
    nbt_to_bytes(&root)
}

pub(crate) fn server_data(
    name: String,
    address: String,
    accept_textures: Option<bool>,
) -> NbtCompound {
    let mut server = NbtCompound::new();
    server.insert("name", name);
    server.insert("ip", address);
    if let Some(accept_textures) = accept_textures {
        server.insert("acceptTextures", i8::from(accept_textures));
    }
    server.insert("hidden", 0_i8);
    server
}

pub(super) fn update_server_data(
    server: &mut NbtCompound,
    name: String,
    address: String,
    accept_textures: Option<bool>,
) {
    server.insert("name", name);
    server.insert("ip", address);
    match accept_textures {
        Some(value) => server.insert("acceptTextures", i8::from(value)),
        None => {
            server.inner_mut().remove("acceptTextures");
        }
    }
}

pub(super) fn server_hidden(server: &NbtCompound) -> bool {
    server.get::<_, i8>("hidden").unwrap_or(0) != 0
}

pub(super) fn server_address(server: &NbtCompound) -> String {
    server.get::<_, &str>("ip").unwrap_or_default().to_string()
}

pub(super) fn server_identity_address(server: &NbtCompound) -> String {
    server_address(server).trim().to_ascii_lowercase()
}

pub(super) fn server_identity_name(server: &NbtCompound) -> String {
    server
        .get::<_, &str>("name")
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
}
