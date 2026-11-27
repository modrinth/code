---
name: options-syncing-update
description: Update the game options sync catalog in packages/app-lib for a new Minecraft version, for example `/options-syncing-update <version>`. Use when a Minecraft release, release candidate, or pre-release needs catalog support, or to re-check a catalog row that came from a pre-release.
---

# Update the Options Sync Catalog

The argument is the target Minecraft version, for example `<major>.<minor>`. The catalog is in `packages/app-lib/src/api/instance/synced_options/game_options/catalog/`.

Before you start, read these files:

- `README.md`, `mod.rs`, and `version_changes.rs` in the catalog directory
- `packages/app-lib/AGENTS.md`

## 1. Select the Builds

1. Find the previous catalog version in the last row of the README summary table.
2. Find the newest build of the target version in `https://piston-meta.mojang.com/mc/game/version_manifest_v2.json`. Use the release if it exists. If not, use the newest release candidate. If no release candidate exists, use the newest pre-release.
3. Check whether the previous README row has a suffix that names a build, for example `<previous> (<previous>-pre-N)`.
4. If the row has a suffix and the previous version is now released, compare that build with the release. Use steps 2.1 to 2.4 for the comparison. If `options.txt` is the same in both builds, remove the suffix.

## 2. Compare the Source on mcsrc.dev

Use the built-in browser. Before mcsrc.dev decompiles a version, it asks you to agree to the Minecraft EULA. Ask the user for permission before you select the checkbox.

The site has two views:

- The class view, for example `https://mcsrc.dev/2/<version>/net/minecraft/client/Options`.
- The diff view, for example `https://mcsrc.dev/2/diff/<new>/<old>/net/minecraft/client/Options`. The panel on the left lists all changed classes. The `added`, `modified`, and `deleted` labels are relative to the second version in the URL.

Monaco renders only the visible lines. Use this script to read the full text from the editor model. For a diff view, the script returns two models. The first model is the `<new>` build.

```js
const models = () => {
	const out = []
	let done = false
	document.querySelectorAll('.monaco-editor').forEach((el) => {
		for (let n = el, i = 0; i < 20 && n && !done; i++, n = n.parentElement) {
			const fk = Object.keys(n).find((k) => k.startsWith('__reactFiber'))
			for (let f = fk && n[fk], j = 0; f && j < 60 && !done; j++, f = f.return) {
				const check = (v) => {
					const m = v?.getModel?.()
					if (!m) return
					m.original
						? out.push(m.original.getValue(), m.modified.getValue())
						: out.push(m.getValue())
					done = true
				}
				Object.values(f.memoizedProps ?? {}).forEach(
					(v) => !done && (check(v), check(v?.current)),
				)
				for (let st = f.memoizedState, k = 0; st && k < 30 && !done; st = st.next, k++) {
					check(st.memoizedState)
					check(st.memoizedState?.current)
				}
			}
		}
	})
	return out
}
```

Do these steps with the two `Options` sources:

1. Get the option keys from the `access.process("…")` calls. Get the key bindings from the `KeyMapping(…"key.…")` calls. Compare the two lists.
2. Compare the full `Options` sources line by line. Look for changes to defaults, ranges, codecs, and enum value sets.
3. Look for `.filter(...)` calls on `debugKeys` and `keyMappings`. If only debug builds or IDE builds include a key, a normal `options.txt` file does not contain that key.
4. Look for new `Options*Fix` entries in `net/minecraft/util/datafix/DataFixers`.
   - An `OptionsRenameFieldFix` entry renames a key.
   - A split fix or a set-to-custom fix changes stored values.
5. Look for these classes in the changed-class list of the diff view, and compare each changed class. These enums supply stored option values.
   - `PreferredGraphicsApi`
   - `PresenceSharing`
   - `SoundSource`
   - `GraphicsPreset`
   - `TextureFilteringMethod`
   - `MusicToastDisplayState`
   - `CloudStatus`
   - `ParticleStatus`
   - `NarratorStatus`
   - `ChatVisiblity`
   - `InactivityFpsLimit`
   - `PrioritizeChunkUpdates`
   - `AttackIndicatorStatus`
   - `MusicFrequency`
   - `PlayerModelPart`
   - `HumanoidArm`
   - `KeyMapping`

For context, read the minecraft.net articles for each snapshot and pre-release, and the wiki page `Options.txt (Java Edition)`. If these sources disagree with the decompiled source, use the decompiled source.

## 3. Update the Catalog

1. Change `"<previous>"` to `"<target>"` only where it is an `until` bound.
   - Do not change a `since` bound. A `since` bound is a `since:` field, or a version literal that another version literal follows.
   - Use a script for this change. Then use grep to examine the remaining occurrences of `"<previous>"`.
2. Add each new key with `ranged_setting(...)` or `key_setting!(...)`.
   - Use the target version for both bounds.
   - Use the category of the related settings.
   - Set `default_on` to `false` if the value depends on the computer, for example a graphics backend or a platform toggle.
3. Model a renamed key as a `VersionedKey` constant in `version_changes.rs`. `SWAP_OFFHAND_KEYS` is an example.
   - Give the old key the `Legacy` mapping, and set its `until` to the previous version.
   - Give the new key the `Direct` mapping.
   - Use `versioned_setting`, and put the new key first in `keys`.
4. Do not change the `id` of a setting. The app database stores these IDs.
5. For a removed key, set `until` to the last version that contains the key.
6. Add each new vanilla key to `KNOWN_VANILLA_KEYS`. Include keys that only debug builds write. Use the order of the `Options` source.

## 4. Update the README

The repository does not contain the script that generated the README. Edit the file manually.

- Add a row to the summary table.
  - Use `<target>` as the label for a release. For a release candidate or a pre-release, use `<target> (<build>)`.
  - Supported keys are the keys that the game reads. This count includes old keys that a datafixer renames.
  - Fresh keys are the keys in a new `options.txt` file.
- Add a `<details>` block to the Changes section. List the supported keys and fresh keys that were added and removed.
- Update the type history table.
  - Change each `–<previous>` range to `–<target>`.
  - Change each single-version `<previous>` range to `<previous>–<target>`.
  - For a renamed key, change only the supported range.
  - Put new rows in plain ASCII sort order. Pad each cell to the width of its column.

## 5. Add the Frontend Labels

1. Open `apps/app-frontend/src/features/settings/syncing/game-settings-modal/messages.ts`.
2. Add each new setting to `catalogSettingMessages` or `catalogKeyMessages`. Then add the setting to the map from setting ID to label.
3. Run `pnpm intl:extract` in `apps/app-frontend`.
   - If the dependencies are not installed, add the entries manually to `src/locales/en-US/index.json`. Keep the keys in sorted order.

## 6. Verify and Report

1. Run `SQLX_OFFLINE=true cargo clippy -p theseus --all-targets` and `cargo fmt --all --check`.
2. Report your results to the user.
   - Include a table that shows each changed key, the change, and the catalog update for it.
   - Name the builds that you compared, and identify the checks that you did not complete.
3. Write changelog bullets that follow `standards/maintaining/CHANGELOG.md`. Use the UI names "Sync game settings" and "Game settings".
4. Do not commit unless the user tells you to.
