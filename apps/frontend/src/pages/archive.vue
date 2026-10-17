<template>
	<div class="h-dvh w-full bg-surface-1">
		<ClientOnly>
			<ArchiveViewerLayout
				:source="source"
				:path="archivePath"
				:storage-key="embedded ? null : 'archive-viewer'"
				:hidden-regions="hiddenRegions"
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
						<span class="truncate text-sm font-semibold text-contrast">{{ source.name }}</span>
						<ButtonLink type="quiet" size="xs" :href="source.url" :download="source.name">
							<DownloadIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.downloadButton) }}
						</ButtonLink>
						<ButtonLink type="quiet" size="xs" :href="slicerUrl" target="_blank">
							<ExternalIcon aria-hidden="true" />
							{{ formatMessage(messages.openInSlicer) }}
						</ButtonLink>
					</template>
				</template>
				<template v-if="project" #project-details>
					<ArchiveProjectDetails
						:project="project"
						:versions="sortedVersions"
						:versions-loading="versionsLoading"
						:version="selectedVersion"
						:file="selectedFile"
						:slicer-url="slicerUrl"
						@select-version="selectVersion"
						@select-file="selectFile"
					/>
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
import type { Labrinth } from '@modrinth/api-client'
import { DownloadIcon, ExternalIcon, LeftArrowIcon } from '@modrinth/assets'
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
	useVIntl,
	WORKBENCH_REGIONS,
	type WorkbenchRegion,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import ArchiveProjectDetails from '~/components/ui/archive-viewer/ArchiveProjectDetails.vue'

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
	openInSlicer: {
		id: 'archive.open-in-slicer',
		defaultMessage: 'Open in Slicer',
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
		defaultMessage:
			'Open a project with version files, or a file link from the Modrinth CDN.',
	},
})

function queryString(value: unknown) {
	const first = Array.isArray(value) ? value[0] : value
	return typeof first === 'string' && first.length > 0 ? first : null
}

/** Only Modrinth's CDN is fetched from, so a crafted link can't point the viewer elsewhere. */
function isModrinthCdnUrl(url: string) {
	try {
		const { protocol, hostname } = new URL(url)
		return (
			protocol === 'https:' && (hostname === 'modrinth.com' || hostname.endsWith('.modrinth.com'))
		)
	} catch {
		return false
	}
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

useHead({
	title: () =>
		project.value
			? `${formatMessage(messages.title)} - ${project.value.name}`
			: formatMessage(messages.title),
})

function primaryFile(version: Labrinth.Versions.v3.Version) {
	return version.files.find((file) => file.primary) ?? version.files[0]
}

function sourceFrom(file: Labrinth.Versions.v3.VersionFile): ArchiveSource {
	return { url: file.url, name: file.filename, size: file.size, sha512Hash: file.hashes.sha512 }
}

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
	return (url ? version.files.find((file) => file.url === url) : undefined) ?? primaryFile(version) ?? null
})

/** The selected version's file, or a requested CDN link no version of the project holds. */
const source = computed<ArchiveSource | null>(() => {
	if (selectedFile.value) return sourceFrom(selectedFile.value)
	const url = requestedUrl.value
	return url ? { url, name: decodeURIComponent(url.split('/').pop() ?? url) } : null
})

/**
 * The entry open within the archive. Slicer links prefix it with the archive's name, which is
 * dropped so they can be opened here as is.
 */
const archivePath = computed(() => {
	const file = queryString(route.query.file)
	const name = source.value?.name
	if (!file) return null
	return name && file.startsWith(`${name}/`) ? file.slice(name.length + 1) : file
})

const slicerUrl = computed(() => {
	if (!source.value) return undefined
	const params = new URLSearchParams({ url: source.value.url })
	if (archivePath.value) params.set('file', `${source.value.name}/${archivePath.value}`)
	return `https://slicer.run/?${params}`
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
