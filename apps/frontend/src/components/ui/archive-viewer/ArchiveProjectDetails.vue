<template>
	<div class="flex flex-col gap-4 p-3 text-sm">
		<section class="flex flex-col gap-3">
			<div class="flex min-w-0 items-center gap-3">
				<Avatar :src="project.icon_url" :alt="project.name" size="48px" no-shadow />
				<div class="flex min-w-0 flex-col gap-0.5">
					<nuxt-link
						:to="projectUrl"
						class="truncate text-base font-semibold text-contrast hover:underline"
					>
						{{ project.name }}
					</nuxt-link>
					<span class="line-clamp-2 text-secondary">{{ project.summary }}</span>
				</div>
			</div>
			<div class="flex flex-wrap items-center gap-x-3 gap-y-1.5">
				<ProjectStatusBadge
					:status="project.status"
					:style="{ color: `var(--color-${getProjectStatusColor(project.status)})` }"
				/>
				<span
					v-for="type in project.project_types"
					:key="type"
					class="flex items-center gap-1 font-semibold text-secondary"
				>
					<component :is="getProjectTypeIcon(type)" class="size-4 shrink-0" aria-hidden="true" />
					{{ formatMessage(getProjectTypeTitleMessage(type), { count: 1 }) }}
				</span>
			</div>
			<div class="flex flex-wrap gap-2">
				<ButtonLink type="outlined" size="sm" :to="projectUrl">
					<ExternalIcon aria-hidden="true" />
					{{ formatMessage(messages.openProject) }}
				</ButtonLink>
				<ButtonLink type="outlined" size="sm" :to="`/moderation/technical-review/${project.id}`">
					<ShieldIcon aria-hidden="true" />
					{{ formatMessage(messages.techReview) }}
				</ButtonLink>
			</div>
			<dl class="m-0 grid grid-cols-[minmax(6rem,max-content)_minmax(0,1fr)] gap-x-4 gap-y-2">
				<dt class="text-secondary">{{ formatMessage(messages.id) }}</dt>
				<dd class="m-0 min-w-0"><CopyCode :text="project.id" /></dd>
				<dt class="text-secondary">{{ formatMessage(messages.published) }}</dt>
				<dd class="m-0 min-w-0">
					<time v-tooltip="formatDate(project.published)" :datetime="project.published">
						{{ relativeTime(project.published) }}
					</time>
				</dd>
			</dl>
		</section>

		<section class="flex flex-col gap-3 border-0 border-t border-solid border-surface-5 pt-4">
			<h4 class="m-0 text-sm font-semibold text-contrast">{{ formatMessage(messages.version) }}</h4>
			<Combobox
				v-if="versions.length"
				:model-value="version?.id"
				:options="versionOptions"
				:placeholder="formatMessage(messages.selectVersion)"
				:search-placeholder="formatMessage(messages.searchVersions)"
				searchable
				@update:model-value="(id) => id && emit('select-version', id)"
			/>
			<span v-else-if="versionsLoading" class="flex items-center gap-2 text-secondary">
				<SpinnerIcon class="size-4 animate-spin" aria-hidden="true" />
				{{ formatMessage(messages.loadingVersions) }}
			</span>
			<template v-if="version">
				<div class="flex min-w-0 items-center gap-2">
					<VersionChannelIndicator
						v-tooltip="formatMessage(channelMessages[version.version_type])"
						:channel="version.version_type"
						size="sm"
					/>
					<div class="flex min-w-0 flex-col">
						<span class="truncate font-semibold text-contrast">{{ version.version_number }}</span>
						<span v-if="version.name !== version.version_number" class="truncate text-secondary">
							{{ version.name }}
						</span>
					</div>
				</div>
				<div class="flex flex-wrap gap-1.5">
					<TagItem
						v-if="version.status !== 'listed'"
						class="font-semibold"
						:class="versionStatusClass"
					>
						{{ formatMessage(versionStatusMessages[version.status] ?? versionStatusMessages.unknown) }}
					</TagItem>
					<TagTagItem v-for="loader in version.loaders" :key="loader" :tag="loader" />
					<TagItem v-for="gameVersion in shownGameVersions" :key="gameVersion">
						{{ gameVersion }}
					</TagItem>
					<TagItem v-if="hiddenGameVersions.length" v-tooltip="hiddenGameVersions.join(', ')">
						{{ formatMessage(messages.moreGameVersions, { count: hiddenGameVersions.length }) }}
					</TagItem>
				</div>
				<dl class="m-0 grid grid-cols-[minmax(6rem,max-content)_minmax(0,1fr)] gap-x-4 gap-y-2">
					<dt class="text-secondary">{{ formatMessage(messages.id) }}</dt>
					<dd class="m-0 min-w-0"><CopyCode :text="version.id" /></dd>
					<dt class="text-secondary">{{ formatMessage(messages.published) }}</dt>
					<dd class="m-0 min-w-0">
						<time v-tooltip="formatDate(version.date_published)" :datetime="version.date_published">
							{{ relativeTime(version.date_published) }}
						</time>
					</dd>
					<dt class="text-secondary">{{ formatMessage(messages.downloads) }}</dt>
					<dd class="m-0 min-w-0">{{ formatNumber(version.downloads) }}</dd>
				</dl>
			</template>
		</section>

		<section
			v-if="file"
			class="flex flex-col gap-3 border-0 border-t border-solid border-surface-5 pt-4"
		>
			<h4 class="m-0 text-sm font-semibold text-contrast">{{ formatMessage(messages.file) }}</h4>
			<Combobox
				v-if="(version?.files.length ?? 0) > 1"
				:model-value="file.url"
				:options="fileOptions"
				@update:model-value="(url) => url && emit('select-file', url)"
			/>
			<dl class="m-0 grid grid-cols-[minmax(6rem,max-content)_minmax(0,1fr)] gap-x-4 gap-y-2">
				<dt class="text-secondary">{{ formatMessage(messages.name) }}</dt>
				<dd class="m-0 min-w-0 break-all">
					{{ file.filename }}
					<TagItem v-if="file.primary" class="ml-1 inline-flex align-middle">
						{{ formatMessage(messages.primary) }}
					</TagItem>
				</dd>
				<dt class="text-secondary">{{ formatMessage(messages.size) }}</dt>
				<dd class="m-0 min-w-0">{{ formatBytes(file.size) }}</dd>
				<template v-if="file.file_type">
					<dt class="text-secondary">{{ formatMessage(messages.fileType) }}</dt>
					<dd class="m-0 min-w-0">{{ file.file_type }}</dd>
				</template>
				<dt class="text-secondary">{{ formatMessage(messages.sha512) }}</dt>
				<dd class="m-0 min-w-0"><ArchiveHashValue :hash="file.hashes.sha512" /></dd>
				<dt class="text-secondary">{{ formatMessage(messages.hashCheck) }}</dt>
				<dd class="m-0 flex min-w-0 items-center gap-1" :class="hashCheck.class">
					<component :is="hashCheck.icon" class="size-4 shrink-0" aria-hidden="true" />
					{{ hashCheck.label }}
				</dd>
			</dl>
			<div class="flex flex-wrap gap-2">
				<ButtonLink type="outlined" size="sm" :href="file.url" :download="file.filename">
					<DownloadIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.downloadButton) }}
				</ButtonLink>
				<ButtonLink v-if="slicerUrl" type="outlined" size="sm" :href="slicerUrl" target="_blank">
					<ExternalIcon aria-hidden="true" />
					{{ formatMessage(messages.openInSlicer) }}
				</ButtonLink>
			</div>
		</section>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	CheckCircleIcon,
	DownloadIcon,
	ExternalIcon,
	ShieldIcon,
	SpinnerIcon,
	TriangleAlertIcon,
} from '@modrinth/assets'
import {
	ArchiveHashValue,
	Avatar,
	ButtonLink,
	Combobox,
	type ComboboxOption,
	commonMessages,
	CopyCode,
	defineMessages,
	getProjectStatusColor,
	getProjectTypeIcon,
	getProjectTypeTitleMessage,
	injectArchiveViewer,
	ProjectStatusBadge,
	TagItem,
	TagTagItem,
	useFormatBytes,
	useFormatDateTime,
	useFormatNumber,
	useRelativeTime,
	useVIntl,
	VersionChannelIndicator,
} from '@modrinth/ui'
import { type Component, computed } from 'vue'

const props = defineProps<{
	project: Labrinth.Projects.v3.Project
	/** The project's versions, newest first. */
	versions: Labrinth.Versions.v3.Version[]
	versionsLoading?: boolean
	version: Labrinth.Versions.v3.Version | null
	file: Labrinth.Versions.v3.VersionFile | null
	slicerUrl?: string
}>()

const emit = defineEmits<{
	'select-version': [id: string]
	'select-file': [url: string]
}>()

const { formatMessage } = useVIntl()
const formatBytes = useFormatBytes()
const formatNumber = useFormatNumber()
const formatDate = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })
const relativeTime = useRelativeTime()
const archive = injectArchiveViewer()

const messages = defineMessages({
	openProject: { id: 'archive.project-details.open-project', defaultMessage: 'Project page' },
	techReview: { id: 'archive.project-details.tech-review', defaultMessage: 'Tech review' },
	id: { id: 'archive.project-details.id', defaultMessage: 'ID' },
	published: { id: 'archive.project-details.published', defaultMessage: 'Published' },
	version: { id: 'archive.project-details.version', defaultMessage: 'Version' },
	selectVersion: { id: 'archive.select-version', defaultMessage: 'Select a version' },
	searchVersions: { id: 'archive.search-versions', defaultMessage: 'Search versions...' },
	loadingVersions: { id: 'archive.loading-versions', defaultMessage: 'Loading versions...' },
	moreGameVersions: {
		id: 'archive.project-details.more-game-versions',
		defaultMessage: '+{count} more',
	},
	downloads: { id: 'archive.project-details.downloads', defaultMessage: 'Downloads' },
	file: { id: 'archive.project-details.file', defaultMessage: 'File' },
	name: { id: 'archive.project-details.name', defaultMessage: 'Name' },
	primary: { id: 'archive.project-details.primary', defaultMessage: 'Primary' },
	size: { id: 'archive.project-details.size', defaultMessage: 'Size' },
	fileType: { id: 'archive.project-details.file-type', defaultMessage: 'File type' },
	sha512: { id: 'archive.project-details.sha512', defaultMessage: 'Listed SHA-512' },
	hashCheck: { id: 'archive.project-details.hash-check', defaultMessage: 'Download matches' },
	hashPending: {
		id: 'archive.project-details.hash-pending',
		defaultMessage: 'Checked once fully downloaded',
	},
	hashMatch: { id: 'archive.project-details.hash-match', defaultMessage: 'Matches listed hash' },
	hashMismatch: {
		id: 'archive.project-details.hash-mismatch',
		defaultMessage: 'Does not match listed hash',
	},
	openInSlicer: { id: 'archive.open-in-slicer', defaultMessage: 'Open in Slicer' },
})

const channelMessages = defineMessages({
	release: { id: 'archive.project-details.channel.release', defaultMessage: 'Release' },
	beta: { id: 'archive.project-details.channel.beta', defaultMessage: 'Beta' },
	alpha: { id: 'archive.project-details.channel.alpha', defaultMessage: 'Alpha' },
})

const versionStatusMessages = defineMessages({
	listed: { id: 'archive.project-details.version-status.listed', defaultMessage: 'Listed' },
	archived: { id: 'archive.project-details.version-status.archived', defaultMessage: 'Archived' },
	draft: { id: 'archive.project-details.version-status.draft', defaultMessage: 'Draft' },
	unlisted: { id: 'archive.project-details.version-status.unlisted', defaultMessage: 'Unlisted' },
	scheduled: {
		id: 'archive.project-details.version-status.scheduled',
		defaultMessage: 'Scheduled',
	},
	unknown: { id: 'archive.project-details.version-status.unknown', defaultMessage: 'Unknown' },
})

/** Game versions shown before the rest collapse into a count. */
const SHOWN_GAME_VERSIONS = 6

const projectUrl = computed(() => `/project/${props.project.slug ?? props.project.id}`)

const shownGameVersions = computed(
	() => props.version?.game_versions.slice(0, SHOWN_GAME_VERSIONS) ?? [],
)
const hiddenGameVersions = computed(
	() => props.version?.game_versions.slice(SHOWN_GAME_VERSIONS) ?? [],
)

const versionStatusClass = computed(() =>
	props.version?.status === 'archived' || props.version?.status === 'unknown'
		? '!text-secondary'
		: '!border-orange-highlight !bg-orange-highlight !text-orange',
)

const versionOptions = computed<ComboboxOption<string>[]>(() =>
	props.versions.map((version) => ({
		value: version.id,
		label: version.version_number,
		subLabel: version.name !== version.version_number ? version.name : undefined,
		searchTerms: [version.name, version.version_number],
	})),
)

const fileOptions = computed<ComboboxOption<string>[]>(() =>
	(props.version?.files ?? []).map((file) => ({
		value: file.url,
		label: file.filename,
		subLabel: file.primary ? formatMessage(messages.primary) : undefined,
	})),
)

/** The downloaded archive's hash against the one the version lists for the file. */
const hashCheck = computed<{ label: string; icon: Component; class: string }>(() => {
	const listed = props.file?.hashes.sha512
	const actual = archive.sha512Hash.value
	if (!actual || !listed) {
		return {
			label: formatMessage(messages.hashPending),
			icon: SpinnerIcon,
			class: 'text-secondary [&>svg]:animate-spin',
		}
	}
	return actual === listed
		? { label: formatMessage(messages.hashMatch), icon: CheckCircleIcon, class: 'text-green' }
		: {
				label: formatMessage(messages.hashMismatch),
				icon: TriangleAlertIcon,
				class: 'font-semibold text-red',
			}
})
</script>
