use std::{path::{PathBuf, Path}, fs::read_to_string};

use serde::{Serialize, Deserialize, de};
use tokio::{fs, io::BufReader};

use crate::{profile_create, pack::{install_from::{PackFormat, CreatePackDescription}, install_mrpack}, event::LoadingBarId, util::fetch, State};

use super::install_from::{PackDependency, self};

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
    pub managed_pack_type : Option<PrismManagedPackType>,
    pub managed_pack_version_id : Option<String>,
    pub managed_pack_version_name : Option<String>,

    #[serde(rename = "iconKey")]
    pub icon_key : Option<String>,
    #[serde(rename = "name")]
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
    Modrinth,
    #[serde(rename = "")]
    Unmanaged
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
    prism_base_path: PathBuf, // path to base prism folder 
    instance_folder: String, // instance folder in prism_base_path
    profile_path : PathBuf, // path to profile
    existing_loading_bar: Option<LoadingBarId>,

) -> crate::Result<()> {
    let state = crate::State::get().await?;
    let prism_instance_path = prism_base_path.join("instances").join(instance_folder.clone());

    
    // NOTE: Seems like mmc-packs are for meta-folder dependencies
    // uid is the folder name, then version?
    println!("import_prism({:?})", prism_instance_path);
    let mmc_pack = fs::read_to_string(&prism_instance_path.join("mmc-pack.json")).await?;
    let mmc_pack: PrismPack = serde_json::from_str::<PrismPack>(&mmc_pack)?;
    println!("mmc-pack.json: {:#?}", mmc_pack);

    let instance_cfg = fs::read_to_string(&prism_instance_path.join("instance.cfg")).await?;
    let instance_cfg: PrismInstance = serde_ini::from_str::<PrismInstance>(&instance_cfg)?;

    println!("instance.cfg: {:#?}", &instance_cfg);

    // Re-cache icon
    let icon = if let Some(icon_key) = instance_cfg.general.icon_key  {
        let icon_path = prism_base_path.join("icons").join(icon_key);
        dbg!(&icon_path);
        let bytes = tokio::fs::read(&icon_path).await;
        if let Ok(bytes ) = bytes {
            let bytes = bytes::Bytes::from(bytes);
            let cache_dir = &state.directories.caches_dir();
            dbg!(&cache_dir);
            let semaphore = &state.io_semaphore;
            Some(fetch::write_cached_icon(&icon_path.to_string_lossy(), cache_dir, bytes, semaphore).await?)    
        } else {
            // could not find icon (for instance, prism default icon, etc)
            None
        }
    } else { 
        None
    };

    // Create description from instance.cfg
    let description = CreatePackDescription {
        icon,
        override_title:  instance_cfg.general.name,
        project_id:  instance_cfg.general.managed_pack_id,
        version_id:  instance_cfg.general.managed_pack_version_id,
        existing_loading_bar: existing_loading_bar.clone(),
        profile: profile_path.clone(), 
    };

    // Managed pack
    if instance_cfg.general.managed_pack  {
        match instance_cfg.general.managed_pack_type {
            // MODRINTH MANAGED PACK
            Some(PrismManagedPackType::Modrinth) => {
                // Get overrides.txt in /mrpack
                println!("importing modrinth managed pack");
                let file = fs::File::open(&prism_instance_path.join("mrpack").join("overrides.txt")).await?;
                // add each line as a different String in a Vec
                use tokio::io::AsyncBufReadExt;
                let mut lines = BufReader::new(file).lines();
                println!("lines: {:?}", lines);
                let mut override_paths : Vec<PathBuf> = Vec::new();
                while let Some(line) = lines.next_line().await? {
                    override_paths.push(line.into());
                }

                // Get mrpack.json in /mrpack
                let mrpack = serde_json::from_str::<PackFormat>(&read_to_string(&prism_instance_path.join("mrpack").join("modrinth.index.json"))?)?;
                println!("mrpack");


                install_mrpack::install_importable_mrpack(profile_path, description, mrpack, prism_instance_path.join(".minecraft"),  override_paths).await?;

            }
            // For flame, etc
            Some(x) => todo!("import non-modrinth managed pack: {:?}", x),
            _ => return Err(crate::ErrorKind::InputError({
                format!("Managed pack type not specified in instance.cfg")
            }).into())
        }
    } else {
        let backup_name = "Imported Modpack".to_string();
        import_prism_unmanaged(profile_path, prism_base_path, instance_folder, backup_name,  description, mmc_pack, existing_loading_bar).await?;

    }
    Ok(())
}

async fn import_prism_unmanaged(profile_path: PathBuf, prism_base_path: PathBuf, instance_folder: String, backup_name : String, description: CreatePackDescription, mmc_pack: PrismPack, existing_loading_bar : Option<LoadingBarId>)-> crate::Result<()> {
    let state = crate::State::get().await?;
    

    println!("loading dependencies {:?}", mmc_pack.components);

    // Pack dependencies stored in mmc-pack.json, we convert to .mrpack pack dependencies
    let dependencies = mmc_pack.components.iter().filter_map(|component| {
        if component.uid.starts_with("net.fabricmc.fabric-loader") {
            return Some((PackDependency::FabricLoader, component.cached_version.clone()));
        }
        println!("Examining: {}", component.uid);
        if component.uid.starts_with("net.minecraftforge") {
            println!("Found forge: {}", component.uid);
            return Some((PackDependency::Forge, component.cached_version.clone()));
        }
        if component.uid.starts_with("org.quiltmc.quilt-loader") {
            return Some((PackDependency::QuiltLoader, component.cached_version.clone()));
        }
        if component.uid.starts_with("net.minecraft") {
            return Some((PackDependency::Minecraft, component.cached_version.clone()));
        }

        None
    }).collect();

    println!("dependencies: {:?}", dependencies);
    println!("description: {:?}", description);
    // Sets profile information to be that loaded from mmc-pack.json and instance.cfg
    install_from::set_profile_information(profile_path.clone(), &description, &backup_name, &dependencies).await?;

    println!("mmc_pack.components: {:?}", mmc_pack.components);
    // Moves .minecraft folder over (ie: overrides such as resourcepacks, mods, etc)
    import_dotminecraft(profile_path.clone(), prism_base_path.join("instances").join(instance_folder).join(".minecraft")).await?;

    if let Some(profile_val) =
    crate::api::profile::get(&profile_path, None).await?
    {
        crate::launcher::install_minecraft(
            &profile_val,
            existing_loading_bar,
        )
        .await?;

        State::sync().await?;
    }
Ok(())


}


async fn import_dotminecraft(profile_path : PathBuf, dotminecraft: PathBuf) -> crate::Result<()> {

    println!("here import_dotminecraft {:?} {:?}", profile_path, dotminecraft);
    // std fs copy every file in dotminecraft to profile_path
    for entry in std::fs::read_dir(dotminecraft)? {
        let entry = entry?;
        let path = entry.path();
        copy_dir_to(&path, &profile_path.join(path.file_name().ok_or_else(|| crate::ErrorKind::InputError(format!("Invalid file: {}", &path.display())))?)).await?;
    }
    
    Ok(())
}

// recursively fs::copy every file in src to dest
// uses async recursion
#[theseus_macros::debug_pin]
#[async_recursion::async_recursion]
#[tracing::instrument]
async fn copy_dir_to(src: &Path, dst: &Path) -> crate::Result<()> {
    if !src.is_dir() {
        fs::copy(src, dst).await?;
        return Ok(());
    }

    // Create the destination directory
    fs::create_dir_all(&dst).await?;

    // Iterate over the directory
    let mut dir = fs::read_dir(src).await?;
    while let Some(child) = dir.next_entry().await? {
        let src_child = child.path();
        let dst_child = dst.join(src_child.file_name().ok_or_else(|| crate::ErrorKind::InputError(format!("Invalid file: {}", &src_child.display())))?);

        if child.metadata().await?.is_dir() {
            // Recurse into sub-directory
            copy_dir_to(&src_child, &dst_child).await?;
        } else {
            // Copy file
            fs::copy(&src_child, &dst_child).await?;
        }
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use crate::pack::install_from::CreatePackProfile;

    use super::*;

    const PRISM_FOLDER : &'static str = r"/home/thesuzerain/.local/share/PrismLauncher";

    // #[tokio::test]
    // async fn test_import_prism_cobblemon() {

    //     setup_tests().await.unwrap();

    //     // Import cobblemon mrpack
    //     let profile_path = profile_create::profile_create_from_creator(CreatePackProfile::default()).await.unwrap();
    //     let result = import_prism(PRISM_FOLDER.into(), r"Cobblemon [Fabric]".to_string(),  profile_path.clone(), None).await.unwrap();
    //     crate::profile::remove(&profile_path).await.unwrap();

    //     println!("Cobblemon result: {:?}", result);

    //     panic!("hello");
    // }

    #[tokio::test]
    async fn test_import_prism_tempor() {

        setup_tests().await.unwrap();

        // Tempor test
        let profile_path = profile_create::profile_create_from_creator(CreatePackProfile::default()).await.unwrap();
        let result = import_prism(PRISM_FOLDER.into(), r"tempor".to_string(),  profile_path.clone(), None).await.unwrap();
        // crate::profile::remove(&profile_path).await.unwrap();

        println!("tempor result: {:?}", result);

        panic!("hello");
    }

    // #[tokio::test]
    // async fn test_import_prism_pixelmon() {

    //     setup_tests().await.unwrap();

    //     // Tempor test
    //     let profile_path = profile_create::profile_create_from_creator(CreatePackProfile::default()).await.unwrap();
    //     let result = import_prism(PRISM_FOLDER.into(), r"The Pixelmon Modpack".to_string(),  profile_path.clone(), None).await.unwrap();
    //     crate::profile::remove(&profile_path).await.unwrap();

    //     println!("The Pixelmon Modpack result: {:?}", result);

    //     panic!("hello");
    // }



    async fn setup_tests() -> crate::Result<()> {

        // Initialize state
        let state = crate::State::get().await?;

        // Set java globals to auto detected ones
        {
            let jres = crate::jre::get_all_jre().await?;
            let java_8 =
                crate::jre::find_filtered_jres("1.8", jres.clone(), false).await?;
            let java_17 =
                crate::jre::find_filtered_jres("1.17", jres.clone(), false).await?;
            let java_18plus =
                crate::jre::find_filtered_jres("1.18", jres.clone(), true).await?;
            let java_globals =
                crate::jre::autodetect_java_globals(java_8, java_17, java_18plus)
                    .await?;
            state.settings.write().await.java_globals = java_globals;
    }

        Ok(())
    }
}