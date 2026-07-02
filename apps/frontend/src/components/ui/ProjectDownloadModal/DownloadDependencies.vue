<template>
	<div v-if="dependencyRows.length > 0" class="flex flex-col gap-2">
		<h3 v-if="showTitle" class="m-0 text-sm font-bold text-contrast">
			{{ formatMessage(messages.dependenciesTitle) }}
		</h3>
		<div class="flex flex-col gap-2">
			<DownloadDependency
				v-for="dependency in dependencyRows"
				:key="dependency.key"
				:dependency="dependency"
				@download="emit('download')"
			/>
		</div>
	</div>
</template>

<script setup>
import { defineMessages, injectModrinthClient, useVIntl } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import DownloadDependency from './DownloadDependency.vue'

defineOptions({
	name: 'DownloadDependencies',
})

const props = defineProps({
	dependencies: {
		type: Array,
		default: null,
	},
	project: {
		type: Object,
		default: null,
	},
	selectedVersion: {
		type: Object,
		default: null,
	},
	currentGameVersion: {
		type: [String, Boolean],
		default: null,
	},
	currentPlatform: {
		type: [String, Boolean],
		default: null,
	},
	downloadReason: {
		type: String,
		default: 'standalone',
	},
	showTitle: {
		type: Boolean,
		default: true,
	},
})

const emit = defineEmits(['download'])
const client = injectModrinthClient()
const { createProjectDownloadUrl } = useCdnDownloadContext()
const { formatMessage } = useVIntl()

const shouldResolveDependencies = computed(
	() => !props.dependencies && !!props.project && !!props.selectedVersion,
)

const dependencyResolutionPreferences = computed(() => ({
	game_versions: props.selectedVersion?.game_versions || [],
	loaders: props.selectedVersion?.loaders || [],
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
			project_id: props.project.id,
			version_id: props.selectedVersion.id,
			content_type: resolveContentType(props.project.project_type),
			selected: dependencyResolutionPreferences.value,
			target: dependencyResolutionPreferences.value,
		}),
	enabled: shouldResolveDependencies,
})

const dependencyVersionIds = computed(() => {
	return [
		...new Set(
			[
				...(dependencyResolution.value?.dependencies || []),
				...(dependencyResolution.value?.skipped || []),
			]
				.map((dependency) => dependency.version_id)
				.filter(Boolean),
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
	const map = new Map()
	for (const version of dependencyVersions.value || []) {
		if (!version) continue
		map.set(version.id, version)
	}
	return map
})

const dependencyProjectIds = computed(() => {
	return [
		...new Set(
			[
				...(dependencyResolution.value?.dependencies || []),
				...(dependencyResolution.value?.skipped || []),
			]
				.map((dependency) => dependency.project_id)
				.filter(Boolean),
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
	const map = new Map()
	for (const project of dependencyProjects.value || []) {
		map.set(project.id, project)
	}
	return map
})

const dependenciesByParentVersionId = computed(() => {
	const map = new Map()

	for (const dependency of [
		...(dependencyResolution.value?.dependencies || []),
		...(dependencyResolution.value?.skipped || []),
	]) {
		if (!dependency.dependent_on_version_id) continue

		const dependencies = map.get(dependency.dependent_on_version_id) || []
		dependencies.push(dependency)
		map.set(dependency.dependent_on_version_id, dependencies)
	}

	return map
})

const resolvedDependencyRows = computed(() => {
	const primaryVersionId =
		dependencyResolution.value?.primary.version_id || props.selectedVersion?.id
	if (!primaryVersionId) return []

	const dependencies = dependenciesByParentVersionId.value.get(primaryVersionId) || []

	return dependencies.map((dependency) => createDependencyRow(dependency))
})

const dependencyRows = computed(() => props.dependencies || resolvedDependencyRows.value)

function primaryFileForVersion(version) {
	return version?.files?.find((file) => file.primary) || version?.files?.[0]
}

function createDependencyRow(dependency) {
	const version = dependencyVersionById.value.get(dependency.version_id)
	const project = dependencyProjectById.value.get(dependency.project_id)
	const primaryFile = primaryFileForVersion(version)
	const unavailableTooltip = dependency.reason
		? skippedReasonLabel(dependency.reason)
		: formatMessage(messages.unavailableDependency)
	const name =
		project?.title ||
		version?.name ||
		version?.version_number ||
		dependency.version_id ||
		dependency.project_id ||
		'Dependency'

	return {
		key: `${dependency.project_id}-${dependency.version_id ?? 'unresolved'}-${dependency.reason ?? 'resolved'}`,
		name,
		icon: project?.icon_url,
		projectHref: project ? `/${project.project_type}/${project.slug || project.id}` : undefined,
		downloadHref: dependency.reason || !primaryFile ? undefined : getDownloadUrl(primaryFile.url),
		filename: primaryFile?.filename,
		typeLabel: 'Required',
		unavailableTooltip,
		dependencies: (
			(dependency.version_id && dependenciesByParentVersionId.value.get(dependency.version_id)) ||
			[]
		).map((subDependency) => createDependencyRow(subDependency)),
	}
}

function skippedReasonLabel(reason) {
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

function resolveContentType(projectType) {
	return ['mod', 'plugin', 'datapack', 'resourcepack', 'shader', 'modpack'].includes(projectType)
		? projectType
		: 'mod'
}

function getDownloadUrl(url) {
	return createProjectDownloadUrl(url, {
		reason: props.downloadReason,
		gameVersion: props.currentGameVersion ?? undefined,
		loader: props.currentPlatform ?? undefined,
	})
}

const messages = defineMessages({
	dependenciesTitle: {
		id: 'project.download.dependencies-title',
		defaultMessage: 'Dependencies',
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
})
</script>
