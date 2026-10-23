<template>
	<div class="h-dvh w-full bg-surface-1">
		<ClientOnly>
			<ArchiveViewerLayout
				:source="source"
				:path="archivePath"
				:storage-key="embedded ? null : 'archive-viewer'"
				:hidden-regions="hiddenRegions"
				popout-url="/project-review-popout.html"
				@update:path="setArchivePath"
			>
				<template #toolbar>
					<Button v-if="!embedded" type="quiet" size="xs" @click="goBack">
						<LeftArrowIcon aria-hidden="true" />
						{{ formatMessage(messages.goBack) }}
					</Button>
					<div v-if="project" class="flex min-w-0 items-center gap-2">
						<Avatar :src="project.icon_url" :alt="project.name" size="20px" no-shadow />
						<span class="truncate text-sm font-semibold text-contrast">{{ project.name }}</span>
						<span v-if="selectedVersion" class="truncate text-sm text-secondary">
							{{ selectedVersion.version_number }}
						</span>
					</div>
					<template v-else-if="source">
						<TagItem class="shrink-0">{{ formatMessage(messages.external) }}</TagItem>
						<span class="truncate text-sm font-semibold text-contrast">{{ source.name }}</span>
						<ButtonLink type="quiet" size="xs" :href="source.url" :download="source.name">
							<DownloadIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.downloadButton) }}
						</ButtonLink>
					</template>
				</template>
				<template v-if="project || source" #project-details>
					<ArchiveProjectDetails
						v-if="project"
						:project="project"
						:versions="sortedVersions"
						:versions-loading="versionsLoading"
						:version="selectedVersion"
						:file="selectedFile"
						@select-version="selectVersion"
						@select-file="selectFile"
					/>
					<ArchiveExternalDetails v-else-if="source" :source="source" />
				</template>
				<template #empty>
					<Admonition
						v-if="isError"
						type="critical"
						:header="formatMessage(messages.loadFailed)"
						:body="formatMessage(messages.loadFailedDescription)"
					/>
					<Admonition
						v-else-if="!isLoading && !source"
						type="info"
						:header="formatMessage(messages.nothingToShow)"
						:body="formatMessage(messages.nothingToShowDescription)"
					/>
				</template>
			</ArchiveViewerLayout>
		</ClientOnly>
	</div>
</template>

<script setup lang="ts">
import { DownloadIcon, LeftArrowIcon } from '@modrinth/assets'
import {
	Admonition,
	type ArchiveSource,
	ArchiveViewerLayout,
	Avatar,
	Button,
	ButtonLink,
	commonMessages,
	defineMessages,
	injectModrinthClient,
	TagItem,
	useVIntl,
	WORKBENCH_REGIONS,
	type WorkbenchRegion,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import ArchiveExternalDetails from '~/components/ui/archive-viewer/ArchiveExternalDetails.vue'
import ArchiveProjectDetails from '~/components/ui/archive-viewer/ArchiveProjectDetails.vue'
import {
	archiveNameFrom,
	archivePathFrom,
	archiveSourceFrom,
	isModrinthCdnUrl,
	primaryFile,
} from '~/helpers/archive-viewer'

definePageMeta({
	layout: 'empty',
	middleware: ['auth', 'staff'],
})

const { formatMessage } = useVIntl()
const client = injectModrinthClient()
const route = useRoute()
const router = useRouter()

const messages = defineMessages({
	title: {
		id: 'archive.title',
		defaultMessage: 'Archive viewer',
	},
	goBack: {
		id: 'archive.go-back',
		defaultMessage: 'Go back',
	},
	external: {
		id: 'archive.external',
		defaultMessage: 'External',
	},
	loadFailed: {
		id: 'archive.load-failed',
		defaultMessage: 'Unable to load the project',
	},
	loadFailedDescription: {
		id: 'archive.load-failed-description',
		defaultMessage: 'The project or its versions could not be fetched.',
	},
	nothingToShow: {
		id: 'archive.nothing-to-show',
		defaultMessage: 'No archive to show',
	},
	nothingToShowDescription: {
		id: 'archive.nothing-to-show-description',
		defaultMessage: 'Open a project with version files, or a file link from the Modrinth CDN.',
	},
})

function queryString(value: unknown) {
	const first = Array.isArray(value) ? value[0] : value
	return typeof first === 'string' && first.length > 0 ? first : null
}

const projectId = computed(() => queryString(route.query.project))

/** A one-off view (e.g. embedded elsewhere) that leaves the user's own panel layout untouched. */
const embedded = computed(() => queryString(route.query.embed) != null)

/** Regions to start hidden, e.g. `?hide=left,bottom`. */
const hiddenRegions = computed<WorkbenchRegion[]>(() =>
	(queryString(route.query.hide)?.split(',') ?? []).filter((region): region is WorkbenchRegion =>
		(WORKBENCH_REGIONS as readonly string[]).includes(region),
	),
)

const projectQuery = useQuery({
	queryKey: computed(() => ['project', 'v3', projectId.value]),
	queryFn: () => client.labrinth.projects_v3.get(projectId.value!),
	enabled: computed(() => !!projectId.value),
})

const versionsQuery = useQuery({
	queryKey: computed(() => ['project', projectId.value, 'versions', 'v3']),
	queryFn: () =>
		client.labrinth.versions_v3.getProjectVersions(projectId.value!, {
			include_changelog: false,
			apiVersion: 3,
		}),
	enabled: computed(() => !!projectId.value),
})

const project = computed(() => projectQuery.data.value ?? null)
const versionsLoading = computed(() => versionsQuery.isFetching.value)
const isLoading = computed(
	() => !!projectId.value && (projectQuery.isPending.value || versionsQuery.isPending.value),
)
const isError = computed(() => projectQuery.isError.value || versionsQuery.isError.value)

const sortedVersions = computed(() =>
	[...(versionsQuery.data.value ?? [])].sort(
		(a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime(),
	),
)

const requestedUrl = computed(() => {
	const url = queryString(route.query.url)
	return url && isModrinthCdnUrl(url) ? url : null
})

/**
 * The version from the query, else the one holding the requested file, else the latest. Only
 * staff reach this page, so every version (including drafts and unlisted ones) is available.
 */
const selectedVersion = computed(() => {
	const id = queryString(route.query.version)
	const url = requestedUrl.value
	return (
		(id ? sortedVersions.value.find((version) => version.id === id) : undefined) ??
		(url
			? sortedVersions.value.find((version) => version.files.some((file) => file.url === url))
			: undefined) ??
		(url ? undefined : sortedVersions.value[0]) ??
		null
	)
})

const selectedFile = computed(() => {
	const version = selectedVersion.value
	if (!version) return null
	const url = requestedUrl.value
	return (
		(url ? version.files.find((file) => file.url === url) : undefined) ??
		primaryFile(version) ??
		null
	)
})

/** The selected version's file, or a requested CDN link no version of the project holds. */
const source = computed<ArchiveSource | null>(() => {
	if (selectedFile.value) return archiveSourceFrom(selectedFile.value)
	const url = requestedUrl.value
	return url ? { url, name: archiveNameFrom(url) } : null
})

useHead({
	title: () => {
		const name = project.value?.name ?? source.value?.name
		return name ? `${formatMessage(messages.title)} - ${name}` : formatMessage(messages.title)
	},
})

/**
 * The entry open within the archive. Slicer links prefix it with the archive's name, which is
 * dropped so they can be opened here as is.
 */
const archivePath = computed(() => {
	const name = source.value?.name
	return name ? archivePathFrom(queryString(route.query.file), name) : queryString(route.query.file)
})

function replaceQuery(query: Record<string, string | undefined>) {
	router.replace({ query: { ...route.query, ...query } })
}

/** Back to where the viewer was opened from, or to the project when opened directly. */
function goBack() {
	if (window.history.state?.back) {
		router.back()
		return
	}
	navigateTo(project.value ? `/project/${project.value.slug ?? project.value.id}` : '/')
}

function selectVersion(id: string | undefined) {
	replaceQuery({ version: id, url: undefined, file: undefined })
}

function selectFile(url: string | undefined) {
	replaceQuery({ version: selectedVersion.value?.id, url, file: undefined })
}

function setArchivePath(path: string) {
	if (path === (archivePath.value ?? '')) return
	replaceQuery({ file: path || undefined })
}
</script>
