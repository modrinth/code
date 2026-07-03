<template>
	<div v-if="downloadRows.length > 0" class="flex flex-col gap-1">
		<div
			v-if="showTitle || downloadableDependencyFiles.length > 0"
			class="flex flex-wrap items-center justify-between gap-2"
		>
			<h3 v-if="showTitle" class="m-0 text-base font-semibold text-contrast">
				{{ sectionTitle }}
			</h3>
			<ButtonStyled v-if="downloadableDependencyFiles.length > 0" type="transparent">
				<button :disabled="downloadingDependencies" @click="downloadAllDependencies">
					<SpinnerIcon v-if="downloadingDependencies" aria-hidden="true" class="animate-spin" />
					<DownloadIcon v-else aria-hidden="true" />
					{{
						formatMessage(
							downloadingDependencies
								? messages.downloadingDependencies
								: messages.downloadAllDependencies,
						)
					}}
				</button>
			</ButtonStyled>
		</div>
		<div class="flex flex-col gap-2">
			<DownloadDependency
				v-for="dependency in downloadRows"
				:key="dependency.key"
				:dependency="dependency"
				@download="emit('download')"
			/>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { DownloadIcon, FileIcon, SpinnerIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	type CdnDownloadReason,
	defineMessages,
	fileTypeMessages,
	injectModrinthClient,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import type { DisplayProjectType } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import JSZip from 'jszip'
import { type Component, computed, ref } from 'vue'

import DownloadDependency from './DownloadDependency.vue'

defineOptions({
	name: 'DownloadDependencies',
})

type DownloadModalProject = Omit<Labrinth.Projects.v2.Project, 'project_type'> & {
	project_type: DisplayProjectType
	actualProjectType: Labrinth.Projects.v2.ProjectType
}

type ResolvedContent = Labrinth.Content.v3.ResolvedContent | Labrinth.Content.v3.SkippedContent

interface DownloadDependencyRow {
	key: string
	name: string
	icon?: string
	fallbackIcon?: Component
	projectHref?: string
	downloadHref?: string
	filename?: string
	typeLabel: string
	unavailableTooltip: string
	dependencies: DownloadDependencyRow[]
}

interface DownloadableDependencyFile {
	href: string
	filename: string
	name: string
}

const props = withDefaults(
	defineProps<{
		dependencies?: DownloadDependencyRow[] | null
		project?: DownloadModalProject | null
		selectedVersion?: Labrinth.Versions.v3.Version | null
		currentGameVersion?: string | null
		currentPlatform?: string | null
		downloadReason?: CdnDownloadReason
		additionalFiles?: Labrinth.Versions.v3.VersionFile[]
		showTitle?: boolean
	}>(),
	{
		dependencies: null,
		project: null,
		selectedVersion: null,
		currentGameVersion: null,
		currentPlatform: null,
		downloadReason: 'standalone',
		additionalFiles: () => [],
		showTitle: true,
	},
)

const emit = defineEmits<{
	download: []
}>()
const client = injectModrinthClient()
const { createProjectDownloadUrl } = useCdnDownloadContext()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const downloadingDependencies = ref(false)

const shouldResolveDependencies = computed(
	() => !props.dependencies && !!props.project && !!props.selectedVersion,
)

const dependencyResolutionPreferences = computed<Labrinth.Content.v3.ResolutionPreferences>(() => ({
	game_versions: props.selectedVersion?.game_versions || [],
	loaders: props.currentPlatform ? [props.currentPlatform] : props.selectedVersion?.loaders || [],
}))

const { data: dependencyResolution } = useQuery({
	queryKey: computed(() => [
		'project-download-modal',
		'content-resolve',
		props.project?.id,
		props.selectedVersion?.id,
		props.project?.project_type,
		dependencyResolutionPreferences.value,
	]),
	queryFn: () =>
		client.labrinth.content_v3.resolve({
			project_id: props.project!.id,
			version_id: props.selectedVersion!.id,
			content_type: resolveContentType(props.project!.project_type),
			selected: dependencyResolutionPreferences.value,
			target: dependencyResolutionPreferences.value,
		}),
	enabled: shouldResolveDependencies,
})

const visibleResolvedDependencies = computed<ResolvedContent[]>(() => {
	return [
		...(dependencyResolution.value?.dependencies || []),
		...(dependencyResolution.value?.skipped || []),
	].filter(shouldShowDependency)
})

const dependencyVersionIds = computed<string[]>(() => {
	return [
		...new Set(
			visibleResolvedDependencies.value
				.filter((dependency) => !('reason' in dependency))
				.map((dependency) => dependency.version_id)
				.filter((versionId): versionId is string => !!versionId),
		),
	]
})

const { data: dependencyVersions } = useQuery({
	queryKey: computed(() => [
		'project-download-modal',
		'resolved-versions',
		dependencyVersionIds.value,
	]),
	queryFn: () => client.labrinth.versions_v3.getVersions(dependencyVersionIds.value),
	enabled: computed(() => shouldResolveDependencies.value && dependencyVersionIds.value.length > 0),
})

const dependencyVersionById = computed(() => {
	const map = new Map<string, Labrinth.Versions.v3.Version>()
	for (const version of dependencyVersions.value || []) {
		if (!version) continue
		map.set(version.id, version)
	}
	return map
})

const dependencyProjectIds = computed<string[]>(() => {
	return [
		...new Set(
			visibleResolvedDependencies.value
				.map((dependency) => dependency.project_id)
				.filter((projectId): projectId is string => !!projectId),
		),
	]
})

const { data: dependencyProjects } = useQuery({
	queryKey: computed(() => [
		'project-download-modal',
		'resolved-projects',
		dependencyProjectIds.value,
	]),
	queryFn: () => client.labrinth.projects_v2.getMultiple(dependencyProjectIds.value),
	enabled: computed(() => shouldResolveDependencies.value && dependencyProjectIds.value.length > 0),
})

const dependencyProjectById = computed(() => {
	const map = new Map<string, Labrinth.Projects.v2.Project>()
	for (const project of dependencyProjects.value || []) {
		map.set(project.id, project)
	}
	return map
})

const dependenciesByParentVersionId = computed(() => {
	const map = new Map<string, ResolvedContent[]>()

	for (const dependency of visibleResolvedDependencies.value) {
		if (!dependency.dependent_on_version_id) continue

		const dependencies = map.get(dependency.dependent_on_version_id) || []
		dependencies.push(dependency)
		map.set(dependency.dependent_on_version_id, dependencies)
	}

	return map
})

const dependenciesLoaded = computed(() => {
	if (!shouldResolveDependencies.value) return false
	if (!dependencyResolution.value) return false
	if (
		dependencyResolution.value.primary.version_id &&
		dependencyResolution.value.primary.version_id !== props.selectedVersion?.id
	) {
		return false
	}
	if (
		!dependencyVersionIds.value.every((versionId) => dependencyVersionById.value.has(versionId))
	) {
		return false
	}
	if (
		!dependencyProjectIds.value.every((projectId) => dependencyProjectById.value.has(projectId))
	) {
		return false
	}
	return true
})

const resolvedDependencyRows = computed<DownloadDependencyRow[]>(() => {
	if (!dependenciesLoaded.value) return []

	const primaryVersionId =
		dependencyResolution.value?.primary.version_id || props.selectedVersion?.id
	if (!primaryVersionId) return []

	const dependencies = dependenciesByParentVersionId.value.get(primaryVersionId) || []

	return dependencies.flatMap((dependency) => {
		const row = createDependencyRow(dependency)
		return row ? [row] : []
	})
})

const dependencyRows = computed<DownloadDependencyRow[]>(
	() => props.dependencies || resolvedDependencyRows.value,
)

const additionalFileRows = computed<DownloadDependencyRow[]>(() =>
	props.additionalFiles.map((file) => ({
		key: `additional-file-${additionalFileKey(file)}`,
		name: file.filename,
		fallbackIcon: FileIcon,
		downloadHref: getDownloadUrl(file.url),
		filename: file.filename,
		typeLabel: fileTypeLabel(file.file_type),
		unavailableTooltip: formatMessage(messages.unavailableFile),
		dependencies: [],
	})),
)

const downloadRows = computed<DownloadDependencyRow[]>(() => [
	...dependencyRows.value,
	...additionalFileRows.value,
])

const sectionTitle = computed(() =>
	formatMessage(
		dependencyRows.value.length > 0
			? messages.dependenciesTitle
			: messages.additionalFilesTitle,
	),
)

const downloadableDependencyFiles = computed<DownloadableDependencyFile[]>(() =>
	collectDownloadableDependencyFiles(downloadRows.value),
)

const dependencyZipFilename = computed(
	() => `${sanitizeFilename(props.project?.title || 'Project')} dependencies.zip`,
)

function primaryFileForVersion(version?: Labrinth.Versions.v3.Version) {
	return version?.files?.find((file) => file.primary) || version?.files?.[0]
}

function shouldShowDependency(dependency: ResolvedContent) {
	return !('reason' in dependency && dependency.reason === 'quilt_fabric_api')
}

function createDependencyRow(dependency: ResolvedContent): DownloadDependencyRow | null {
	const versionId = dependency.version_id ?? undefined
	const version = versionId ? dependencyVersionById.value.get(versionId) : undefined
	const project = dependencyProjectById.value.get(dependency.project_id)
	if (!project) return null

	const primaryFile = primaryFileForVersion(version)
	const unavailableTooltip =
		'reason' in dependency && dependency.reason
			? skippedReasonLabel(dependency.reason)
			: formatMessage(messages.unavailableDependency)
	const name = project.title

	return {
		key: `${dependency.project_id}-${versionId ?? 'unresolved'}-${
			'reason' in dependency ? dependency.reason : 'resolved'
		}`,
		name,
		icon: project.icon_url ?? undefined,
		projectHref: `/${project.project_type}/${project.slug || project.id}`,
		downloadHref:
			'reason' in dependency || !primaryFile ? undefined : getDownloadUrl(primaryFile.url),
		filename: primaryFile?.filename,
		typeLabel: 'Required',
		unavailableTooltip,
		dependencies: (versionId && dependenciesByParentVersionId.value.get(versionId)
			? dependenciesByParentVersionId.value.get(versionId)!
			: []
		).flatMap((subDependency) => {
			const row = createDependencyRow(subDependency)
			return row ? [row] : []
		}),
	}
}

function skippedReasonLabel(reason: Labrinth.Content.v3.SkippedContent['reason']) {
	return (
		{
			already_installed: formatMessage(messages.alreadyInstalledDependency),
			duplicate_project: formatMessage(messages.duplicateDependency),
			conflicting_dependency: formatMessage(messages.conflictingDependency),
			no_compatible_version: formatMessage(messages.noCompatibleDependency),
			missing_version: formatMessage(messages.missingDependencyVersion),
			quilt_fabric_api: formatMessage(messages.quiltFabricApiDependency),
		}[reason] || formatMessage(messages.unavailableDependency)
	)
}

function resolveContentType(projectType: DisplayProjectType): Labrinth.Content.v3.ContentType {
	return ['mod', 'plugin', 'datapack', 'resourcepack', 'shader', 'modpack'].includes(projectType)
		? (projectType as Labrinth.Content.v3.ContentType)
		: 'mod'
}

function getDownloadUrl(url: string) {
	return createProjectDownloadUrl(url, {
		reason: props.downloadReason,
		gameVersion: props.currentGameVersion ?? undefined,
		loader: props.currentPlatform ?? undefined,
	})
}

function fileTypeLabel(type?: Labrinth.Versions.v3.FileType | null) {
	return formatMessage(fileTypeMessages[type ?? 'unknown'] ?? fileTypeMessages.unknown)
}

function additionalFileKey(file: Labrinth.Versions.v3.VersionFile) {
	return file.hashes?.sha1 ?? file.filename
}

function collectDownloadableDependencyFiles(
	rows: DownloadDependencyRow[],
	seenHrefs = new Set<string>(),
): DownloadableDependencyFile[] {
	const files: DownloadableDependencyFile[] = []

	for (const row of rows) {
		if (row.downloadHref && !seenHrefs.has(row.downloadHref)) {
			seenHrefs.add(row.downloadHref)
			files.push({
				href: row.downloadHref,
				filename: row.filename || filenameFromUrl(row.downloadHref),
				name: row.name,
			})
		}

		files.push(...collectDownloadableDependencyFiles(row.dependencies, seenHrefs))
	}

	return files
}

async function downloadAllDependencies() {
	if (downloadingDependencies.value || downloadableDependencyFiles.value.length === 0) return

	downloadingDependencies.value = true

	try {
		const zip = new JSZip()
		const usedFilenames = new Set<string>()

		await Promise.all(
			downloadableDependencyFiles.value.map(async (file) => {
				const response = await fetch(file.href)

				if (!response.ok) {
					throw new Error(`Failed to download ${file.name}`)
				}

				zip.file(uniqueFilename(file.filename, usedFilenames), await response.blob())
			}),
		)

		downloadBlob(
			await zip.generateAsync({
				type: 'blob',
				mimeType: 'application/zip',
			}),
			dependencyZipFilename.value,
		)
		emit('download')
	} catch (error) {
		console.error('Failed to download dependencies:', error)
		addNotification({
			title: formatMessage(messages.downloadDependenciesFailedTitle),
			text: formatMessage(messages.downloadDependenciesFailedText),
			type: 'error',
		})
	} finally {
		downloadingDependencies.value = false
	}
}

function downloadBlob(blob: Blob, filename: string) {
	const url = URL.createObjectURL(blob)
	const link = document.createElement('a')

	link.href = url
	link.download = filename
	document.body.appendChild(link)
	link.click()
	link.remove()
	window.setTimeout(() => URL.revokeObjectURL(url), 0)
}

function filenameFromUrl(url: string) {
	try {
		const filename = new URL(url).pathname.split('/').pop()
		return filename ? decodeURIComponent(filename) : 'dependency.jar'
	} catch {
		return 'dependency.jar'
	}
}

function sanitizeFilename(value: string) {
	const sanitized = value
		.replace(/[<>:"/\\|?*]/g, '')
		.replace(/\s+/g, ' ')
		.trim()

	return sanitized || 'dependencies'
}

function uniqueFilename(filename: string, usedFilenames: Set<string>) {
	const sanitizedFilename = sanitizeFilename(filename)

	if (!usedFilenames.has(sanitizedFilename)) {
		usedFilenames.add(sanitizedFilename)
		return sanitizedFilename
	}

	const extensionIndex = sanitizedFilename.lastIndexOf('.')
	const basename =
		extensionIndex > 0 ? sanitizedFilename.slice(0, extensionIndex) : sanitizedFilename
	const extension = extensionIndex > 0 ? sanitizedFilename.slice(extensionIndex) : ''
	let index = 2
	let candidate = `${basename} (${index})${extension}`

	while (usedFilenames.has(candidate)) {
		index += 1
		candidate = `${basename} (${index})${extension}`
	}

	usedFilenames.add(candidate)
	return candidate
}

const messages = defineMessages({
	dependenciesTitle: {
		id: 'project.download.dependencies-title',
		defaultMessage: 'Dependencies',
	},
	additionalFilesTitle: {
		id: 'project.download.additional-files-title',
		defaultMessage: 'Additional files',
	},
	downloadAllDependencies: {
		id: 'project.download.dependencies-download-all',
		defaultMessage: 'Download all',
	},
	downloadingDependencies: {
		id: 'project.download.dependencies-downloading',
		defaultMessage: 'Downloading...',
	},
	downloadDependenciesFailedTitle: {
		id: 'project.download.dependencies-failed-title',
		defaultMessage: 'Could not download dependencies',
	},
	downloadDependenciesFailedText: {
		id: 'project.download.dependencies-failed-text',
		defaultMessage: 'One or more dependency files could not be downloaded. Please try again.',
	},
	alreadyInstalledDependency: {
		id: 'project.download.dependency-already-installed',
		defaultMessage: 'This dependency is already installed',
	},
	conflictingDependency: {
		id: 'project.download.dependency-conflicting',
		defaultMessage: 'This dependency conflicts with another dependency',
	},
	duplicateDependency: {
		id: 'project.download.dependency-duplicate',
		defaultMessage: 'This dependency is already included',
	},
	missingDependencyVersion: {
		id: 'project.download.dependency-missing-version',
		defaultMessage: 'This dependency version is unavailable',
	},
	noCompatibleDependency: {
		id: 'project.download.dependency-no-compatible-version',
		defaultMessage: 'No compatible version is available for this dependency',
	},
	quiltFabricApiDependency: {
		id: 'project.download.dependency-quilt-fabric-api',
		defaultMessage: 'Fabric API is skipped for Quilt',
	},
	unavailableDependency: {
		id: 'project.download.dependency-unavailable',
		defaultMessage: 'This dependency cannot be downloaded',
	},
	unavailableFile: {
		id: 'project.download.file-unavailable',
		defaultMessage: 'This file cannot be downloaded',
	},
})
</script>
