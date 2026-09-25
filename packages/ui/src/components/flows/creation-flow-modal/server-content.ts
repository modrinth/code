import type { AbstractModrinthClient, Archon, Labrinth } from '@modrinth/api-client'

import { retainServerContextRuntime } from '../../../composables/server-context-runtime'
import type { CreationFlowContextValue, ProjectInstallSelection } from './creation-flow-context'

const serverEnvironments = [
	'client_and_server',
	'server_only',
	'server_only_client_optional',
	'dedicated_server_only',
	'client_or_server',
	'client_or_server_prefers_both',
	'client_only_server_optional',
]

export async function searchServerContent(
	client: AbstractModrinthClient,
	query: string,
	limit = 10,
) {
	const results = await client.labrinth.projects_v3.search({
		query: query || undefined,
		new_filters: `project_types IN ["modpack", "mod", "plugin", "datapack"] AND environment IN ${JSON.stringify(serverEnvironments)}`,
		limit,
	})
	return {
		hits: results.hits.map((hit) => ({
			project_id: hit.project_id,
			project_type: ['modpack', 'plugin', 'datapack', 'mod'].find((type) =>
				hit.project_types.includes(type as Labrinth.Projects.v3.ProjectType),
			),
			title: hit.name,
			icon_url: hit.icon_url ?? '',
		})),
		total_hits: results.total_hits,
		offset: (results.page - 1) * results.hits_per_page,
		limit: results.hits_per_page,
	}
}

export async function prepareServerContent(
	client: AbstractModrinthClient,
	projectId: string,
	projectType: string,
	availableLoaders: string[],
	versionId?: string,
): Promise<ProjectInstallSelection> {
	if (!['mod', 'plugin', 'datapack'].includes(projectType)) {
		throw new Error('This content cannot be installed on a server.')
	}
	const [project, versions] = await Promise.all([
		client.labrinth.projects_v3.get(projectId),
		versionId
			? client.labrinth.versions_v3.getVersion(versionId).then((version) => [version])
			: client.labrinth.versions_v3.getProjectVersions(projectId),
	])
	const getLoaders = (version: Labrinth.Versions.v3.Version) =>
		availableLoaders.filter((loader) => {
			if (projectType === 'datapack') {
				return loader === 'vanilla' && version.loaders.includes('datapack')
			}
			if (version.loaders.includes(loader)) return true
			return (
				['paper', 'purpur'].includes(loader) &&
				version.loaders.some((value) => ['bukkit', 'spigot', 'paper'].includes(value))
			)
		})
	const version = [...versions]
		.sort((a, b) => Date.parse(b.date_published) - Date.parse(a.date_published))
		.find(
			(version) =>
				version.project_id === projectId &&
				(!version.environment || serverEnvironments.includes(version.environment)) &&
				getLoaders(version).length > 0 &&
				version.game_versions.length > 0,
		)
	if (!version) throw new Error('No compatible server version is available for this project.')
	const gameVersions = [...version.game_versions].sort((a, b) =>
		b.localeCompare(a, undefined, { numeric: true }),
	)
	return {
		projectId,
		versionId: version.id,
		contentType: projectType as 'mod' | 'plugin' | 'datapack',
		title: project.name,
		iconUrl: project.icon_url,
		link: `/${projectType}/${project.slug ?? projectId}`,
		compatibleLoaders: getLoaders(version),
		gameVersions,
		releaseGameVersions: new Set(
			gameVersions.filter((version) => /^\d+\.\d+(\.\d+)?$/.test(version)),
		),
	}
}

export async function installServerContent(
	client: AbstractModrinthClient,
	ctx: CreationFlowContextValue,
	serverId: string,
	worldId: string,
) {
	const selection = ctx.projectInstall.value
	const loader = ctx.selectedLoader.value
	const gameVersion = ctx.selectedGameVersion.value
	const loaderVersion = ctx.selectedLoaderVersion.value ?? ''
	if (!selection?.contentType || !selection.versionId || !loader || !gameVersion) {
		throw new Error('Select content, a loader, and a Minecraft version before installing.')
	}
	if (loader !== 'vanilla' && !loaderVersion) {
		throw new Error('Select a loader version before installing.')
	}
	const properties = ctx.buildProperties()
	const resolved = await client.labrinth.content_v3.resolve({
		project_id: selection.projectId,
		version_id: selection.versionId,
		content_type: selection.contentType,
		selected: {
			game_versions: [gameVersion],
			loaders: selection.contentType === 'datapack' ? ['datapack'] : [loader],
		},
	})
	const runtime = retainServerContextRuntime(client, serverId)
	let unsubscribe = () => {}
	let timeout: ReturnType<typeof setTimeout> | undefined
	try {
		await runtime.waitUntilReady()
		const previousIds = new Set(runtime.installProgressItems.value.map((item) => item.id))
		let started = false
		const completed = new Promise<void>((resolve, reject) => {
			unsubscribe = client.archon.sockets.on(serverId, 'install-progress', ({ items }) => {
				const item = items.find(
					(item) =>
						item.world_id === worldId &&
						item.key.type === 'platform' &&
						item.key.platform === loader &&
						item.key.game_version === gameVersion &&
						item.key.platform_version === loaderVersion,
				)
				if (!item) return
				if (!previousIds.has(item.id) || (item.progress != null && item.progress < 100)) {
					started = true
				}
				if (!started) return
				if (item?.error) reject(new Error(item.error))
				else if (item?.progress === 100) resolve()
			})
			timeout = setTimeout(() => reject(new Error('Timed out waiting for server setup.')), 600_000)
		})
		await Promise.all([
			completed,
			client.archon.content_v1.installContent(serverId, worldId, {
				content_variant: 'bare',
				loader: loader === 'neoforge' ? 'neo_forge' : (loader as Archon.Content.v1.Modloader),
				version: loaderVersion,
				game_version: gameVersion,
				soft_override: false,
				properties,
			}),
		])
		await client.archon.content_v1.addAddons(
			serverId,
			worldId,
			[resolved.primary, ...resolved.dependencies].map((item) => ({
				project_id: item.project_id,
				version_id: item.version_id,
			})),
		)
	} finally {
		unsubscribe()
		clearTimeout(timeout)
		runtime.release()
	}
}
