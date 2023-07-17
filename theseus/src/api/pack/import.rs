use std::{path::PathBuf, fs::read_to_string};

use serde::{Serialize, Deserialize, de};
use tokio::{fs, io::BufReader};

use crate::{profile_create, pack::{install_from::{PackFormat, CreatePackDescription}, install}};

// instance.cfg
// https://github.com/PrismLauncher/PrismLauncher/blob/develop/launcher/minecraft/MinecraftInstance.cpp
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct PrismInstance {
    pub general : PrismInstanceGeneral,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct PrismInstanceGeneral {
    pub java_path : Option<String>,
    pub jvm_args : Option<String>,

    #[serde(deserialize_with = "deserialize_bool")]
    pub managed_pack : bool,
    pub managed_pack_id : Option<String>,
    pub managed_pack_type : PrismManagedPackType,
    pub managed_pack_version_id : Option<String>,
    pub managed_pack_version_name : Option<String>,

    pub icon_key : Option<String>,
    pub name : Option<String>,  
}

// serde_ini reads 'true' and 'false' as strings, so we need to convert them to booleans
fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(de::Error::custom("expected 'true' or 'false'")),
    }
}



#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum PrismManagedPackType {
    Modrinth
}


// mmc-pack.json
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PrismPack {
    components : Vec<PrismComponent>,
    format_version : u32,
}

// https://github.com/PrismLauncher/PrismLauncher/blob/develop/launcher/minecraft/Component.h
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PrismComponent {
    pub uid: String,

    #[serde(default)]
    pub version : Option<String>,
    #[serde(default)]
    pub dependency_only : bool,

    #[serde(default)]
    pub important : bool,
    #[serde(default)]
    pub disabled : bool,

    #[serde(default)]
    pub cached_name : String,
    #[serde(default)]
    pub cached_version : String,

    #[serde(default)]
    pub cached_requires : Vec<PrismComponentRequirement>,
    #[serde(default)]
    pub cached_conflicts : Vec<PrismComponentRequirement>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PrismComponentRequirement {
    pub uid: String,
    pub equals_version : Option<String>,
    pub suggests : Option<String>,
}

#[tracing::instrument]
#[theseus_macros::debug_pin]
pub async fn import_prism(
    base_path: PathBuf, // path to prism instance

) -> crate::Result<()> {

    // NOTE: Seems like mmc-packs are for meta-folder dependencies
    // uid is the folder name, then version?
    println!("import_prism({:?})", base_path);
    let mmc_pack = fs::read_to_string(&base_path.join("mmc-pack.json")).await?;
    let mmc_pack: PrismPack = serde_json::from_str::<PrismPack>(&mmc_pack)?;
    println!("mmc-pack.json: {:#?}", mmc_pack);

    let instance_cfg = fs::read_to_string(&base_path.join("instance.cfg")).await?;
    let instance_cfg: PrismInstance = serde_ini::from_str::<PrismInstance>(&instance_cfg)?;

    // Managed pack
    if instance_cfg.general.managed_pack  {
        match instance_cfg.general.managed_pack_type {
            // MODRINTH MANAGED PACK
            PrismManagedPackType::Modrinth => {
                // Get overrides.txt in /mrpack
                let file = fs::File::open(&base_path.join("mrpack").join("overrides.txt")).await?;
                // add each line as a different String in a Vec
                use tokio::io::AsyncBufReadExt;
                let mut lines = BufReader::new(file).lines();
                let mut override_paths : Vec<PathBuf> = Vec::new();
                while let Some(line) = lines.next_line().await? {
                    override_paths.push(base_path.join(line));
                }

                // Get mrpack.json in /mrpack
                // let file = fs::File::open(&base_path.join("mrpack").join("mrpack.json")).await?;
                let mrpack = serde_json::from_str::<PackFormat>(&read_to_string(&base_path.join("mrpack").join("mrpack.json"))?)?;
                
                let description = CreatePackDescription {
                    // pub override_title: Option<String>,
                    // pub project_id: Option<String>,
                    // pub version_id: Option<String>,
                    // pub existing_loading_bar: Option<LoadingBarId>,
                    // pub profile: PathBuf,
                    icon: None, // TODO: cant just get icon directly, need to copy it to cache
                    override_title:  instance_cfg.general.name,
                    project_id:  instance_cfg.general.managed_pack_id,
                    version_id:  instance_cfg.general.managed_pack_version_id,
                    existing_loading_bar: None,
                    profile: base_path.clone(), 
                };

                install::import_pack(base_path, description, mrpack, override_paths).await?;

            }
            // For flame, etc
            _ => todo!("import non-modrinth managed pack: {:?}", instance_cfg.general.managed_pack_type)
        }
    } else {
        let name = instance_cfg.general.name.unwrap_or("Imported Modpack".to_string());
        todo!("import non-managed pack: {}", name);   
    }
    
        panic!("hello");
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    const PRISM_FOLDER : &'static str = r"/Users/wyattverchere/Library/Application Support/PrismLauncher/instances";

    #[tokio::test]
    async fn test_import_prism() {
        let path = PathBuf::from(PRISM_FOLDER).join(r"Cobblemon [Fabric]");
        let result = import_prism(path).await.unwrap();

    }
}