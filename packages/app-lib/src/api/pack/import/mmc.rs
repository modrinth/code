use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize, de};

use crate::{
    State,
    pack::{
        import::{self, copy_dotminecraft},
        install_from::{self, CreatePackDescription, PackDependency},
    },
    util::io,
};

// instance.cfg
// https://github.com/PrismLauncher/PrismLauncher/blob/develop/launcher/minecraft/MinecraftInstance.cpp
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
#[serde(untagged)]
enum MMCInstanceEnum {
    General(MMCInstanceGeneral),
    Instance(MMCInstance),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct MMCInstanceGeneral {
    pub general: MMCInstance,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct MMCInstance {
    pub java_path: Option<String>,
    pub jvm_args: Option<String>,

    #[serde(default)]
    #[serde(deserialize_with = "deserialize_optional_bool")]
    pub managed_pack: Option<bool>,

    #[serde(rename = "ManagedPackID")]
    pub managed_pack_id: Option<String>,
    pub managed_pack_type: Option<MMCManagedPackType>,
    #[serde(rename = "ManagedPackVersionID")]
    pub managed_pack_version_id: Option<String>,
    pub managed_pack_version_name: Option<String>,

    #[serde(rename = "iconKey")]
    pub icon_key: Option<String>,
    #[serde(rename = "name")]
    pub name: Option<String>,
}

// serde_ini reads 'true' and 'false' as strings, so we need to convert them to booleans
fn deserialize_optional_bool<'de, D>(
    deserializer: D,
) -> Result<Option<bool>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?;
    match s {
        Some(string) => match string.as_str() {
            "true" => Ok(Some(true)),
            "false" => Ok(Some(false)),
            _ => Err(de::Error::custom("expected 'true' or 'false'")),
        },
        None => Ok(None),
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum MMCManagedPackType {
    Modrinth,
    Flame,
    ATLauncher,
    #[serde(other)]
    Unknown,
}

// mmc-pack.json
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MMCPack {
    components: Vec<MMCComponent>,
    format_version: u32,
}

// https://github.com/PrismLauncher/PrismLauncher/blob/develop/launcher/minecraft/Component.h
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MMCComponent {
    pub uid: String,

    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub dependency_only: bool,

    #[serde(default)]
    pub important: bool,
    #[serde(default)]
    pub disabled: bool,

    pub cached_name: Option<String>,
    pub cached_version: Option<String>,

    #[serde(default)]
    pub cached_requires: Vec<MMCComponentRequirement>,
    #[serde(default)]
    pub cached_conflicts: Vec<MMCComponentRequirement>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MMCComponentRequirement {
    pub uid: String,
    pub equals_version: Option<String>,
    pub suggests: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
#[serde(untagged)]
enum MMCLauncherEnum {
    General(MMCLauncherGeneral),
    Instance(MMCLauncher),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct MMCLauncherGeneral {
    pub general: MMCLauncher,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct MMCLauncher {
    instance_dir: String,
}

// Checks if if its a folder, and the folder contains instance.cfg and mmc-pack.json, and they both parse
#[tracing::instrument]
pub async fn is_valid_mmc(instance_folder: PathBuf) -> bool {
    let instance_cfg = instance_folder.join("instance.cfg");
    let mmc_pack = instance_folder.join("mmc-pack.json");

    let mmc_pack = match io::read_to_string(&mmc_pack).await {
        Ok(mmc_pack) => mmc_pack,
        Err(_) => return false,
    };

    load_instance_cfg(&instance_cfg).await.is_ok()
        && serde_json::from_str::<MMCPack>(&mmc_pack).is_ok()
}

#[tracing::instrument]
pub async fn get_instances_subpath(config: PathBuf) -> Option<String> {
    let launcher = io::read_to_string(&config).await.ok()?;
    let launcher: MMCLauncherEnum = serde_ini::from_str(&launcher).ok()?;
    match launcher {
        MMCLauncherEnum::General(p) => Some(p.general.instance_dir),
        MMCLauncherEnum::Instance(p) => Some(p.instance_dir),
    }
}

// Loading the INI (instance.cfg) file
async fn load_instance_cfg(file_path: &Path) -> crate::Result<MMCInstance> {
    let instance_cfg: String = io::read_to_string(file_path).await?;
    let instance_cfg_enum: MMCInstanceEnum =
        serde_ini::from_str::<MMCInstanceEnum>(&instance_cfg)?;
    match instance_cfg_enum {
        MMCInstanceEnum::General(instance_cfg) => Ok(instance_cfg.general),
        MMCInstanceEnum::Instance(instance_cfg) => Ok(instance_cfg),
    }
}

// #[tracing::instrument]
pub async fn import_mmc(
    mmc_base_path: PathBuf,  // path to base mmc folder
    instance_folder: String, // instance folder in mmc_base_path
    profile_path: &str,      // path to profile
) -> crate::Result<()> {
    let mmc_instance_path =
        mmc_base_path.join("instances").join(instance_folder);

    let mmc_pack =
        io::read_to_string(&mmc_instance_path.join("mmc-pack.json")).await?;
    let mmc_pack: MMCPack = serde_json::from_str::<MMCPack>(&mmc_pack)?;

    let instance_cfg =
        load_instance_cfg(&mmc_instance_path.join("instance.cfg")).await?;

    // Re-cache icon
    let icon = if let Some(icon_key) = instance_cfg.icon_key {
        let icon_path = mmc_base_path.join("icons").join(icon_key);
        import::recache_icon(icon_path).await?
    } else {
        None
    };

    // Create description from instance.cfg
    let mut description = CreatePackDescription {
        icon,
        override_title: instance_cfg.name,
        project_id: None,
        version_id: None,
        existing_loading_bar: None,
        profile_path: profile_path.to_string(),
    };

    let mut minecraft_folder = mmc_instance_path.join("minecraft");
    if !minecraft_folder.is_dir() {
        minecraft_folder = mmc_instance_path.join(".minecraft");
        if !minecraft_folder.is_dir() {
            return Err(crate::ErrorKind::InputError(
                "Instance is missing Minecraft directory".to_string(),
            )
            .into());
        }
    }

    // Managed pack
    if instance_cfg.managed_pack.unwrap_or(false) {
        match instance_cfg.managed_pack_type {
            Some(MMCManagedPackType::Modrinth) => {
                description.project_id = instance_cfg.managed_pack_id;
                description.version_id = instance_cfg.managed_pack_version_id;

                // Modrinth Managed Pack
                // Kept separate as we may in the future want to add special handling for modrinth managed packs
                import_mmc_unmanaged(profile_path, minecraft_folder, "Imported Modrinth Modpack".to_string(), description, mmc_pack).await?;
            }
            Some(MMCManagedPackType::Flame) | Some(MMCManagedPackType::ATLauncher) => {
                // For flame/atlauncher managed packs
                // Treat as unmanaged, but with 'minecraft' folder instead of '.minecraft'
                import_mmc_unmanaged(profile_path, minecraft_folder, "Imported Modpack".to_string(), description, mmc_pack).await?;
            },
            Some(_) => {
                // For managed packs that aren't modrinth, flame, atlauncher
                // Treat as unmanaged
                import_mmc_unmanaged(profile_path, minecraft_folder, "ImportedModpack".to_string(), description, mmc_pack).await?;
            },
            _ => return Err(crate::ErrorKind::InputError("Instance is managed, but managed pack type not specified in instance.cfg".to_string()).into())
        }
    } else {
        // Direclty import unmanaged pack
        import_mmc_unmanaged(
            profile_path,
            minecraft_folder,
            "Imported Modpack".to_string(),
            description,
            mmc_pack,
        )
        .await?;
    }
    Ok(())
}

async fn import_mmc_unmanaged(
    profile_path: &str,
    minecraft_folder: PathBuf,
    backup_name: String,
    description: CreatePackDescription,
    mmc_pack: MMCPack,
) -> crate::Result<()> {
    // Pack dependencies stored in mmc-pack.json, we convert to .mrpack pack dependencies
    let dependencies = mmc_pack
        .components
        .iter()
        .filter_map(|component| {
            if component.uid.starts_with("net.fabricmc.fabric-loader") {
                return Some((
                    PackDependency::FabricLoader,
                    component.version.clone().unwrap_or_default(),
                ));
            }
            if component.uid.starts_with("net.minecraftforge") {
                return Some((
                    PackDependency::Forge,
                    component.version.clone().unwrap_or_default(),
                ));
            }
            if component.uid.starts_with("org.quiltmc.quilt-loader") {
                return Some((
                    PackDependency::QuiltLoader,
                    component.version.clone().unwrap_or_default(),
                ));
            }
            if component.uid.starts_with("net.minecraft") {
                return Some((
                    PackDependency::Minecraft,
                    component.version.clone().unwrap_or_default(),
                ));
            }

            None
        })
        .collect();

    // Sets profile information to be that loaded from mmc-pack.json and instance.cfg
    install_from::set_profile_information(
        profile_path.to_string(),
        &description,
        &backup_name,
        &dependencies,
        false,
    )
    .await?;

    // Moves .minecraft folder over (ie: overrides such as resourcepacks, mods, etc)
    let state = State::get().await?;
    let loading_bar = copy_dotminecraft(
        profile_path,
        minecraft_folder,
        &state.io_semaphore,
        None,
    )
    .await?;

    if let Some(profile_val) = crate::api::profile::get(profile_path).await? {
        crate::launcher::install_minecraft(
            &profile_val,
            Some(loading_bar),
            false,
        )
        .await?;
    }
    Ok(())
}
