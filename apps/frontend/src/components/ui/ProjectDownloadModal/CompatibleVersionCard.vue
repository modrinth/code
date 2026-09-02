<template>
	<div
		:role="selectable ? 'radio' : undefined"
		:aria-checked="selectable ? selected : undefined"
		:tabindex="selectable ? 0 : undefined"
		class="grid items-center gap-3 rounded-2xl border border-solid px-3 py-3 transition-all"
		:class="{
			'grid-cols-[min-content_minmax(0,1fr)_min-content]': selectable,
			'grid-cols-[minmax(0,1fr)_min-content]': !selectable,
			'cursor-pointer border-brand bg-surface-4 text-contrast': selectable && selected,
			'cursor-pointer border-surface-5 bg-surface-4 hover:brightness-[115%]':
				selectable && !selected,
			'border-transparent bg-surface-2': !selectable,
		}"
		@click="select"
		@keydown.enter.self.prevent="select"
		@keydown.space.self.prevent="select"
	>
		<template v-if="selectable">
			<RadioButtonCheckedIcon v-if="selected" aria-hidden="true" class="size-5 text-brand" />
			<RadioButtonIcon v-else aria-hidden="true" class="size-5 text-secondary" />
		</template>
		<div class="flex min-w-0 flex-col gap-1">
			<div class="flex min-w-0 items-center gap-2">
				<nuxt-link
					v-tooltip="truncatedTooltip(versionNumberRef, version.version_number)"
					:to="`/${project.project_type}/${project.slug || project.id}/version/${version.id}`"
					target="_blank"
					rel="noopener noreferrer"
					class="block min-w-0 text-contrast no-underline hover:underline"
				>
					<span ref="versionNumberRef" class="block truncate font-semibold">
						{{ version.version_number }}
					</span>
				</nuxt-link>
				<VersionChannelTag :channel="version.version_type" class="relative -top-px !py-1" />
			</div>
			<div class="flex min-w-0 items-center gap-1.5 text-sm text-secondary">
				<span v-tooltip="publishedTooltip" class="min-w-0 truncate">
					{{ publishedLabel }}
				</span>
				<div
					v-if="primaryFile"
					class="h-1.5 w-1.5 flex-shrink-0 rounded-full bg-primary opacity-30"
				></div>
				<span v-if="primaryFile" class="flex-shrink-0">
					{{ primaryFileSizeLabel }}
				</span>
			</div>
		</div>
		<ButtonLink
			v-if="primaryFile && showDownload"
			v-tooltip="circular ? formatMessage(messages.download) : null"
			:type="downloadButtonType"
			:color="downloadButtonColor"
			:href="primaryFileDownloadUrl"
			:download="primaryFile.filename"
			:aria-label="
				formatMessage(messages.downloadVersion, {
					version: version.version_number,
				})
			"
			:class="circular ? '!w-9 !rounded-full !px-0' : undefined"
			@click="emit('download')"
		>
			<DownloadIcon aria-hidden="true" />
			<template v-if="!circular">
				{{ formatMessage(messages.download) }}
			</template>
		</ButtonLink>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { DownloadIcon, RadioButtonCheckedIcon, RadioButtonIcon } from '@modrinth/assets'
import { ButtonLink } from '@modrinth/ui'
import {
	type CdnDownloadReason,
	defineMessages,
	truncatedTooltip,
	useFormatBytes,
	useFormatDateTime,
	useRelativeTime,
	useVIntl,
} from '@modrinth/ui'
import VersionChannelTag from '@modrinth/ui/src/components/version/VersionChannelTag.vue'
import { capitalizeString, type DisplayProjectType } from '@modrinth/utils'
import { computed, ref } from 'vue'

defineOptions({
	name: 'CompatibleVersionCard',
})

type DownloadModalProject = Omit<Labrinth.Projects.v2.Project, 'project_type'> & {
	project_type: DisplayProjectType
	actualProjectType: Labrinth.Projects.v2.ProjectType
}

const props = withDefaults(
	defineProps<{
		project: DownloadModalProject
		version: Labrinth.Versions.v3.Version
		downloadReason?: CdnDownloadReason
		currentGameVersion?: string | null
		currentPlatform?: string | null
		color?: 'brand' | 'standard'
		type?: 'standard' | 'transparent'
		circular?: boolean
		selectable?: boolean
		selected?: boolean
		showDownload?: boolean
	}>(),
	{
		downloadReason: 'standalone',
		currentGameVersion: null,
		currentPlatform: null,
		color: 'brand',
		type: 'standard',
		circular: false,
		selectable: false,
		selected: false,
		showDownload: true,
	},
)

const emit = defineEmits<{
	download: []
	select: []
}>()

const { createProjectDownloadUrl } = useCdnDownloadContext()
const { formatMessage } = useVIntl()
const formatBytes = useFormatBytes()
const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'long',
})
const formatRelativeTime = useRelativeTime()

const versionNumberRef = ref<HTMLElement | null>(null)

const primaryFile = computed<Labrinth.Versions.v3.VersionFile | null>(() => {
	return props.version.files?.find((file) => file.primary) || props.version.files?.[0] || null
})

const primaryFileDownloadUrl = computed(() => {
	if (!primaryFile.value) return '#'

	return createProjectDownloadUrl(primaryFile.value.url, {
		reason: props.downloadReason,
		gameVersion: props.currentGameVersion ?? undefined,
		loader: props.currentPlatform ?? undefined,
	})
})

const publishedLabel = computed(() =>
	capitalizeString(formatRelativeTime(props.version.date_published)),
)
const publishedTooltip = computed(() => formatDateTime(props.version.date_published))
const downloadButtonType = computed(() => {
	if (props.type === 'transparent') return 'quiet'
	return props.color === 'brand' ? 'colored' : 'base'
})
const downloadButtonColor = computed(() =>
	downloadButtonType.value === 'colored' || downloadButtonType.value === 'quiet'
		? props.color === 'brand'
			? 'brand'
			: undefined
		: undefined,
)
const primaryFileSizeLabel = computed(() => {
	if (!primaryFile.value) return ''
	return formatBytes(primaryFile.value.size)
})

function select() {
	if (!props.selectable) return
	emit('select')
}

const messages = defineMessages({
	downloadVersion: {
		id: 'project.download.download-version',
		defaultMessage: 'Download {version}',
	},
	download: {
		id: 'project.download.download',
		defaultMessage: 'Download',
	},
})
</script>
