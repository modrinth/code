use std::{
    collections::{BTreeSet, HashMap, HashSet},
    sync::atomic::{self, AtomicUsize},
};

use anyhow::{Result, ensure};
use futures::{StreamExt, stream};
use tracing::{debug, info, info_span};
use tracing_anyhow::FutureContext;
use url::Url;

use crate::{
    import::download::DownloadRunContext,
    model,
    upstream::{
        mojang::{JavaVersion, Library},
        ornithe::{
            GAMES_URL, GameLoaderProfile, GameVersion, GameVersionName,
            LIBRARIES_BASE_URL, LOADERS_URL, LoaderVersion, LoaderVersionName,
            PROFILE_BASE_URL, VERSION_MANIFEST_BASE_URL, VersionManifest,
            is_lwjgl_upgrade,
        },
    },
    util::Sha1,
};

/// Mojang game version IDs paired with the Ornithe ID of the same client JAR.
const GAME_VERSION_ALIASES: &[(&str, &str)] = &[
    ("1.0", "1.0.0"),
    ("b1.3b", "b1.3"),
    ("a1.2.2b", "a1.2.2"),
    ("a1.2.2a", "a1.2.2-1624"),
    ("c0.30_01c", "c0.30-c-renew"),
];

/// Releases whose pre-releases Mojang named `<release> Pre-Release <n>`,
/// where Ornithe uses `<release>-pre<n>`.
const SPACED_PRE_RELEASES: &[&str] = &["1.14", "1.14.1", "1.14.2"];

/// Game versions which share the same libraries, and so the same profile.
#[derive(Debug, Clone)]
pub struct MetadataGroup {
    pub id: String,
    pub template: GameVersionName,
    pub templated: bool,
    pub games: Vec<GameVersionName>,
    /// Libraries added to every profile of the group, sorted by name.
    pub libraries: Vec<Library>,
    /// Artifacts (`groupId:artifactId`) of Mojang's libraries which Ornithe
    /// drops.
    pub removed_libraries: Vec<String>,
    /// The Java version Ornithe asks for, in place of the game's.
    pub java_version: Option<JavaVersion>,
}

/// Libraries Ornithe adds to and removes from a game version, which decide
/// its group.
#[derive(Debug, Clone)]
pub struct GameLibraries {
    pub game: GameVersionName,
    /// The libraries Ornithe's installer adds, and its LWJGL upgrades.
    pub libraries: Vec<Library>,
    pub removed_libraries: Vec<String>,
    pub java_version: Option<JavaVersion>,
}

pub async fn download(cx: &mut DownloadRunContext<'_>) -> Result<()> {
    let initial_errors = cx.errors.len();
    let (games, games_sha256) = cx
        .download_json::<Vec<GameVersion>>(GAMES_URL)
        .context(info_span!("fetching game versions"))
        .await?;
    let (loaders, loaders_sha256) = cx
        .download_json::<Vec<LoaderVersion>>(LOADERS_URL)
        .context(info_span!("fetching loader versions"))
        .await?;
    info!(
        num_game_versions = games.len(),
        num_loader_versions = loaders.len(),
        "downloaded Ornithe catalog"
    );

    let num_total = games.len();
    let num_done = AtomicUsize::new(0);
    let task = async |game: GameVersion| {
        let libraries = download_game_libraries(cx, &game.version)
            .context(
                info_span!("downloading game libraries", game = %game.version),
            )
            .await
            .inspect_err(|err| cx.errors.push(err))
            .ok()
            .flatten();

        let num_done = num_done.fetch_add(1, atomic::Ordering::SeqCst) + 1;
        if num_done.is_multiple_of(100) || num_done == num_total {
            info!("downloaded {num_done}/{num_total} game library lists");
        }
        libraries
    };
    let game_libraries = stream::iter(games)
        .map(task)
        .buffered(cx.download_concurrency)
        .filter_map(futures::future::ready)
        .collect::<Vec<_>>()
        .await;

    let groups = profile_groups(game_libraries);
    let template_pairs = groups
        .iter()
        .flat_map(|group| {
            loaders
                .iter()
                .map(|loader| (&group.template, &loader.version))
        })
        .collect::<Vec<_>>();
    let num_total = template_pairs.len();
    info!(
        num_groups = groups.len(),
        "downloading {num_total} loader profiles"
    );

    let num_done = AtomicUsize::new(0);
    let task =
        async |(game, loader): (&GameVersionName, &LoaderVersionName)| {
            cx.download_json::<GameLoaderProfile>(profile_url(game, loader))
                .await
                .inspect_err(|err| cx.errors.push(err))
                .ok();

            let num_done = num_done.fetch_add(1, atomic::Ordering::SeqCst) + 1;
            if num_done.is_multiple_of(100) || num_done == num_total {
                info!("downloaded {num_done}/{num_total} loader profiles");
            }
        };
    stream::iter(template_pairs)
        .for_each_concurrent(cx.download_concurrency, task)
        .await;

    ensure!(
        cx.errors.len() == initial_errors,
        "Ornithe item downloads failed; catalog not registered"
    );

    toasty::create!(model::OrnitheCatalog {
        download_run_id: cx.download_run_id,
        games_sha256,
        loaders_sha256,
    })
    .exec(*cx.conn().await)
    .context(info_span!("inserting catalog"))
    .await?;

    Ok(())
}

/// Downloads what Ornithe adds to a game version, or [`None`] if Ornithe has
/// no version manifest for it.
async fn download_game_libraries(
    cx: &DownloadRunContext<'_>,
    game: &GameVersionName,
) -> Result<Option<GameLibraries>> {
    let Some((manifest, _)) = cx
        .download_json_if_found::<VersionManifest>(version_manifest_url(game))
        .await?
    else {
        debug!(%game, "skipping game version without an Ornithe manifest");
        return Ok(None);
    };
    let (libraries, _) = cx
        .download_json::<Vec<Library>>(libraries_url(game))
        .await?;
    Ok(Some(GameLibraries::new(
        game.clone(),
        libraries,
        manifest,
        &[],
    )))
}

impl GameLibraries {
    /// Combines the libraries Ornithe adds to `game` with its version
    /// manifest, dropping the artifacts of `mojang_libraries` which the
    /// manifest no longer has.
    pub fn new(
        game: GameVersionName,
        mut libraries: Vec<Library>,
        manifest: VersionManifest,
        mojang_libraries: &[Library],
    ) -> Self {
        let removed_libraries =
            removed_libraries(mojang_libraries, &manifest.libraries);
        libraries
            .extend(manifest.libraries.into_iter().filter(is_lwjgl_upgrade));
        libraries.sort_by(|a, b| a.name.cmp(&b.name));
        Self {
            game,
            libraries,
            removed_libraries,
            java_version: manifest.java_version,
        }
    }
}

fn library_artifact(library: &Library) -> &str {
    library
        .name
        .rsplit_once(':')
        .map_or(library.name.as_str(), |(artifact, _)| artifact)
}

/// The artifacts of Mojang's libraries which Ornithe's version manifest drops.
pub fn removed_libraries(
    mojang_libraries: &[Library],
    ornithe_libraries: &[Library],
) -> Vec<String> {
    let ornithe_artifacts = ornithe_libraries
        .iter()
        .map(library_artifact)
        .collect::<HashSet<_>>();
    mojang_libraries
        .iter()
        .map(library_artifact)
        .filter(|artifact| !ornithe_artifacts.contains(artifact))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(str::to_owned)
        .collect()
}

fn url_with_segments(base: &str, segments: &[&str]) -> Url {
    let mut url = Url::parse(base).expect("valid Ornithe base URL");
    url.path_segments_mut()
        .expect("Ornithe URL supports path segments")
        .pop_if_empty()
        .extend(segments);
    url
}

pub fn profile_url(game: &GameVersionName, loader: &LoaderVersionName) -> Url {
    url_with_segments(
        PROFILE_BASE_URL,
        &[game.0.as_str(), loader.0.as_str(), "profile", "json"],
    )
}

pub fn libraries_url(game: &GameVersionName) -> Url {
    url_with_segments(LIBRARIES_BASE_URL, &[game.0.as_str()])
}

pub fn version_manifest_url(game: &GameVersionName) -> Url {
    url_with_segments(VERSION_MANIFEST_BASE_URL, &[&format!("{game}.json")])
}

/// The Mojang ID of the client JAR Ornithe names `game`.
pub fn mojang_game_version(game: &GameVersionName) -> String {
    if let Some((mojang, _)) = GAME_VERSION_ALIASES
        .iter()
        .find(|(_, ornithe)| *ornithe == game.0)
    {
        return (*mojang).to_owned();
    }
    if let Some((release, pre_release)) = game.0.split_once("-pre")
        && SPACED_PRE_RELEASES.contains(&release)
        && !pre_release.is_empty()
        && pre_release.bytes().all(|x| x.is_ascii_digit())
    {
        return format!("{release} Pre-Release {pre_release}");
    }
    game.0.clone()
}

/// Hashes what decides the profile of `game` into a group ID, matching the
/// IDs of the legacy daedalus client.
fn group_id(game: &GameLibraries, removed_libraries: &[String]) -> String {
    let renamed = (mojang_game_version(&game.game) != game.game.0)
        .then_some(&game.game.0);
    let java = game.java_version.as_ref().map(|java_version| {
        format!(
            "java:{}:{}",
            java_version.component, java_version.major_version
        )
    });
    let mut libraries = game
        .libraries
        .iter()
        .map(|library| library.name.clone())
        .collect::<Vec<_>>();
    libraries.sort_unstable();
    let names = libraries
        .into_iter()
        .chain(renamed.cloned())
        .chain(java)
        .chain(removed_libraries.iter().map(|x| format!("removed:{x}")))
        .collect::<Vec<_>>();
    Sha1::from_digest(names.join("\n")).to_string()[..8].to_owned()
}

/// Groups game versions by the libraries and Java version Ornithe gives them,
/// and the libraries it removes, preserving the order of `games`.
///
/// Profiles of a group are downloaded for the longest game version which
/// shares its added libraries, so that templating it never touches a shorter
/// game version inside another string. Removed libraries don't change the
/// downloaded profile, so they split groups without changing the template.
/// Game versions Mojang names differently get their own untemplated group.
pub fn profile_groups(
    games: impl IntoIterator<Item = GameLibraries>,
) -> Vec<MetadataGroup> {
    let games = games.into_iter().collect::<Vec<_>>();
    let mut templates = HashMap::<String, &GameVersionName>::new();
    for game in &games {
        templates
            .entry(group_id(game, &[]))
            .and_modify(|template| {
                if game.game.0.len() >= template.0.len() {
                    *template = &game.game;
                }
            })
            .or_insert(&game.game);
    }
    let templates = templates
        .into_iter()
        .map(|(id, template)| (id, template.clone()))
        .collect::<HashMap<_, _>>();

    let mut groups = Vec::<MetadataGroup>::new();
    for game in games {
        let template = &templates[&group_id(&game, &[])];
        let id = group_id(&game, &game.removed_libraries);
        let GameLibraries {
            game,
            libraries,
            removed_libraries,
            java_version,
        } = game;

        if let Some(group) = groups.iter_mut().find(|group| group.id == id) {
            group.games.push(game);
        } else {
            groups.push(MetadataGroup {
                id,
                template: template.clone(),
                templated: mojang_game_version(&game) == game.0,
                games: vec![game],
                libraries,
                removed_libraries,
                java_version,
            });
        }
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(id: &str) -> GameVersionName {
        GameVersionName(id.to_owned())
    }

    fn libraries(value: serde_json::Value) -> Vec<Library> {
        serde_json::from_value(value).unwrap()
    }

    fn game(id: &str, names: &[&str]) -> GameLibraries {
        GameLibraries {
            game: name(id),
            libraries: libraries(
                names
                    .iter()
                    .map(|name| serde_json::json!({ "name": name }))
                    .collect(),
            ),
            removed_libraries: Vec::new(),
            java_version: None,
        }
    }

    #[test]
    fn game_versions_with_the_same_libraries_share_a_group() {
        let groups = profile_groups([
            game("1.8.9", &["a:a:1", "b:b:1"]),
            game("13w16a-04192037", &["b:b:1", "a:a:1"]),
            game("1.14.4", &["a:a:1"]),
        ]);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].games, [name("1.8.9"), name("13w16a-04192037")]);
        assert_eq!(groups[0].template, name("13w16a-04192037"));
        assert!(groups[0].templated);
        assert_eq!(groups[1].games, [name("1.14.4")]);
        assert_ne!(groups[0].id, groups[1].id);
    }

    #[test]
    fn manifests_add_lwjgl_upgrades_and_remove_dropped_libraries() {
        let game = GameLibraries::new(
            name("1.5.2"),
            libraries(serde_json::json!([
                { "name": "org.slf4j:slf4j-api:2.0.17" },
                { "name": "com.google.code.gson:gson:2.10" }
            ])),
            VersionManifest {
                libraries: libraries(serde_json::json!([
                    { "name": "org.ow2.asm:asm:7.0" },
                    { "name": "net.minecraft:launchwrapper:1.5" },
                    {
                        "name": "org.lwjgl.lwjgl:lwjgl:2.9.4+legacyfabric.15",
                        "url": "https://maven.legacyfabric.net/"
                    }
                ])),
                java_version: None,
            },
            &libraries(serde_json::json!([
                { "name": "net.minecraft:launchwrapper:1.5" },
                { "name": "org.ow2.asm:asm-all:4.1" },
                { "name": "org.lwjgl.lwjgl:lwjgl:2.9.0" },
                { "name": "org.lwjgl.lwjgl:lwjgl:2.9.1-nightly-20130708-debug3" }
            ])),
        );

        assert_eq!(
            game.libraries
                .iter()
                .map(|library| library.name.as_str())
                .collect::<Vec<_>>(),
            [
                "com.google.code.gson:gson:2.10",
                "org.lwjgl.lwjgl:lwjgl:2.9.4+legacyfabric.15",
                "org.slf4j:slf4j-api:2.0.17"
            ]
        );
        assert_eq!(game.removed_libraries, ["org.ow2.asm:asm-all"]);
    }

    #[test]
    fn group_ids_match_the_legacy_daedalus_client() {
        let mut game = game("1.5.2", &["b:b:1", "a:a:1"]);
        game.java_version = Some(JavaVersion {
            component: "jre-legacy".to_owned(),
            major_version: 8,
        });
        game.removed_libraries = vec!["org.ow2.asm:asm-all".to_owned()];

        assert_eq!(profile_groups([game])[0].id, "4237e563");
    }

    #[test]
    fn removed_libraries_split_groups_without_changing_templates() {
        let mut dropping_asm_all = game("1.5.2", &["a:a:1"]);
        dropping_asm_all.removed_libraries =
            vec!["org.ow2.asm:asm-all".to_owned()];
        let groups = profile_groups([
            dropping_asm_all,
            game("13w16a-04192037", &["a:a:1"]),
        ]);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].removed_libraries, ["org.ow2.asm:asm-all"]);
        assert!(groups[1].removed_libraries.is_empty());
        assert_ne!(groups[0].id, groups[1].id);
        assert_eq!(groups[0].template, name("13w16a-04192037"));
        assert_eq!(groups[1].template, name("13w16a-04192037"));
    }

    #[test]
    fn java_versions_split_groups() {
        let mut modern = game("1.12.2", &["a:a:1"]);
        modern.java_version = Some(JavaVersion {
            component: "java-runtime-epsilon".to_owned(),
            major_version: 25,
        });
        let groups = profile_groups([game("1.8.9", &["a:a:1"]), modern]);

        assert_eq!(groups.len(), 2);
        assert!(groups[0].java_version.is_none());
        assert_eq!(
            groups[1].java_version.as_ref().map(|x| x.major_version),
            Some(25)
        );
        assert_ne!(groups[0].id, groups[1].id);
    }

    #[test]
    fn renamed_game_versions_get_an_untemplated_group() {
        let groups = profile_groups([
            game("1.0.1", &["a:a:1"]),
            game("1.0.0", &["a:a:1"]),
            game("1.14.2-pre4", &["a:a:1"]),
        ]);

        assert_eq!(groups.len(), 3);
        assert!(groups[0].templated);
        assert_eq!(groups[1].games, [name("1.0.0")]);
        assert!(!groups[1].templated);
        assert_eq!(groups[2].template, name("1.14.2-pre4"));
        assert!(!groups[2].templated);
    }

    #[test]
    fn maps_ornithe_ids_to_mojang_ids() {
        for (ornithe, mojang) in [
            ("1.0.0", "1.0"),
            ("c0.30-c-renew", "c0.30_01c"),
            ("1.14-pre1", "1.14 Pre-Release 1"),
            ("1.14.2-pre4", "1.14.2 Pre-Release 4"),
            ("1.14.2-pre4-270720", "1.14.2-pre4-270720"),
            ("1.14.3-pre1", "1.14.3-pre1"),
            ("1.8.9", "1.8.9"),
        ] {
            assert_eq!(mojang_game_version(&name(ornithe)), mojang);
        }
    }

    #[test]
    fn urls_match_ornithe_meta() {
        assert_eq!(
            profile_url(&name("1.8.9"), &LoaderVersionName("0.19.5".into()))
                .as_str(),
            "https://meta.ornithemc.net/v3/versions/gen2/fabric-loader/1.8.9/0.19.5/profile/json"
        );
        assert_eq!(
            libraries_url(&name("b1.7.3")).as_str(),
            "https://meta.ornithemc.net/v3/versions/gen2/libraries/b1.7.3"
        );
        assert_eq!(
            version_manifest_url(&name("1.8.9")).as_str(),
            "https://ornithemc.net/mc-versions/gen2/version/manifest/1.8.9.json"
        );
    }

    #[test]
    fn only_legacy_fabric_libraries_are_lwjgl_upgrades() {
        let libraries: Vec<Library> = serde_json::from_value(serde_json::json!([
            {
                "name": "org.lwjgl.lwjgl:lwjgl-platform:2.9.4+legacyfabric.15",
                "downloads": { "classifiers": { "natives-osx": {
                    "sha1": "0000000000000000000000000000000000000000",
                    "size": 1,
                    "url": "https://maven.legacyfabric.net/natives-osx.jar"
                } } },
                "natives": { "osx-arm64": "natives-osx" }
            },
            {
                "name": "net.java.jinput:jinput:2.0.5",
                "downloads": { "artifact": {
                    "sha1": "0000000000000000000000000000000000000000",
                    "size": 2,
                    "url": "https://libraries.minecraft.net/jinput.jar"
                } }
            }
        ]))
        .unwrap();

        assert!(is_lwjgl_upgrade(&libraries[0]));
        assert!(!is_lwjgl_upgrade(&libraries[1]));
    }
}
