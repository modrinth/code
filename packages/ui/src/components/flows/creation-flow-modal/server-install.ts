import type { AbstractModrinthClient, Archon, UploadProgress } from '@modrinth/api-client'

import type { CreationFlowContextValue } from './creation-flow-context'
import { installCurseForgeModpack } from './curseforge'

/** Installs the content a server onboarding flow configured onto the given world. */
export async function installServerCreationConfig(
	client: AbstractModrinthClient,
	config: CreationFlowContextValue,
	serverId: string,
	worldId: string,
) {
	if (config.projectInstall.value) {
		await config.installServerContent(serverId, worldId)
		return
	}
	if (config.setupType.value === 'modpack' && config.modpackFile.value) {
		const auth = await client.archon.servers_v0.getFilesystemAuth(serverId)
		const file = config.modpackFile.value
		await config.trackUpload(
			(onProgress) =>
				client.kyros.content_v1.uploadModpackFile(worldId, file, config.buildProperties(), {
					auth,
					softOverride: false,
					onProgress,
				}).promise,
		)
		return
	}
	if (config.setupType.value === 'curseforge') {
		const auth = await client.archon.servers_v0.getFilesystemAuth(serverId)
		const installModpack = (onUploadProgress?: (progress: UploadProgress) => void) =>
			installCurseForgeModpack(client, worldId, config, {
				auth,
				softOverride: false,
				onUploadProgress,
			})
		if (config.curseforgeModpackFile.value || config.curseforgeServerPackFile.value) {
			await config.trackUpload(installModpack)
		} else {
			await installModpack()
		}
		return
	}
	await client.archon.content_v1.installContent(serverId, worldId, buildInstallContent(config))
}

function buildInstallContent(
	config: CreationFlowContextValue,
): Archon.Content.v1.InstallWorldContent {
	const selection = config.modpackSelection.value
	if (config.setupType.value === 'modpack' && selection) {
		return {
			content_variant: 'modpack',
			spec: {
				platform: 'modrinth',
				project_id: selection.projectId,
				version_id: selection.versionId,
			},
			soft_override: false,
			properties: config.buildProperties(),
		}
	}
	const loader = config.selectedLoader.value
	return {
		content_variant: 'bare',
		loader: (loader === 'neoforge'
			? 'neo_forge'
			: (loader ?? 'vanilla')) as Archon.Content.v1.Modloader,
		version: config.selectedLoaderVersion.value ?? '',
		game_version: config.selectedGameVersion.value ?? undefined,
		soft_override: false,
		properties: config.buildProperties(),
	}
}
