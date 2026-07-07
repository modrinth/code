import type { AbstractModrinthClient, Labrinth } from '@modrinth/api-client'
import { FileIcon } from '@modrinth/assets'
import {
	type CdnDownloadReason,
	createContext,
	defineMessages,
	fileTypeMessages,
	injectModrinthClient,
	useVIntl,
} from '@modrinth/ui'
import type { DisplayProjectType } from '@modrinth/utils'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { type Component, computed, type ComputedRef } from 'vue'

import { STALE_TIME } from '~/composables/queries/project'

type DownloadModalProject = Omit<Labrinth.Projects.v2.Project, 'project_type'> & {
	project_type: DisplayProjectType
	actualProjectType: Labrinth.Projects.v2.ProjectType
}

type ResolvedContent = Labrinth.Content.v3.ResolvedContent | Labrinth.Content.v3.SkippedContent

export interface DownloadDependencyRow {
	key: string
	name: string
	icon?: string
	fallbackIcon?: Component
	projectHref?: string
	downloadHref?: string
	filename?: string
	fileSize?: number
	metadataLabel?: string
	typeLabel: string
	unavailableTooltip: string
	dependencies: DownloadDependencyRow[]
}

export interface DownloadableDependencyFile {
	href: string
	filename: string
	name: string
}

export interface ProjectDownloadSelection {
	currentGameVersion: string | null
	currentPlatform: string | null
	selectedVersion: Labrinth.Versions.v3.Version | null
	selectedPrimaryFile: Labrinth.Versions.v3.VersionFile | null
}

interface DownloadModalProviderOptions {
	project: ComputedRef<DownloadModalProject | null>
	selectedVersion: ComputedRef<Labrinth.Versions.v3.Version | null>
	currentGameVersion: ComputedRef<string | null>
	currentPlatform: ComputedRef<string | null>
	downloadReason: ComputedRef<CdnDownloadReason>
	additionalFiles: ComputedRef<Labrinth.Versions.v3.VersionFile[]>
}

export interface DownloadModalProvider {
	visibleDependencyRows: ComputedRef<DownloadDependencyRow[]>
	duplicateDependencyRowsHidden: ComputedRef<boolean>
	downloadRows: ComputedRef<DownloadDependencyRow[]>
	downloadableDependencyFiles: ComputedRef<DownloadableDependencyFile[]>
	downloadableDependencyFilesLoaded: ComputedRef<boolean>
	preloadDependenciesForSelection: (selection: ProjectDownloadSelection) => Promise<void>
}

export const [injectDownloadModalProvider, provideDownloadModalContext] =
	createContext<DownloadModalProvider>('DownloadModal')

export function provideDownloadModalProvider(
	options: DownloadModalProviderOptions,
): DownloadModalProvider {
	const client = injectModrinthClient()
	const queryClient = useQueryClient()
	const { createProjectDownloadUrl } = useCdnDownloadContext()
	const { formatMessage } = useVIntl()

	const shouldResolveDependencies = computed(
		() => !!options.project.value && !!options.selectedVersion.value,
	)

	const dependencyResolutionPreferences = computed(() =>
		createResolutionPreferences(options.selectedVersion.value, options.currentPlatform.value),
	)

	const { data: dependencyResolution, isFetching: dependencyResolutionFetching } = useQuery({
		...dependencyResolutionQueryOptions(
			client,
			options.project,
			options.selectedVersion,
			dependencyResolutionPreferences,
		),
		enabled: shouldResolveDependencies,
	})

	const visibleResolvedDependencies = computed<ResolvedContent[]>(() =>
		visibleDependencies(dependencyResolution.value),
	)

	const dependencyVersionIds = computed(() =>
		sortedUnique(
			visibleResolvedDependencies.value
				.filter((dependency) => !('reason' in dependency))
				.map((dependency) => dependency.version_id)
				.filter((versionId): versionId is string => !!versionId),
		),
	)

	const { data: dependencyVersions, isFetching: dependencyVersionsFetching } = useQuery({
		...dependencyVersionsQueryOptions(client, dependencyVersionIds),
		enabled: computed(
			() => shouldResolveDependencies.value && dependencyVersionIds.value.length > 0,
		),
	})

	const dependencyVersionById = computed(() => {
		const map = new Map<string, Labrinth.Versions.v3.Version>()
		for (const version of dependencyVersions.value || []) {
			if (!version) continue
			map.set(version.id, version)
		}
		return map
	})

	const dependencyProjectIds = computed(() =>
		sortedUnique(
			visibleResolvedDependencies.value
				.map((dependency) => dependency.project_id)
				.filter((projectId): projectId is string => !!projectId),
		),
	)

	const { data: dependencyProjects, isFetching: dependencyProjectsFetching } = useQuery({
		...dependencyProjectsQueryOptions(client, dependencyProjectIds),
		enabled: computed(
			() => shouldResolveDependencies.value && dependencyProjectIds.value.length > 0,
		),
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
		if (dependencyResolutionFetching.value) return false
		if (!dependencyResolution.value) return false
		if (
			dependencyResolution.value.primary.version_id &&
			dependencyResolution.value.primary.version_id !== options.selectedVersion.value?.id
		) {
			return false
		}
		if (
			dependencyVersionsFetching.value ||
			!dependencyVersionIds.value.every((versionId) => dependencyVersionById.value.has(versionId))
		) {
			return false
		}
		if (
			dependencyProjectsFetching.value ||
			!dependencyProjectIds.value.every((projectId) => dependencyProjectById.value.has(projectId))
		) {
			return false
		}
		return true
	})

	const resolvedDependencyRows = computed<DownloadDependencyRow[]>(() => {
		if (!dependenciesLoaded.value) return []

		const primaryVersionId =
			dependencyResolution.value?.primary.version_id || options.selectedVersion.value?.id
		if (!primaryVersionId) return []

		const dependencies = dependenciesByParentVersionId.value.get(primaryVersionId) || []

		return dependencies.flatMap((dependency) => {
			const row = createDependencyRow(dependency)
			return row ? [row] : []
		})
	})

	const visibleDependencyRows = computed<DownloadDependencyRow[]>(() =>
		dedupeDependencyRows(resolvedDependencyRows.value),
	)

	const duplicateDependencyRowsHidden = computed(() =>
		hasSkippedDuplicateDependency(dependencyResolution.value) ||
		hasDuplicateDependencyRows(resolvedDependencyRows.value),
	)

	const additionalFileRows = computed<DownloadDependencyRow[]>(() =>
		options.additionalFiles.value.map((file) => ({
			key: `additional-file-${additionalFileKey(file)}`,
			name: file.filename,
			fallbackIcon: FileIcon,
			downloadHref: getDownloadUrl(file.url),
			filename: file.filename,
			fileSize: file.size,
			metadataLabel: fileTypeLabel(file.file_type),
			typeLabel: fileTypeLabel(file.file_type),
			unavailableTooltip: formatMessage(messages.unavailableFile),
			dependencies: [],
		})),
	)

	const downloadRows = computed<DownloadDependencyRow[]>(() => [
		...visibleDependencyRows.value,
		...additionalFileRows.value,
	])

	const downloadableDependencyFiles = computed<DownloadableDependencyFile[]>(() =>
		collectDownloadableDependencyFiles(visibleDependencyRows.value),
	)

	const downloadableDependencyFilesLoaded = computed(() => {
		if (!shouldResolveDependencies.value) return false
		return dependenciesLoaded.value
	})

	async function preloadDependenciesForSelection(selection: ProjectDownloadSelection) {
		if (!options.project.value || !selection.selectedVersion) return

		const preferences = createResolutionPreferences(
			selection.selectedVersion,
			selection.currentPlatform,
		)

		const resolution = await queryClient.ensureQueryData({
			queryKey: [
				'project-download-modal',
				'content-resolve',
				options.project.value.id,
				selection.selectedVersion.id,
				options.project.value.project_type,
				preferences,
			],
			queryFn: () =>
				client.labrinth.content_v3.resolve({
					project_id: options.project.value!.id,
					version_id: selection.selectedVersion!.id,
					content_type: resolveContentType(options.project.value!.project_type),
					selected: preferences,
					target: preferences,
				}),
			staleTime: STALE_TIME,
		})
		const visible = visibleDependencies(resolution)
		const versionIds = getDependencyVersionIds(visible)
		const projectIds = getDependencyProjectIds(visible)

		await Promise.all([
			versionIds.length > 0
				? queryClient.ensureQueryData({
						queryKey: ['project-download-modal', 'resolved-versions', versionIds],
						queryFn: () => client.labrinth.versions_v3.getVersions(versionIds),
						staleTime: STALE_TIME,
					})
				: Promise.resolve(),
			projectIds.length > 0
				? queryClient.ensureQueryData({
						queryKey: ['project-download-modal', 'resolved-projects', projectIds],
						queryFn: () => client.labrinth.projects_v2.getMultiple(projectIds),
						staleTime: STALE_TIME,
					})
				: Promise.resolve(),
		])
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
			fileSize: primaryFile?.size,
			metadataLabel: version?.version_number ?? formatMessage(messages.anyCompatibleDependency),
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

	function getDownloadUrl(url: string) {
		return createProjectDownloadUrl(url, {
			reason: options.downloadReason.value,
			gameVersion: options.currentGameVersion.value ?? undefined,
			loader: options.currentPlatform.value ?? undefined,
		})
	}

	function fileTypeLabel(type?: Labrinth.Versions.v3.FileType | null) {
		return formatMessage(fileTypeMessages[type ?? 'unknown'] ?? fileTypeMessages.unknown)
	}

	const provider = {
		visibleDependencyRows,
		duplicateDependencyRowsHidden,
		downloadRows,
		downloadableDependencyFiles,
		downloadableDependencyFilesLoaded,
		preloadDependenciesForSelection,
	}

	provideDownloadModalContext(provider)

	return provider
}

function dependencyResolutionQueryOptions(
	client: AbstractModrinthClient,
	project: ComputedRef<DownloadModalProject | null>,
	selectedVersion: ComputedRef<Labrinth.Versions.v3.Version | null>,
	preferences: ComputedRef<Labrinth.Content.v3.ResolutionPreferences>,
) {
	return {
		queryKey: computed(() => [
			'project-download-modal',
			'content-resolve',
			project.value?.id,
			selectedVersion.value?.id,
			project.value?.project_type,
			preferences.value,
		]),
		queryFn: () =>
			client.labrinth.content_v3.resolve({
				project_id: project.value!.id,
				version_id: selectedVersion.value!.id,
				content_type: resolveContentType(project.value!.project_type),
				selected: preferences.value,
				target: preferences.value,
			}),
		staleTime: STALE_TIME,
	}
}

function dependencyVersionsQueryOptions(
	client: AbstractModrinthClient,
	versionIds: ComputedRef<string[]>,
) {
	return {
		queryKey: computed(() => ['project-download-modal', 'resolved-versions', versionIds.value]),
		queryFn: () => client.labrinth.versions_v3.getVersions(versionIds.value),
		staleTime: STALE_TIME,
	}
}

function dependencyProjectsQueryOptions(
	client: AbstractModrinthClient,
	projectIds: ComputedRef<string[]>,
) {
	return {
		queryKey: computed(() => ['project-download-modal', 'resolved-projects', projectIds.value]),
		queryFn: () => client.labrinth.projects_v2.getMultiple(projectIds.value),
		staleTime: STALE_TIME,
	}
}

function createResolutionPreferences(
	version: Labrinth.Versions.v3.Version | null,
	currentPlatform: string | null,
): Labrinth.Content.v3.ResolutionPreferences {
	return {
		game_versions: version?.game_versions || [],
		loaders: currentPlatform ? [currentPlatform] : version?.loaders || [],
	}
}

function visibleDependencies(resolution?: Labrinth.Content.v3.ResolveContentPlan) {
	return [...(resolution?.dependencies || []), ...(resolution?.skipped || [])].filter(
		shouldShowDependency,
	)
}

function getDependencyVersionIds(dependencies: ResolvedContent[]) {
	return sortedUnique(
		dependencies
			.filter((dependency) => !('reason' in dependency))
			.map((dependency) => dependency.version_id)
			.filter((versionId): versionId is string => !!versionId),
	)
}

function getDependencyProjectIds(dependencies: ResolvedContent[]) {
	return sortedUnique(
		dependencies
			.map((dependency) => dependency.project_id)
			.filter((projectId): projectId is string => !!projectId),
	)
}

function primaryFileForVersion(version?: Labrinth.Versions.v3.Version) {
	return version?.files?.find((file) => file.primary) || version?.files?.[0]
}

function shouldShowDependency(dependency: ResolvedContent) {
	return !(
		'reason' in dependency && ['duplicate_project', 'quilt_fabric_api'].includes(dependency.reason)
	)
}

function hasSkippedDuplicateDependency(resolution?: Labrinth.Content.v3.ResolveContentPlan) {
	return (resolution?.skipped || []).some((dependency) => dependency.reason === 'duplicate_project')
}

function resolveContentType(projectType: DisplayProjectType): Labrinth.Content.v3.ContentType {
	return ['mod', 'plugin', 'datapack', 'resourcepack', 'shader', 'modpack'].includes(projectType)
		? (projectType as Labrinth.Content.v3.ContentType)
		: 'mod'
}

function additionalFileKey(file: Labrinth.Versions.v3.VersionFile) {
	return file.hashes?.sha1 ?? file.filename
}

function dedupeDependencyRows(
	rows: DownloadDependencyRow[],
	seenDependencies = new Set<string>(),
): DownloadDependencyRow[] {
	return rows.flatMap((row) => {
		const identity = dependencyRowIdentity(row)
		if (seenDependencies.has(identity)) return []

		seenDependencies.add(identity)

		return [
			{
				...row,
				dependencies: dedupeDependencyRows(row.dependencies, seenDependencies),
			},
		]
	})
}

function dependencyRowIdentity(row: DownloadDependencyRow) {
	return row.projectHref ?? row.downloadHref ?? row.key
}

function hasDuplicateDependencyRows(
	rows: DownloadDependencyRow[],
	seenDependencies = new Set<string>(),
): boolean {
	for (const row of rows) {
		const rowId = dependencyRowIdentity(row)
		if (seenDependencies.has(rowId)) return true
		seenDependencies.add(rowId)
		if (hasDuplicateDependencyRows(row.dependencies, seenDependencies)) return true
	}

	return false
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

function filenameFromUrl(url: string) {
	try {
		const filename = new URL(url).pathname.split('/').pop()
		return filename ? decodeURIComponent(filename) : 'dependency.jar'
	} catch {
		return 'dependency.jar'
	}
}

function sortedUnique(values: string[]) {
	return [...new Set(values)].sort()
}

const messages = defineMessages({
	anyCompatibleDependency: {
		id: 'project.download.dependency-any-compatible',
		defaultMessage: 'Any compatible',
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
