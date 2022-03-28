//! Profile management subcommand
use crate::util::{
    confirm_async, prompt_async, select_async, table_path_display,
};
use daedalus::modded::LoaderVersion;
use eyre::{ensure, Result};
use paris::*;
use std::path::{Path, PathBuf};
use tabled::{Table, Tabled};
use theseus::{
    data::{profiles::PROFILE_JSON_PATH, Metadata, Profile, Profiles},
    launcher::ModLoader,
};
use tokio::fs;
use tokio_stream::{wrappers::ReadDirStream, StreamExt};
use uuid::Uuid;

#[derive(argh::FromArgs)]
#[argh(subcommand, name = "profile")]
/// profile management
pub struct ProfileCommand {
    #[argh(subcommand)]
    action: ProfileSubcommand,
}

#[derive(argh::FromArgs)]
#[argh(subcommand)]
pub enum ProfileSubcommand {
    Add(ProfileAdd),
    Init(ProfileInit),
    List(ProfileList),
    Remove(ProfileRemove),
    Run(ProfileRun),
}

#[derive(argh::FromArgs)]
#[argh(subcommand, name = "add")]
/// add a new profile to Theseus
pub struct ProfileAdd {
    #[argh(positional, default = "std::env::current_dir().unwrap()")]
    /// the profile to add
    profile: PathBuf,
}

impl ProfileAdd {
    pub async fn run(
        &self,
        _args: &crate::Args,
        _largs: &ProfileCommand,
    ) -> Result<()> {
        info!(
            "Adding profile at path '{}' to Theseus",
            self.profile.display()
        );

        let profile = self.profile.canonicalize()?;
        let json_path = profile.join(PROFILE_JSON_PATH);
        ensure!(
            json_path.exists(),
            "Profile json does not exist. Perhaps you wanted `profile init` or `profile fetch`?"
        );
        ensure!(
            Profiles::get().await.unwrap().0.get(&profile).is_none(),
            "Profile already managed by Theseus. If the contents of the profile are invalid or missing, the profile can be regenerated using `profile init` or `profile fetch`"
        );
        Profiles::insert_from(profile).await?;
        success!("Profile added!");

        Ok(())
    }
}

#[derive(argh::FromArgs)]
#[argh(subcommand, name = "init")]
/// create a new profile and manage it with Theseus
pub struct ProfileInit {
    #[argh(positional, default = "std::env::current_dir().unwrap()")]
    /// the path of the newly created profile
    path: PathBuf,

    #[argh(option)]
    /// the name of the profile
    name: Option<String>,

    #[argh(option)]
    /// the game version of the profile
    game_version: Option<String>,

    #[argh(option)]
    /// the icon for the profile
    icon: Option<PathBuf>,

    #[argh(option, from_str_fn(modloader_from_str))]
    /// the modloader to use
    modloader: Option<ModLoader>,

    #[argh(option)]
    /// the modloader version to use, set to "latest", "stable", or the ID of your chosen loader
    loader_version: Option<String>,
}

impl ProfileInit {
    pub async fn run(
        &self,
        _args: &crate::Args,
        _largs: &ProfileCommand,
    ) -> Result<()> {
        // TODO: validate inputs from args early
        if self.path.exists() {
            ensure!(
                self.path.is_dir(),
                "Attempted to create profile in something other than a folder!"
            );
            ensure!(
                !self.path.join(PROFILE_JSON_PATH).exists(),
                "Profile already exists! Perhaps you want `profile add` instead?"
            );
            if ReadDirStream::new(fs::read_dir(&self.path).await?)
                .next()
                .await
                .is_some()
            {
                warn!("You are trying to create a profile in a non-empty directory. If this is an instance from another launcher, please be sure to properly fill the profile.json fields!");
                if !confirm_async(
                    String::from("Do you wish to continue"),
                    false,
                )
                .await?
                {
                    eyre::bail!("Aborted!");
                }
            }
        } else {
            fs::create_dir_all(&self.path).await?;
        }
        info!(
            "Creating profile at path {}",
            &self.path.canonicalize()?.display()
        );

        let metadata = Metadata::get().await?;

        // TODO: abstract default prompting
        let name = match &self.name {
            Some(name) => name.clone(),
            None => {
                let default = self.path.file_name().unwrap().to_string_lossy();

                prompt_async(
                    String::from("Instance name"),
                    Some(default.into_owned()),
                )
                .await?
            }
        };

        let game_version = match &self.game_version {
            Some(version) => version.clone(),
            None => {
                let default = &metadata.minecraft.latest.release;

                prompt_async(
                    String::from("Game version"),
                    Some(default.clone()),
                )
                .await?
            }
        };

        let loader = match &self.modloader {
            Some(loader) => *loader,
            None => {
                let choice = select_async(
                    "Modloader".to_owned(),
                    &["vanilla", "fabric", "forge"],
                )
                .await?;

                match choice {
                    0 => ModLoader::Vanilla,
                    1 => ModLoader::Fabric,
                    2 => ModLoader::Forge,
                    _ => eyre::bail!(
                        "Invalid modloader ID: {choice}. This is a bug in the launcher!"
                    ),
                }
            }
        };

        let loader = if loader != ModLoader::Vanilla {
            let version = match &self.loader_version {
                Some(version) => String::from(version),
                None => prompt_async(
                    String::from(
                        "Modloader version (latest, stable, or a version ID)",
                    ),
                    Some(String::from("latest")),
                )
                .await?,
            };

            let filter = |it: &LoaderVersion| match version.as_str() {
                "latest" => true,
                "stable" => it.stable,
                id => it.id == String::from(id),
            };

            let loader_data = match loader {
                ModLoader::Forge => &metadata.forge,
                ModLoader::Fabric => &metadata.fabric,
                _ => eyre::bail!("Could not get manifest for loader {loader}. This is a bug in the CLI!"),
            };

            let ref loaders = loader_data.game_versions
                .iter()
                .find(|it| it.id == game_version)
                .ok_or_else(|| eyre::eyre!("Modloader {loader} unsupported for Minecraft version {game_version}"))?
                .loaders;

            let loader_version =
                loaders.iter().cloned().find(filter).ok_or_else(|| {
                    eyre::eyre!(
                        "Invalid version {version} for modloader {loader}"
                    )
                })?;

            Some((loader_version, loader))
        } else {
            None
        };

        let icon = match &self.icon {
            Some(icon) => Some(icon.clone()),
            None => Some(
                prompt_async("Icon".to_owned(), Some(String::new())).await?,
            )
            .filter(|it| !it.trim().is_empty())
            .map(PathBuf::from),
        };

        // We don't really care if the profile already is managed, as getting this far means that the user probably wanted to re-create a profile
        drop(metadata);
        let mut profile =
            Profile::new(name, game_version, self.path.clone()).await?;

        if let Some(ref icon) = icon {
            profile.with_icon(icon).await?;
        }

        if let Some((loader_version, loader)) = loader {
            profile.with_loader(loader, Some(loader_version));
        }

        Profiles::insert(profile).await?;
        Profiles::save().await?;

        success!(
            "Successfully created instance, it is now available to use with Theseus!"
        );
        Ok(())
    }
}

#[derive(argh::FromArgs)]
/// list all managed profiles
#[argh(subcommand, name = "list")]
pub struct ProfileList {}

#[derive(Tabled)]
struct ProfileRow<'a> {
    name: &'a str,
    #[field(display_with = "table_path_display")]
    path: &'a Path,
    #[header("game version")]
    game_version: &'a str,
    loader: &'a ModLoader,
    #[header("loader version")]
    loader_version: &'a str,
}

impl<'a> From<&'a Profile> for ProfileRow<'a> {
    fn from(it: &'a Profile) -> Self {
        Self {
            name: &it.metadata.name,
            path: &it.path,
            game_version: &it.metadata.game_version,
            loader: &it.metadata.loader,
            loader_version: it
                .metadata
                .loader_version
                .as_ref()
                .map_or("", |it| &it.id),
        }
    }
}

impl ProfileList {
    pub async fn run(
        &self,
        _args: &crate::Args,
        _largs: &ProfileCommand,
    ) -> Result<()> {
        let profiles = Profiles::get().await?;
        let profiles = profiles.0.values().map(ProfileRow::from);

        let table = Table::new(profiles).with(tabled::Style::psql()).with(
            tabled::Modify::new(tabled::Column(1..=1))
                .with(tabled::MaxWidth::wrapping(40)),
        );
        println!("{table}");
        Ok(())
    }
}

#[derive(argh::FromArgs)]
/// unmanage a profile
#[argh(subcommand, name = "remove")]
pub struct ProfileRemove {
    #[argh(positional, default = "std::env::current_dir().unwrap()")]
    /// the profile to get rid of
    profile: PathBuf,
}

impl ProfileRemove {
    pub async fn run(
        &self,
        _args: &crate::Args,
        _largs: &ProfileCommand,
    ) -> Result<()> {
        let profile = self.profile.canonicalize()?;
        info!("Removing profile {} from Theseus", self.profile.display());
        if confirm_async(String::from("Do you wish to continue"), true).await? {
            if Profiles::remove(&profile).await?.is_none() {
                warn!("Profile was not managed by Theseus!");
            } else {
                success!("Profile removed!");
            }
        } else {
            error!("Aborted!");
        }

        Ok(())
    }
}

#[derive(argh::FromArgs)]
/// run a profile
#[argh(subcommand, name = "run")]
pub struct ProfileRun {
    #[argh(positional, default = "std::env::current_dir().unwrap()")]
    /// the profile to run
    profile: PathBuf,

    // TODO: auth
    #[argh(option, short = 't')]
    /// the Minecraft token to use for player login. Should be replaced by auth when that is a thing.
    token: String,

    #[argh(option, short = 'n')]
    /// the uername to use for running the game
    name: String,

    #[argh(option, short = 'i')]
    /// the account id to use for running the game
    id: Uuid,
}

impl ProfileRun {
    pub async fn run(
        &self,
        _args: &crate::Args,
        _largs: &ProfileCommand,
    ) -> Result<()> {
        info!("Starting profile at path {}...", self.profile.display());
        let ref profiles = Profiles::get().await?.0;
        let path = self.profile.canonicalize()?;
        let profile = profiles
            .get(&path)
            .ok_or(
                eyre::eyre!(
                    "Profile not managed by Theseus (if it exists, try using `profile add` first!)"
                )
            )?;

        let credentials = theseus::launcher::Credentials {
            id: self.id.clone(),
            username: self.name.clone(),
            access_token: self.token.clone(),
        };

        let mut proc = profile.run(&credentials).await?;
        profile.wait_for(&mut proc).await?;

        success!("Process exited successfully!");
        Ok(())
    }
}

impl ProfileCommand {
    pub async fn dispatch(&self, args: &crate::Args) -> Result<()> {
        match &self.action {
            ProfileSubcommand::Add(ref cmd) => cmd.run(args, self).await,
            ProfileSubcommand::Init(ref cmd) => cmd.run(args, self).await,
            ProfileSubcommand::List(ref cmd) => cmd.run(args, self).await,
            ProfileSubcommand::Remove(ref cmd) => cmd.run(args, self).await,
            ProfileSubcommand::Run(ref cmd) => cmd.run(args, self).await,
        }
    }
}

fn modloader_from_str(it: &str) -> core::result::Result<ModLoader, String> {
    match it {
        "vanilla" => Ok(ModLoader::Vanilla),
        "forge" => Ok(ModLoader::Forge),
        "fabric" => Ok(ModLoader::Fabric),
        _ => Err(String::from("Invalid modloader: {it}")),
    }
}
