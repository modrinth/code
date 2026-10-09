# Installing a content set

If any step fails set the `content_set.status` to `NotInstalled`.

## Common steps

### C0 - Start installation

1. Load the [content set](../../types/CONTENT_SET.md) and its installation using `content_set.installation_id`.
2. Read `content_set.game_version`, `content_set.loader` and `content_set.loader_version`.
3. Check that the installation is not running and its folder exists.
4. Set the content set's status to `Installing` and update `modified`.

### C1 - Get the version details

1. Get the list of Minecraft versions:

	`GET https://launcher-meta.modrinth.com/minecraft/v0/manifest.json`

2. Find `content_set.game_version` in the list. If a saved list does not contain it, fetch the list again. **Stop if `content_set.game_version` is still missing.**
3. Get the version details using the matching entry's `url`:

	`GET {url}`

4. Save them to `versions/{content_set.game_version}/{content_set.game_version}.json`.

These details tell us which Java version and files Minecraft needs.

### C2 - Set up Java

1. Read `javaVersion.majorVersion` from the version details. **If it is missing, stop.**
2. Check whether the required Java version is already set up. If it is, skip to step 6.
3. Find a Java download for the computer's OS and processor:

	`GET https://api.azul.com/metadata/v1/zulu/packages?arch={arch}&java_version={major}&os={os}&archive_type=zip&javafx_bundled=false&java_package_type=jre&page_size=1`

4. Download the first result:

	`GET {packages[0].download_url}`

5. Extract it into the shared Java folder and save the path to Java.
6. Run Java to check that it works and has the expected version.

### C3 - Download Minecraft

Paths are relative to the shared Minecraft folder. Check each downloaded file against its supplied SHA-1 and stop if it does not match.

1. Client only: Get the asset index, which lists Minecraft's sounds, textures and other assets:

	- Request: `GET {assetIndex.url}`.
	- SHA-1: `assetIndex.sha1`.
	- Save to: `assets/indexes/{assetIndex.id}.json`.

**2a. Client: Get the game JAR.**

- Request: `GET {downloads.client.url}`.
- SHA-1: `downloads.client.sha1`.
- Save to: `versions/{content_set.game_version}/{content_set.game_version}.jar`.

**2b. Server: Get the server JAR.**

- Request: `GET {downloads.server.url}`.
- SHA-1: `downloads.server.sha1`.
- Save to: `versions/{content_set.game_version}/{content_set.game_version}-server.jar`.
- Stop if the version has no server download.

3. Client only: Get each file listed in the asset index:

	- Request: `GET https://resources.download.minecraft.net/{prefix}/{hash}`.
	- `hash` is the file's SHA-1. `prefix` is its first two characters.
	- Save to: `assets/objects/{prefix}/{hash}`.
	- Older versions may also need a copy in the legacy assets folder, using the name from the index.

4. Client only: Get the libraries listed in the version details:

	- Only include libraries allowed for the computer's OS and Java architecture.
	- Skip libraries marked as non-downloadable.
	- For OS-specific native libraries, download the matching file from `downloads.classifiers` and extract it into the version's natives folder.
	- For other libraries, download `downloads.artifact.url` and save under `libraries/` using the path from the library name.
	- Check the SHA-1 supplied with each file.
	- If the download URL is missing, use the library's repository URL and path. The default repository is `https://libraries.minecraft.net/`.

5. Get the logging config, if `logging.client.file` is present:

	- Request: `GET {logging.client.file.url}`.
	- SHA-1: `logging.client.file.sha1`.
	- Save to: The logging config folder, named `logging.client.file.id`.

Steps 2–5 can run at the same time.

### C4 - Finish

1. Read `version.json` inside the game JAR. Set the content set's `protocol_version` from it, if present.
2. Set the content set's status to `Available` if its content items are also installed. Otherwise, leave it `NotInstalled` until content installation finishes. Update `modified`.

## Vanilla

For a content set with `content_set.loader: vanilla` and no `content_set.loader_version`:

1. **C0** - Start installation.
2. **C1** - Get the details for Minecraft `content_set.game_version`.
3. **C2** - Set up Java.
4. **C3** - Download Minecraft.
5. **C4** - Finish.

## Fabric

For a content set with `content_set.loader: fabric` and a selected `content_set.loader_version`:

1. **C0** - Start installation.
2. **C1** - Get the details for Minecraft `content_set.game_version`.
3. Get the Fabric profile for the selected versions. URL-encode both version values.

**3a. Client:**

- Request: `GET https://meta.fabricmc.net/v2/versions/loader/{content_set.game_version}/{content_set.loader_version}/profile/json`.

**3b. Server:**

- Request: `GET https://meta.fabricmc.net/v2/versions/loader/{content_set.game_version}/{content_set.loader_version}/server/json`.

4. Save the profile separately from the vanilla version details. Stop if the selected versions are unsupported.
5. **C2** - Set up Java.
6. **C3** - Download Minecraft for the client or server.
7. Download the libraries listed in the Fabric profile, including Fabric Loader:

	- Request: `GET {library.url}{library_path}`.
	- Build `library_path` from `library.name`. For `group:artifact:version`, use `group/as/path/artifact/version/artifact-version.jar`.
	- Save to: `libraries/{library_path}`.
	- Reuse files already downloaded. Check supplied hashes when available.

8. Save the launch details for this content set:

	- Use the Fabric profile's `mainClass` and additional arguments.
	- Client: Keep Minecraft's launch arguments and include both Minecraft and Fabric libraries.
	- Server: Include the server JAR and Fabric libraries. Point Fabric at the server JAR using `fabric.gameJarPath`.

9. **C4** - Finish.

## NeoForge

For a content set with `content_set.loader: neoforge` and a selected `content_set.loader_version`:

1. **C0** - Start installation.
2. **C1** - Get the details for Minecraft `content_set.game_version`.
3. Download the NeoForge installer:

	- Request: `GET https://maven.neoforged.net/releases/net/neoforged/neoforge/{content_set.loader_version}/neoforge-{content_set.loader_version}-installer.jar`.
	- SHA-1: `GET {installer_url}.sha1`.
	- For Minecraft `1.20.1`, use `net/neoforged/forge/{content_set.loader_version}/forge-{content_set.loader_version}-installer.jar` instead.
	- Check the installer against its SHA-1.

4. Open `install_profile.json` inside the installer:

	- Check that `minecraft` matches `content_set.game_version`. **Stop if it does not.**
	- Read the version details from the file named by `json`.
	- Save these separately from the vanilla version details.

5. **C2** - Set up Java.
6. **C3** - Download Minecraft for the client or server.
7. Get the files listed in the install profile and NeoForge version details:

	- Extract files included in the installer.
	- Download other libraries using `GET {library.downloads.artifact.url}`.
	- Save them under `libraries/` at `library.downloads.artifact.path` and check their supplied SHA-1.
	- Files produced by the installer steps do not need downloading.

8. If the installer needs Minecraft mappings, download them from the version details:

**8a. Client:**

- Request: `GET {downloads.client_mappings.url}`.
- SHA-1: `downloads.client_mappings.sha1`.

**8b. Server:**

- Request: `GET {downloads.server_mappings.url}`.
- SHA-1: `downloads.server_mappings.sha1`.

9. Run the installer steps listed in `processors`, in order:

	- Use the `client` or `server` values from the profile's `data`.
	- Skip steps whose `sides` do not include this installation's side.
	- Run each step with Java, using its `jar`, `classpath` and `args`.
	- Replace references in the arguments with the file paths and values from the profile, including the game JAR and mappings.
	- Check any listed output hashes. **Stop if a processor fails.**

10. Save the launch details for this content set:

	- Client: Use NeoForge's `mainClass`, libraries and arguments alongside the vanilla version details.
	- Server: Keep the server argument files supplied by the installer and use the one for the installation's OS. Keep `user_jvm_args.txt` for Java settings.

11. **C4** - Finish.

## Forge

For a content set with `content_set.loader: forge` and a selected `content_set.loader_version`:

1. **C0** - Start installation.
2. **C1** - Get the details for Minecraft `content_set.game_version`.
3. Find the full Forge version in the version list:

	- Request: `GET https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json`.
	- Find the entry for `content_set.game_version` matching `content_set.loader_version`.
	- Keep the full version string as `forge_version`, including any suffix. **Stop if there is no match.**

4. Download the Forge installer:

	- Request: `GET https://maven.minecraftforge.net/net/minecraftforge/forge/{forge_version}/forge-{forge_version}-installer.jar`.
	- SHA-1: `GET {installer_url}.sha1`.
	- Check the installer against its SHA-1.

5. Open `install_profile.json` inside the installer:

	- Read the version details from the file named by `json`. Older installers store these directly in `versionInfo`.
	- Check that the installer's Minecraft version matches `content_set.game_version`. **Stop if it does not.**
	- Save the Forge version details separately from the vanilla version details.
	- Versions without an installer are not covered by this flow.

6. **C2** - Set up Java.
7. **C3** - Download Minecraft for the client or server.
8. Get the files listed in the install profile and Forge version details:

	- Extract files included in the installer. Older installers identify the Forge file through `install.filePath` and its library location through `install.path`.
	- Download other libraries using `GET {library.downloads.artifact.url}`.
	- If no download URL is provided, use the library's repository URL and path, as in C3.
	- Save under `libraries/` and check supplied hashes.
	- Files produced by the installer steps do not need downloading.

9. If the installer needs Minecraft mappings:

**9a. Client:**

- Request: `GET {downloads.client_mappings.url}`.
- SHA-1: `downloads.client_mappings.sha1`.

**9b. Server:**

- Request: `GET {downloads.server_mappings.url}`.
- SHA-1: `downloads.server_mappings.sha1`.

10. Run the profile's `processors` in order, if present:

	- Use the `client` or `server` values from `data`.
	- Run steps for this side, including steps with no `sides` restriction.
	- Use each step's `jar`, `classpath` and `args`, replacing references with the matching files and values.
	- Check any listed output hashes. **Stop if a processor fails.**

11. Save the launch details for this content set:

	- Client: Use Forge's `mainClass`, libraries and arguments alongside the vanilla version details.
	- Server: Use the server argument file for the installation's OS. Older versions use the installed Forge server JAR instead.

12. **C4** - Finish.

## Quilt

For a content set with `content_set.loader: quilt` and a selected `content_set.loader_version`:

1. **C0** - Start installation.
2. **C1** - Get the details for Minecraft `content_set.game_version`.
3. Get the Quilt profile for the selected versions. URL-encode both version values.

**3a. Client:**

- Request: `GET https://meta.quiltmc.org/v3/versions/loader/{content_set.game_version}/{content_set.loader_version}/profile/json`.

**3b. Server:**

- Request: `GET https://meta.quiltmc.org/v3/versions/loader/{content_set.game_version}/{content_set.loader_version}/server/json`.

4. Save the profile separately from the vanilla version details. Stop if the selected versions are unsupported.
5. **C2** - Set up Java.
6. **C3** - Download Minecraft for the client or server.
7. Download the libraries listed in the Quilt profile, including Quilt Loader:

	- Request: `GET {library.url}{library_path}`.
	- Build `library_path` from `library.name`. For `group:artifact:version`, use `group/as/path/artifact/version/artifact-version.jar`.
	- Save to: `libraries/{library_path}`.
	- Reuse files already downloaded. Check supplied hashes when available.

8. Save the launch details for this content set:

	- Use the Quilt profile's `mainClass` and additional arguments.
	- Client: Keep Minecraft's launch arguments and include both Minecraft and Quilt libraries.
	- Server: Include the server JAR and Quilt libraries. Point Quilt at the server JAR using `loader.gameJarPath`.

9. **C4** - Finish.

## Paper

For a server content set with `content_set.loader: paper`. `content_set.loader_version` is the selected Paper build number.

1. **C0** - Start installation. **Stop if this is a client installation.**
2. **C1** - Get the details for Minecraft `content_set.game_version`.
3. Get the Paper builds for this Minecraft version:

	- Request: `GET https://fill.papermc.io/v3/projects/paper/versions/{content_set.game_version}/builds`.
	- Include a `User-Agent` with the software name, version and contact URL in Paper download requests.
	- Find the build whose `id` matches `content_set.loader_version`. **Stop if there is no match.**

4. Download the selected build:

	- Request: `GET {build.downloads["server:default"].url}`.
	- Check the file against the SHA-256 supplied with the download details.
	- Save to: `server.jar` in the installation folder.

5. **C2** - Set up Java.
6. Prepare the server files from the installation folder:

	`java -Dpaperclip.patchonly=true -jar server.jar`

	- Paperclip downloads the original Minecraft server JAR if needed, using the URL included in `server.jar`.
	- It checks the download, applies Paper's patches and extracts the server libraries.
	- Keep the generated files in the installation folder. **Stop if preparation fails.**
	- This prepares the files without starting the server.

7. Save `java -jar server.jar nogui` as the server launch command.
8. **C4** - Finish, reading the protocol version from the prepared Minecraft server JAR.

C3 is skipped because Paperclip prepares the Minecraft files it needs.

## Purpur

For a server content set with `content_set.loader: purpur`. `content_set.loader_version` is the selected Purpur build number.

1. **C0** - Start installation. **Stop if this is a client installation.**
2. **C1** - Get the details for Minecraft `content_set.game_version`.
3. Get the selected build details:

	- Request: `GET https://api.purpurmc.org/v2/purpur/{content_set.game_version}/{content_set.loader_version}`.
	- Check that `version` and `build` match the content set and `result` is `SUCCESS`. **Stop if they do not match or the build failed.**

4. Download the selected build:

	- Request: `GET https://api.purpurmc.org/v2/purpur/{content_set.game_version}/{content_set.loader_version}/download`.
	- Check the file against `md5` from the build details.
	- Save to: `server.jar` in the installation folder.

5. **C2** - Set up Java.
6. Prepare the server files from the installation folder:

	`java -Dpaperclip.patchonly=true -jar server.jar`

	- Paperclip downloads the original Minecraft server JAR if needed, using the URL included in `server.jar`.
	- It applies the patches and extracts the server libraries.
	- Keep the generated files in the installation folder. **Stop if preparation fails.**
	- This prepares the files without starting the server.

7. Save `java -jar server.jar nogui` as the server launch command.
8. **C4** - Finish, reading the protocol version from the prepared Minecraft server JAR.

C3 is skipped because Paperclip prepares the Minecraft files it needs.
