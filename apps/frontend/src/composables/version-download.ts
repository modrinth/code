import type { Labrinth } from '@modrinth/api-client'
import {
	type CdnDownloadReason,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import type { DisplayProjectType } from '@modrinth/utils'
import { useQueryClient } from '@tanstack/vue-query'
import { type Ref, ref } from 'vue'

import {
	primaryFileForVersion,
	resolveContentType,
} from '~/components/ui/ProjectDownloadModal/download-modal-provider'
import { STALE_TIME } from '~/composables/queries/project'
import { dedupeDownloadFiles, type DownloadableFile, downloadFiles } from '~/helpers/download-files'

type VersionDownloadProject = {
	id: string
	title: string
	project_type: DisplayProjectType
}

interface VersionDownloadOptions {
	project: Ref<VersionDownloadProject | null | undefined>
	downloadReason: Ref<CdnDownloadReason>
	preferredGameVersions: Ref<string[]>
	preferredLoaders: Ref<string[]>
}

export function useVersionDownload(options: VersionDownloadOptions) {
	const client = injectModrinthClient()
	const queryClient = useQueryClient()
	const tags = useGeneratedState()
	const { createProjectDownloadUrl } = useCdnDownloadContext()
	const { addNotification } = injectNotificationManager()
	const { formatMessage } = useVIntl()

	const downloadingVersionId = ref<string | null>(null)

	function hasDownloadDependencies(version: Labrinth.Versions.v3.Version) {
		return (
			version.files.some((file) => file.file_type === 'required-resource-pack') ||
			(version.dependencies ?? []).some(
				(dependency) =>
					dependency.dependency_type === 'required' &&
					(!!dependency.project_id || !!dependency.version_id),
			)
		)
	}

	function selectGameVersion(version: Labrinth.Versions.v3.Version) {
		return (
			options.preferredGameVersions.value.find((gameVersion) =>
				version.game_versions.includes(gameVersion),
			) ??
			tags.value.gameVersions.find((gameVersion) =>
				version.game_versions.includes(gameVersion.version),
			)?.version ??
			version.game_versions.at(-1) ??
			null
		)
	}

	function selectLoaders(version: Labrinth.Versions.v3.Version) {
		const preferredLoaders = options.preferredLoaders.value.filter((loader) =>
			version.loaders.includes(loader),
		)
		return preferredLoaders.length > 0 ? preferredLoaders : version.loaders
	}

	function isDataPackVersion(version: Labrinth.Versions.v3.Version) {
		return version.loaders.some((loader) => tags.value.loaderData.dataPackLoaders.includes(loader))
	}

	async function resolveDependencyVersions(
		project: VersionDownloadProject,
		version: Labrinth.Versions.v3.Version,
		preferences: Labrinth.Content.v3.ResolutionPreferences,
	) {
		const resolution = await queryClient.ensureQueryData({
			queryKey: [
				'project-download-modal',
				'content-resolve',
				project.id,
				version.id,
				project.project_type,
				preferences,
			],
			queryFn: () =>
				client.labrinth.content_v3.resolve({
					project_id: project.id,
					version_id: version.id,
					content_type: resolveContentType(project.project_type),
					selected: preferences,
					target: preferences,
				}),
			staleTime: STALE_TIME,
		})

		const versionIds = [
			...new Set(
				resolution.dependencies
					.map((dependency) => dependency.version_id)
					.filter((versionId): versionId is string => !!versionId),
			),
		].sort()
		if (versionIds.length === 0) return []

		return queryClient.ensureQueryData({
			queryKey: ['project-download-modal', 'resolved-versions', versionIds],
			queryFn: () => client.labrinth.versions_v3.getVersions(versionIds),
			staleTime: STALE_TIME,
		})
	}

	async function collectDownloadFiles(
		project: VersionDownloadProject,
		version: Labrinth.Versions.v3.Version,
	): Promise<DownloadableFile[]> {
		const gameVersion = selectGameVersion(version)
		const loaders = selectLoaders(version)
		const urlContext = {
			gameVersion: gameVersion ?? undefined,
			loader: loaders.length === 1 ? loaders[0] : undefined,
		}

		const primaryFile = primaryFileForVersion(version)
		const versionFiles = version.files
			.filter((file) => file === primaryFile || file.file_type === 'required-resource-pack')
			.map((file) => ({
				href: createProjectDownloadUrl(file.url, {
					...urlContext,
					reason: options.downloadReason.value,
				}),
				filename: file.filename,
			}))

		const dependencyVersions = await resolveDependencyVersions(project, version, {
			game_versions: gameVersion ? [gameVersion] : [],
			loaders,
		})
		const dependencyFiles = dependencyVersions.flatMap((dependencyVersion) => {
			const dependencyPrimaryFile = primaryFileForVersion(dependencyVersion)
			const includeResourcePacks = isDataPackVersion(dependencyVersion)

			return dependencyVersion.files
				.filter(
					(file) =>
						file === dependencyPrimaryFile ||
						(includeResourcePacks && file.file_type === 'required-resource-pack'),
				)
				.map((file) => ({
					href: createProjectDownloadUrl(file.url, { ...urlContext, reason: 'dependency' }),
					filename: file.filename,
				}))
		})

		return dedupeDownloadFiles([...versionFiles, ...dependencyFiles])
	}

	async function downloadVersion(version: Labrinth.Versions.v3.Version) {
		const project = options.project.value
		if (!project || downloadingVersionId.value) return false

		downloadingVersionId.value = version.id

		try {
			await downloadFiles(await collectDownloadFiles(project, version))

			return true
		} catch (error) {
			console.error('Failed to download version files:', error)
			addNotification({
				title: formatMessage(messages.downloadFailedTitle),
				text: formatMessage(messages.downloadFailedText),
				type: 'error',
			})
			return false
		} finally {
			downloadingVersionId.value = null
		}
	}

	return {
		downloadingVersionId,
		hasDownloadDependencies,
		downloadVersion,
	}
}

const messages = defineMessages({
	downloadFailedTitle: {
		id: 'project.download.zip-failed-title',
		defaultMessage: 'Could not download files',
	},
	downloadFailedText: {
		id: 'project.download.zip-failed-text',
		defaultMessage: 'One or more files could not be downloaded. Please try again.',
	},
})
