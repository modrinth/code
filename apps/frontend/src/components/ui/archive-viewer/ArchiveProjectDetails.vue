<template>
	<ArchiveDetailsSection
		v-if="!hideProject"
		:kind="formatMessage(messages.project)"
		:title="project.name"
		details-key="project"
	>
		<template #icon>
			<Avatar :src="project.icon_url" alt="" size="32px" no-shadow class="shrink-0" />
		</template>
		<template #actions>
			<div class="flex shrink-0 items-center gap-1">
				<Tooltip class="flex shrink-0">
					<template #popper>
						<ProjectStatusBadge :status="project.status" :style="statusColor" />
					</template>
					<component
						:is="getProjectStatusIcon(project.status)"
						class="size-4"
						:style="statusColor"
						aria-hidden="true"
					/>
				</Tooltip>
				<component
					:is="getProjectTypeIcon(type)"
					v-for="type in project.project_types"
					:key="type"
					v-tooltip="formatMessage(getProjectTypeTitleMessage(type), { count: 1 })"
					class="size-4 text-secondary"
					role="img"
					:aria-label="formatMessage(getProjectTypeTitleMessage(type), { count: 1 })"
				/>
				<ButtonLink
					v-tooltip="formatMessage(messages.openProject)"
					type="quiet"
					size="xs"
					:to="projectUrl"
					:aria-label="formatMessage(messages.openProject)"
				>
					<ExternalIcon aria-hidden="true" />
				</ButtonLink>
				<ButtonLink
					v-tooltip="formatMessage(messages.techReview)"
					type="quiet"
					size="xs"
					:to="`/moderation/technical-review/${project.id}`"
					:aria-label="formatMessage(messages.techReview)"
				>
					<ShieldIcon aria-hidden="true" />
				</ButtonLink>
			</div>
		</template>

		<p v-if="project.summary" class="m-0 text-sm text-secondary">{{ project.summary }}</p>

		<template #details>
			<div class="flex flex-wrap gap-1.5">
				<ArchiveDetailChip
					:label="formatMessage(messages.projectId)"
					:value="project.id"
					:icon="HashIcon"
					mono
				/>
				<ArchiveDetailChip
					:label="formatMessage(messages.projectPublished)"
					:value="formatShortDate(project.published)"
					:tooltip="
						formatMessage(messages.publishedOn, { date: formatFullDate(project.published) })
					"
					:copy-text="formatFullDate(project.published)"
					:icon="CalendarIcon"
				/>
				<ArchiveDetailChip
					:label="formatMessage(messages.projectDownloads)"
					:value="formatNumber(project.downloads)"
					:icon="DownloadIcon"
					:copyable="false"
				/>
			</div>
		</template>
	</ArchiveDetailsSection>

	<ArchiveDetailsSection
		:kind="formatMessage(messages.version)"
		:title="
			version?.name ??
			formatMessage(versionsLoading ? messages.loadingVersions : messages.selectVersion)
		"
		details-key="version"
	>
		<template #icon>
			<VersionChannelIndicator
				v-if="version"
				v-tooltip="formatMessage(channelMessages[version.version_type])"
				:channel="version.version_type"
				size="sm"
				class="shrink-0"
			/>
			<SpinnerIcon
				v-else-if="versionsLoading"
				class="size-5 shrink-0 animate-spin text-secondary"
				aria-hidden="true"
			/>
		</template>
		<template v-if="file" #actions>
			<div class="flex shrink-0 items-center gap-1">
				<ButtonLink
					v-tooltip="formatMessage(messages.download)"
					type="quiet"
					size="xs"
					:href="file.url"
					:download="file.filename"
					:aria-label="formatMessage(messages.download)"
				>
					<DownloadIcon aria-hidden="true" />
				</ButtonLink>
			</div>
		</template>

		<div v-if="versions.length" class="flex flex-col gap-2 text-sm">
			<Button
				type="outlined"
				size="sm"
				class="w-full"
				:aria-label="formatMessage(messages.changeVersion)"
				@click="(event: MouseEvent) => versionModal?.show(event)"
			>
				<span class="flex min-w-0 flex-1 items-center gap-2 text-left">
					<span class="truncate font-semibold text-contrast">
						{{ version?.version_number ?? formatMessage(messages.selectVersion) }}
					</span>
					<span v-if="version" class="truncate text-xs font-normal text-secondary">
						{{ formatShortDate(version.date_published) }}
					</span>
				</span>
				<GenericListIcon class="shrink-0" aria-hidden="true" />
			</Button>
			<Combobox
				v-if="(version?.files.length ?? 0) > 1"
				:model-value="file?.url"
				:options="fileOptions"
				trigger-size="sm"
				@update:model-value="(url) => url && emit('select-file', url)"
			/>
			<div v-if="version" class="flex flex-wrap gap-1">
				<TagItem
					v-if="version.status !== 'listed'"
					class="font-semibold"
					:class="versionStatusClass"
				>
					{{
						formatMessage(versionStatusMessages[version.status] ?? versionStatusMessages.unknown)
					}}
				</TagItem>
				<TagTagItem v-for="loader in version.loaders" :key="loader" :tag="loader" />
				<TagItem v-for="gameVersion in shownGameVersions" :key="gameVersion">
					{{ gameVersion }}
				</TagItem>
				<TagItem v-if="hiddenGameVersions.length" v-tooltip="hiddenGameVersions.join(', ')">
					{{ formatMessage(messages.moreGameVersions, { count: hiddenGameVersions.length }) }}
				</TagItem>
			</div>
		</div>

		<template v-if="version" #details>
			<div class="flex flex-wrap gap-1.5">
				<ArchiveDetailChip
					:label="formatMessage(messages.versionId)"
					:value="version.id"
					:icon="HashIcon"
					mono
				/>
				<ArchiveDetailChip
					:label="formatMessage(messages.versionPublished)"
					:value="formatShortDate(version.date_published)"
					:tooltip="
						formatMessage(messages.publishedOn, { date: formatFullDate(version.date_published) })
					"
					:copy-text="formatFullDate(version.date_published)"
					:icon="CalendarIcon"
				/>
				<ArchiveDetailChip
					:label="formatMessage(messages.versionDownloads)"
					:value="formatNumber(version.downloads)"
					:icon="DownloadIcon"
					:copyable="false"
				/>
				<ArchiveDetailChip
					v-if="file?.file_type"
					:label="formatMessage(messages.fileType)"
					:value="file.file_type"
					:copyable="false"
				/>
			</div>
		</template>
	</ArchiveDetailsSection>

	<ArchiveVersionModal
		ref="versionModal"
		:project="project"
		:versions="versions"
		:loading="versionsLoading"
		:current-version-id="version?.id"
		@select="(id) => emit('select-version', id)"
	/>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	CalendarIcon,
	DownloadIcon,
	ExternalIcon,
	GenericListIcon,
	HashIcon,
	ShieldIcon,
	SpinnerIcon,
} from '@modrinth/assets'
import {
	ArchiveDetailChip,
	ArchiveDetailsSection,
	Avatar,
	Button,
	ButtonLink,
	Combobox,
	type ComboboxOption,
	defineMessages,
	getProjectStatusColor,
	getProjectStatusIcon,
	getProjectTypeIcon,
	getProjectTypeTitleMessage,
	ProjectStatusBadge,
	TagItem,
	TagTagItem,
	Tooltip,
	useFormatDateTime,
	useFormatNumber,
	useVIntl,
	VersionChannelIndicator,
} from '@modrinth/ui'
import { computed, useTemplateRef } from 'vue'

import ArchiveVersionModal from './ArchiveVersionModal.vue'
import { channelMessages, versionStatusMessages } from './messages'

/**
 * The project an archive was published by, at the top of the archive viewer's details, then the
 * version of it that's open (picked from a modal, along with its file) and its tags. Their IDs, dates and
 * download counts are behind each one's details, and the file itself (size, hashes, whether it
 * matches its listed hash) is described by the archive's own details below.
 */
const props = defineProps<{
	project: Labrinth.Projects.v3.Project
	/** The project's versions, newest first. */
	versions: Labrinth.Versions.v3.Version[]
	versionsLoading?: boolean
	version: Labrinth.Versions.v3.Version | null
	file: Labrinth.Versions.v3.VersionFile | null
	/** Leaves out the project, for hosts showing it already, keeping the version and its picker. */
	hideProject?: boolean
}>()

const emit = defineEmits<{
	'select-version': [id: string]
	'select-file': [url: string]
}>()

const { formatMessage } = useVIntl()
const formatNumber = useFormatNumber()
const formatFullDate = useFormatDateTime({ dateStyle: 'full', timeStyle: 'long' })
const formatShortDate = useFormatDateTime({ dateStyle: 'medium' })

const messages = defineMessages({
	project: { id: 'archive.project-details.project', defaultMessage: 'Project' },
	version: { id: 'archive.project-details.version', defaultMessage: 'Version' },
	openProject: { id: 'archive.project-details.open-project', defaultMessage: 'Project page' },
	techReview: { id: 'archive.project-details.tech-review', defaultMessage: 'Tech review' },
	download: { id: 'archive.project-details.download', defaultMessage: 'Download file' },
	selectVersion: { id: 'archive.select-version', defaultMessage: 'Select a version' },
	changeVersion: { id: 'archive.project-details.change-version', defaultMessage: 'Change version' },
	loadingVersions: { id: 'archive.loading-versions', defaultMessage: 'Loading versions...' },
	moreGameVersions: {
		id: 'archive.project-details.more-game-versions',
		defaultMessage: '+{count} more',
	},
	primary: { id: 'archive.project-details.primary', defaultMessage: 'Primary' },
	projectId: { id: 'archive.project-details.project-id', defaultMessage: 'Project ID' },
	projectPublished: {
		id: 'archive.project-details.project-published',
		defaultMessage: 'Project published',
	},
	versionId: { id: 'archive.project-details.version-id', defaultMessage: 'Version ID' },
	versionPublished: {
		id: 'archive.project-details.version-published',
		defaultMessage: 'Version published',
	},
	projectDownloads: {
		id: 'archive.project-details.project-downloads',
		defaultMessage: 'Project downloads',
	},
	versionDownloads: {
		id: 'archive.project-details.version-downloads',
		defaultMessage: 'Version downloads',
	},
	publishedOn: { id: 'archive.project-details.published-on', defaultMessage: 'Published {date}' },
	fileType: { id: 'archive.project-details.file-type', defaultMessage: 'File type' },
})

/** Game versions shown before the rest collapse into a count. */
const SHOWN_GAME_VERSIONS = 4

const projectUrl = computed(() => `/project/${props.project.slug ?? props.project.id}`)

const statusColor = computed(() => ({
	color: `var(--color-${getProjectStatusColor(props.project.status)})`,
}))

const versionModal = useTemplateRef<InstanceType<typeof ArchiveVersionModal>>('versionModal')

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

const fileOptions = computed<ComboboxOption<string>[]>(() =>
	(props.version?.files ?? []).map((entry) => ({
		value: entry.url,
		label: entry.filename,
		subLabel: entry.primary ? formatMessage(messages.primary) : undefined,
	})),
)
</script>
