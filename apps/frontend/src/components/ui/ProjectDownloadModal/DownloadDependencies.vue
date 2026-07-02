<template>
	<div v-if="dependencyRows.length > 0" class="flex flex-col gap-2">
		<h3 v-if="showTitle" class="m-0 text-sm font-bold text-contrast">
			{{ formatMessage(messages.dependenciesTitle) }}
		</h3>
		<div class="flex flex-col gap-2">
			<div v-for="dependency in dependencyRows" :key="dependency.key" class="flex flex-col gap-1.5">
				<div
					class="grid min-h-9 grid-cols-[minmax(0,1fr)_min-content] items-center gap-2 rounded-xl bg-button-bg px-3 py-2 text-primary"
				>
					<span class="flex min-w-0 items-center gap-2">
						<Avatar
							v-if="dependency.icon"
							:src="dependency.icon"
							:alt="dependency.name"
							size="20px"
						/>
						<PackageIcon v-else aria-hidden="true" class="size-5 flex-shrink-0 text-secondary" />
						<a
							v-if="dependency.projectHref"
							:href="dependency.projectHref"
							target="_blank"
							rel="noopener noreferrer"
							class="min-w-0 truncate font-semibold text-contrast no-underline hover:underline"
						>
							{{ dependency.name }}
						</a>
						<span v-else class="min-w-0 truncate font-semibold text-contrast">
							{{ dependency.name }}
						</span>
						<TagItem class="shrink-0 border !border-solid border-surface-5">
							{{ dependency.typeLabel }}
						</TagItem>
					</span>
					<ButtonStyled v-if="dependency.downloadHref" circular type="transparent">
						<a
							v-tooltip="'Download'"
							:href="dependency.downloadHref"
							:download="dependency.filename"
							:aria-label="`Download ${dependency.name}`"
							@click="emit('download')"
						>
							<DownloadIcon aria-hidden="true" class="size-5 text-secondary" />
						</a>
					</ButtonStyled>
				</div>
				<div
					v-if="dependency.subDependencies.length > 0"
					class="grid grid-cols-[1.5rem_minmax(0,1fr)] items-start gap-1 pl-5"
				>
					<RightArrowIcon aria-hidden="true" class="mt-2.5 size-4 text-secondary" />
					<DownloadDependencies
						:dependencies="dependency.subDependencies"
						:show-title="false"
						@download="emit('download')"
					/>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup>
import { DownloadIcon, PackageIcon, RightArrowIcon } from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	defineMessages,
	injectModrinthClient,
	TagItem,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

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

const dependencyResolutionPreferences = computed(() => ({
	game_versions: props.currentGameVersion ? [props.currentGameVersion] : [],
	loaders: props.currentPlatform ? [props.currentPlatform] : [],
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
	enabled: computed(() => !!props.project && !!props.selectedVersion),
})

const dependencyVersionIds = computed(() => {
	return [
		...new Set(
			(dependencyResolution.value?.dependencies || []).map((dependency) => dependency.version_id),
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
	enabled: computed(() => dependencyVersionIds.value.length > 0),
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
			(dependencyResolution.value?.dependencies || []).map((dependency) => dependency.project_id),
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
	enabled: computed(() => dependencyProjectIds.value.length > 0),
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

	for (const dependency of dependencyResolution.value?.dependencies || []) {
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

	return (dependenciesByParentVersionId.value.get(primaryVersionId) || []).map((dependency) =>
		createDependencyRow(dependency),
	)
})

const dependencyRows = computed(() => props.dependencies || resolvedDependencyRows.value)

function primaryFileForVersion(version) {
	return version?.files?.find((file) => file.primary) || version?.files?.[0]
}

function createDependencyRow(dependency) {
	const version = dependencyVersionById.value.get(dependency.version_id)
	const project = dependencyProjectById.value.get(dependency.project_id)
	const primaryFile = primaryFileForVersion(version)
	const name =
		project?.title ||
		version?.name ||
		version?.version_number ||
		dependency.version_id ||
		dependency.project_id ||
		'Dependency'

	return {
		key: `${dependency.project_id}-${dependency.version_id}`,
		name,
		icon: project?.icon_url,
		projectHref: project ? `/${project.project_type}/${project.slug || project.id}` : undefined,
		downloadHref: primaryFile ? getDownloadUrl(primaryFile.url) : undefined,
		filename: primaryFile?.filename,
		typeLabel: 'Required',
		subDependencies: (dependenciesByParentVersionId.value.get(dependency.version_id) || []).map(
			(subDependency) => createDependencyRow(subDependency),
		),
	}
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
})
</script>
